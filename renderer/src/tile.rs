use maths::{geometry::Shape, linear::Vec2f};

use crate::{
    asset_manager::AssetStore,
    colour::RGB,
    model::ProjectedTriangle,
    renderer::RendererState,
    sat,
    texture::Texture,
    util::{mip_level, normalise_depth},
    DEBUG_TILES, TILE_HEIGHT, TILE_WIDTH,
};

enum Cover {
    Partial,
    Full,
}

struct Bounds {
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
}

struct Tile {
    bounds: Bounds,
    points: [Vec2f; 4],
    triangles: Vec<(Cover, *const ProjectedTriangle)>,
}

#[derive(Default)]
pub struct TileRenderer {
    tiles_x: usize,
    tiles_y: usize,
    tiles: Vec<Tile>,

    #[cfg(feature = "multithreaded")]
    threaded: multithreading::ThreadedRenderer,
}

impl TileRenderer {
    pub fn update_viewport(&mut self, width: usize, height: usize) {
        self.tiles_x = (width + TILE_WIDTH - 1) / TILE_WIDTH;
        self.tiles_y = (height + TILE_HEIGHT - 1) / TILE_HEIGHT;
        self.tiles.clear();

        for y in 0..self.tiles_y {
            for x in 0..self.tiles_x {
                let bounds = Bounds {
                    min_x: x * TILE_WIDTH,
                    min_y: y * TILE_HEIGHT,
                    max_x: ((x + 1) * TILE_WIDTH).min(width),
                    max_y: ((y + 1) * TILE_HEIGHT).min(height),
                };

                let points = [
                    Vec2f::new(bounds.min_x as f32, bounds.min_y as f32),
                    Vec2f::new(bounds.min_x as f32, bounds.max_y as f32),
                    Vec2f::new(bounds.max_x as f32, bounds.max_y as f32),
                    Vec2f::new(bounds.max_x as f32, bounds.min_y as f32),
                ];

                self.tiles.push(Tile {
                    bounds,
                    points,
                    triangles: Vec::new(),
                });
            }
        }
    }

    pub fn render(
        &mut self,
        state: &mut RendererState,
        textures: &AssetStore<Texture>,
        triangles: &[ProjectedTriangle],
    ) {
        self.place_triangles(triangles);

        #[cfg(feature = "multithreaded")]
        self.threaded
            .render(&mut state.framebuffer, textures, &mut self.tiles);

        #[cfg(not(feature = "multithreaded"))]
        for tile in self.tiles.iter_mut() {
            for (cover, triangle) in tile.triangles.drain(..) {
                let triangle = unsafe { &*triangle };

                match cover {
                    Cover::Full => {
                        Self::render_full_tile(
                            state,
                            textures,
                            triangle,
                            &tile.points,
                            &tile.bounds,
                        );
                    }
                    Cover::Partial => {
                        Self::render_partial_tile(
                            state,
                            textures,
                            triangle,
                            &tile.points,
                            &tile.bounds,
                        );
                    }
                }
            }
        }
    }

    fn place_triangles(&mut self, triangles: &[ProjectedTriangle]) {
        for triangle in triangles.iter() {
            let triangle_aabb = triangle.bounds();
            let x_min = (triangle_aabb.min.x as usize / TILE_WIDTH).max(0);
            let x_max = ((triangle_aabb.max.x.ceil() as usize + TILE_WIDTH - 1) / TILE_WIDTH)
                .min(self.tiles_x);
            let y_min = (triangle_aabb.min.y as usize / TILE_HEIGHT).max(0);
            let y_max = ((triangle_aabb.max.y.ceil() as usize + TILE_HEIGHT - 1) / TILE_HEIGHT)
                .min(self.tiles_y);

            for y in y_min..y_max {
                for x in x_min..x_max {
                    let index = y * self.tiles_x + x;
                    let tile = &mut self.tiles[index];

                    match sat::overlap_test(
                        &tile.points,
                        triangle.vertices.points(),
                        &triangle.sat_edges,
                    ) {
                        sat::Overlap::None => {}

                        sat::Overlap::Partial => {
                            tile.triangles.push((Cover::Partial, triangle));
                        }

                        sat::Overlap::Full => {
                            tile.triangles.push((Cover::Full, triangle));
                        }
                    }
                }
            }
        }
    }

    fn render_full_tile(
        state: &mut RendererState,
        textures: &AssetStore<Texture>,
        triangle: &ProjectedTriangle,
        tile_points: &[Vec2f; 4],
        tile_bounds: &Bounds,
    ) {
        let texture = textures.get(triangle.texture_id.unwrap()).unwrap();

        let mut index = tile_bounds.min_y * state.width() + tile_bounds.min_x;
        let mut point = tile_points[0] + 0.5;

        for y in tile_bounds.min_y..tile_bounds.max_y {
            for x in tile_bounds.min_x..tile_bounds.max_x {
                // get barycentric coordinates
                let barycentric = triangle
                    .vertices
                    .barycentric_from_inv_area(point, triangle.two_area_inv);

                // check if coordinates are in triangle (can be skipped for full cover)

                // calculate depth
                let depth = 1.0 / barycentric.dot(triangle.depth_inv);

                // check depth in
                if depth < unsafe { *state.depth().get_unchecked(index) } {
                    let col_a = triangle.col_depth[0] * barycentric.x;
                    let col_b = triangle.col_depth[1] * barycentric.y;
                    let col_c = triangle.col_depth[2] * barycentric.z;
                    let colour = (col_a + col_b + col_c) * depth;

                    let u = (triangle.tex_coords_depth[0].x * barycentric.x
                        + triangle.tex_coords_depth[1].x * barycentric.y
                        + triangle.tex_coords_depth[2].x * barycentric.z)
                        * depth;
                    let v = (triangle.tex_coords_depth[0].y * barycentric.x
                        + triangle.tex_coords_depth[1].y * barycentric.y
                        + triangle.tex_coords_depth[2].y * barycentric.z)
                        * depth;

                    // let normal_depth = normalise_depth(depth);
                    // let mip_level = mip_level(normal_depth, 0.0);

                    let colour = unsafe { texture.sample_unchecked(u, v, 0) };

                    // SAFETY: Tile's integer bounds are within screen bounds
                    unsafe {
                        *state.pixels_mut().get_unchecked_mut(index) = RGB::from(colour);
                        *state.depth_mut().get_unchecked_mut(index) = depth;
                    }
                }

                index += 1;
                point.x += 1.0;
            }

            index += state.width() - (tile_bounds.max_x - tile_bounds.min_x);
            point.x = tile_points[0].x + 0.5;
            point.y += 1.0;
        }

        if DEBUG_TILES {
            let index = (tile_bounds.min_y + tile_bounds.max_y) / 2 * state.width()
                + (tile_bounds.min_x + tile_bounds.max_x) / 2;
            state.pixels_mut()[index] = RGB::CYAN;
        }
    }

    fn render_partial_tile(
        state: &mut RendererState,
        textures: &AssetStore<Texture>,
        triangle: &ProjectedTriangle,
        tile_points: &[Vec2f; 4],
        tile_bounds: &Bounds,
    ) {
        let texture = textures.get(triangle.texture_id.unwrap()).unwrap();

        let mut index = tile_bounds.min_y * state.width() + tile_bounds.min_x;
        let mut point = tile_points[0] + 0.5;

        for y in tile_bounds.min_y..tile_bounds.max_y {
            for x in tile_bounds.min_x..tile_bounds.max_x {
                // get barycentric coordinates
                let barycentric = triangle
                    .vertices
                    .barycentric_from_inv_area(point, triangle.two_area_inv);

                // check if coordinates are in triangle (can be skipped for full cover)
                if barycentric.x.is_sign_positive()
                    && barycentric.y.is_sign_positive()
                    && barycentric.z.is_sign_positive()
                {
                    // calculate depth
                    let depth = 1.0 / barycentric.dot(triangle.depth_inv);

                    // check depth in
                    if depth < unsafe { *state.depth().get_unchecked(index) } {
                        let col_a = triangle.col_depth[0] * barycentric.x;
                        let col_b = triangle.col_depth[1] * barycentric.y;
                        let col_c = triangle.col_depth[2] * barycentric.z;
                        let colour = (col_a + col_b + col_c) * depth;

                        let u = (triangle.tex_coords_depth[0].x * barycentric.x
                            + triangle.tex_coords_depth[1].x * barycentric.y
                            + triangle.tex_coords_depth[2].x * barycentric.z)
                            * depth;
                        let v = (triangle.tex_coords_depth[0].y * barycentric.x
                            + triangle.tex_coords_depth[1].y * barycentric.y
                            + triangle.tex_coords_depth[2].y * barycentric.z)
                            * depth;

                        // let normal_depth = normalise_depth(depth);
                        // let mip_level = mip_level(normal_depth, 0.0);

                        let colour = unsafe { texture.sample_unchecked(u, v, 0) };

                        // SAFETY: Tile's integer bounds are within screen bounds
                        unsafe {
                            *state.pixels_mut().get_unchecked_mut(index) = RGB::from(colour);
                            *state.depth_mut().get_unchecked_mut(index) = depth;
                        }
                    }
                }

                index += 1;
                point.x += 1.0;
            }

            index += state.width() - (tile_bounds.max_x - tile_bounds.min_x);
            point.x = tile_points[0].x + 0.5;
            point.y += 1.0;
        }

        if DEBUG_TILES {
            let index = (tile_bounds.min_y + tile_bounds.max_y) / 2 * state.width()
                + (tile_bounds.min_x + tile_bounds.max_x) / 2;
            state.pixels_mut()[index] = RGB::MAGENTA;
        }
    }
}

#[cfg(feature = "multithreaded")]
mod multithreading {
    use std::{
        cell::UnsafeCell,
        sync::{
            atomic::{AtomicIsize, Ordering},
            Arc,
        },
    };

    use maths::linear::Vec2f;
    use rayon::{ThreadPool, ThreadPoolBuilder};

    use crate::{
        asset_manager::AssetStore,
        colour::RGB,
        framebuffer::Framebuffer,
        model::ProjectedTriangle,
        texture::Texture,
        util::{mip_level, normalise_depth},
        THREADS,
    };

    use super::{Bounds, Cover, Tile};

    struct SharedState<'a> {
        textures: &'a AssetStore<Texture>,
        colour_buffer: &'a [UnsafeCell<RGB>],
        depth_buffer: &'a [UnsafeCell<f32>],
        tiles: &'a [UnsafeCell<Tile>],
    }

    unsafe impl Sync for SharedState<'_> {}
    unsafe impl Send for SharedState<'_> {}

    pub struct ThreadedRenderer {
        workers: ThreadPool,
        tiles_available: Arc<AtomicIsize>,
    }

    impl Default for ThreadedRenderer {
        fn default() -> Self {
            Self::new(THREADS)
        }
    }

    impl ThreadedRenderer {
        pub fn new(num_threads: usize) -> Self {
            let workers = ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap();

            let tiles_available = Arc::new(AtomicIsize::new(isize::MIN));

            Self {
                workers,
                tiles_available,
            }
        }

        pub fn render(
            &self,
            framebuffer: &mut Framebuffer,
            textures: &AssetStore<Texture>,
            tiles: &mut [Tile],
        ) {
            self.tiles_available
                .store(tiles.len().try_into().unwrap(), Ordering::Release);
            let shared_state = unsafe {
                Arc::new(SharedState {
                    textures,
                    colour_buffer: core::mem::transmute(framebuffer.pixels()),
                    depth_buffer: core::mem::transmute(framebuffer.depth()),
                    tiles: core::mem::transmute(tiles),
                })
            };

            self.workers.scope(|s| {
                s.spawn_broadcast(|s, b| loop {
                    let Ok(tiles_available) = self.tiles_available.fetch_update(
                        Ordering::Release,
                        Ordering::Acquire,
                        |x| {
                            if x < 1 {
                                Some(isize::MIN)
                            } else {
                                Some(x - 32)
                            }
                        },
                    ) else {
                        return;
                    };

                    if tiles_available == isize::MIN {
                        return;
                    };

                    let from = (tiles_available - 32).max(0);
                    let to = tiles_available;

                    for i in from..to {
                        let tile =
                            unsafe { &mut *shared_state.tiles.get(i as usize).unwrap().get() };

                        for (cover, triangle) in tile.triangles.drain(..) {
                            let triangle = unsafe { &*triangle };

                            match cover {
                                Cover::Full => {
                                    Self::render_full_tile(
                                        shared_state.colour_buffer,
                                        shared_state.depth_buffer,
                                        shared_state.textures,
                                        triangle,
                                        &tile.points,
                                        &tile.bounds,
                                        framebuffer.width(),
                                    );
                                }
                                Cover::Partial => {
                                    Self::render_partial_tile(
                                        shared_state.colour_buffer,
                                        shared_state.depth_buffer,
                                        shared_state.textures,
                                        triangle,
                                        &tile.points,
                                        &tile.bounds,
                                        framebuffer.width(),
                                    );
                                }
                            }
                        }
                    }
                });
            });
        }

        fn render_partial_tile(
            colour_buffer: &[UnsafeCell<RGB>],
            depth_buffer: &[UnsafeCell<f32>],
            textures: &AssetStore<Texture>,
            triangle: &ProjectedTriangle,
            tile_points: &[Vec2f; 4],
            tile_bounds: &Bounds,
            width: usize,
        ) {
            let texture = textures.get(triangle.texture_id.unwrap()).unwrap();

            let mut index = tile_bounds.min_y * width + tile_bounds.min_x;
            let mut point = tile_points[0] + 0.5;

            for y in tile_bounds.min_y..tile_bounds.max_y {
                for x in tile_bounds.min_x..tile_bounds.max_x {
                    // get barycentric coordinates
                    let barycentric = triangle
                        .vertices
                        .barycentric_from_inv_area(point, triangle.two_area_inv);

                    // check if coordinates are in triangle (can be skipped for full cover)
                    if barycentric.x.is_sign_positive()
                        && barycentric.y.is_sign_positive()
                        && barycentric.z.is_sign_positive()
                    {
                        // calculate depth
                        let depth = 1.0 / barycentric.dot(triangle.depth_inv);

                        // check depth in
                        if depth < unsafe { *depth_buffer.get_unchecked(index).get() } {
                            let u = (triangle.tex_coords_depth[0].x * barycentric.x
                                + triangle.tex_coords_depth[1].x * barycentric.y
                                + triangle.tex_coords_depth[2].x * barycentric.z)
                                * depth;
                            let v = (triangle.tex_coords_depth[0].y * barycentric.x
                                + triangle.tex_coords_depth[1].y * barycentric.y
                                + triangle.tex_coords_depth[2].y * barycentric.z)
                                * depth;

                            // let normal_depth = normalise_depth(depth);
                            // let mip_level = mip_level(normal_depth, 0.0);
                            let colour = unsafe { texture.sample_unchecked(u, v, 0) };

                            // SAFETY: Tile's integer bounds are within screen bounds
                            unsafe {
                                *colour_buffer.get_unchecked(index).get() = RGB::from(colour);
                                *depth_buffer.get_unchecked(index).get() = depth;
                            }
                        }
                    }

                    index += 1;
                    point.x += 1.0;
                }

                index += width - (tile_bounds.max_x - tile_bounds.min_x);
                point.x = tile_points[0].x + 0.5;
                point.y += 1.0;
            }
        }

        fn render_full_tile(
            colour_buffer: &[UnsafeCell<RGB>],
            depth_buffer: &[UnsafeCell<f32>],
            textures: &AssetStore<Texture>,
            triangle: &ProjectedTriangle,
            tile_points: &[Vec2f; 4],
            tile_bounds: &Bounds,
            width: usize,
        ) {
            let texture = textures.get(triangle.texture_id.unwrap()).unwrap();

            let mut index = tile_bounds.min_y * width + tile_bounds.min_x;
            let mut point = tile_points[0] + 0.5;

            for y in tile_bounds.min_y..tile_bounds.max_y {
                for x in tile_bounds.min_x..tile_bounds.max_x {
                    // get barycentric coordinates
                    let barycentric = triangle
                        .vertices
                        .barycentric_from_inv_area(point, triangle.two_area_inv);

                    // calculate depth
                    let depth = 1.0 / barycentric.dot(triangle.depth_inv);

                    // check depth in
                    if depth < unsafe { *depth_buffer.get_unchecked(index).get() } {
                        let u = (triangle.tex_coords_depth[0].x * barycentric.x
                            + triangle.tex_coords_depth[1].x * barycentric.y
                            + triangle.tex_coords_depth[2].x * barycentric.z)
                            * depth;
                        let v = (triangle.tex_coords_depth[0].y * barycentric.x
                            + triangle.tex_coords_depth[1].y * barycentric.y
                            + triangle.tex_coords_depth[2].y * barycentric.z)
                            * depth;

                        // let normal_depth = normalise_depth(depth);
                        // let mip_level = mip_level(normal_depth, 0.0);
                        let colour = unsafe { texture.sample_unchecked(u, v, 0) };

                        // SAFETY: Tile's integer bounds are within screen bounds
                        unsafe {
                            *colour_buffer.get_unchecked(index).get() = RGB::from(colour);
                            *depth_buffer.get_unchecked(index).get() = depth;
                        }
                    }

                    index += 1;
                    point.x += 1.0;
                }

                index += width - (tile_bounds.max_x - tile_bounds.min_x);
                point.x = tile_points[0].x + 0.5;
                point.y += 1.0;
            }
        }
    }
}
