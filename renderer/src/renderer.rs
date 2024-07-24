use std::{
    marker::PhantomData,
    time::{SystemTime, UNIX_EPOCH},
};

use collections::SparseMap;
use maths::{
    geometry::AABB,
    linear::{Mat4f, Vec3f},
};

use crate::{
    asset_manager::AssetManager,
    colour::RGB,
    line::LineRenderer,
    model::{Mesh, Model, ProjectedTriangle},
    shapes::{unit_cube_mesh, unit_sphere_mesh},
    texture::Texture,
    tile::TileRenderer,
    util::{focal_dimensions, view_frustum_bounds},
    RES_SCALE,
};

use super::{camera::Camera, framebuffer::Framebuffer};

pub struct RendererState {
    pub framebuffer: Framebuffer,
    pub camera: Camera,
    view_frustum_bounds: AABB<Vec3f>,
    h_fov_rad: f32,
    v_fov_rad: f32,
    focal_width: f32,
    focal_height: f32,
    clear_colour: RGB,
}

impl RendererState {
    pub fn pixels(&self) -> &[RGB] {
        self.framebuffer.pixels()
    }

    pub fn pixels_mut(&mut self) -> &mut [RGB] {
        self.framebuffer.pixels_mut()
    }

    pub fn depth(&self) -> &[f32] {
        self.framebuffer.depth()
    }

    pub fn depth_mut(&mut self) -> &mut [f32] {
        self.framebuffer.depth_mut()
    }

    pub fn width(&self) -> usize {
        self.framebuffer.width()
    }

    pub fn height(&self) -> usize {
        self.framebuffer.height()
    }

    pub fn focal_width(&self) -> f32 {
        self.focal_width
    }

    pub fn focal_height(&self) -> f32 {
        self.focal_height
    }
}

pub struct Renderer {
    state: RendererState,
    tile_renderer: TileRenderer,
    line_renderer: LineRenderer,

    assets: AssetManager,

    projected_triangles: Vec<ProjectedTriangle>,
}

impl Renderer {
    pub fn new(width: usize, height: usize, horiz_fov: f32) -> Self {
        let tile_renderer = TileRenderer::default();
        let line_renderer = LineRenderer::default();

        let framebuffer = Framebuffer::new(width, height);
        let camera = Camera::new();
        let h_fov_rad = horiz_fov.to_radians();
        let v_fov_rad = h_fov_rad / framebuffer.aspect_ratio();
        let (focal_width, focal_height) = focal_dimensions(
            h_fov_rad,
            v_fov_rad,
            framebuffer.half_width(),
            framebuffer.half_height(),
        );
        let view_frustum_bounds = view_frustum_bounds(h_fov_rad, v_fov_rad);

        let state = RendererState {
            framebuffer,
            camera,
            view_frustum_bounds,
            h_fov_rad,
            v_fov_rad,
            focal_height,
            focal_width,
            clear_colour: RGB::hex(0x0a96ed),
        };

        let projected_triangles = Vec::new();
        let assets = AssetManager::new();

        Self {
            state,
            tile_renderer,
            line_renderer,

            assets,

            projected_triangles,
        }
    }

    pub fn internal_width(&self) -> usize {
        self.state.framebuffer.width()
    }

    pub fn internal_height(&self) -> usize {
        self.state.framebuffer.height()
    }

    pub fn pixels(&self) -> &[RGB] {
        self.state.framebuffer.pixels()
    }

    pub fn pixels_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.state.framebuffer.pixels().as_ptr() as *const u8,
                self.state.framebuffer.pixels().len() * std::mem::size_of::<RGB>(),
            )
        }
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.state.camera
    }

    pub fn assets_mut(&mut self) -> &mut AssetManager {
        &mut self.assets
    }

    pub fn set_clear_colour(&mut self, colour: RGB) {
        self.state.clear_colour = colour;
    }

    pub fn render(&mut self) {
        self.state
            .framebuffer
            .clear_colour_buffer(self.state.clear_colour);
        self.state.framebuffer.clear_depth_buffer();

        self.project_meshes();

        self.tile_renderer.render(
            &mut self.state,
            &self.assets.textures,
            &self.projected_triangles,
        );
        // self.line_renderer
        //     .render(&mut self.state, &self.projected_triangles, RGB::WHITE);
    }

    pub fn update_viewport(&mut self, width: usize, height: usize) {
        let width = (width as f32 * RES_SCALE).round() as usize;
        let height = (height as f32 * RES_SCALE).round() as usize;

        self.state.framebuffer = Framebuffer::new(width, height);
        self.tile_renderer.update_viewport(width, height);
        self.line_renderer.update_viewport(width, height);

        self.state.v_fov_rad = self.state.h_fov_rad / self.state.framebuffer.aspect_ratio();
        (self.state.focal_width, self.state.focal_height) = focal_dimensions(
            self.state.h_fov_rad,
            self.state.v_fov_rad,
            self.state.framebuffer.half_width(),
            self.state.framebuffer.half_height(),
        );
        self.state.view_frustum_bounds =
            view_frustum_bounds(self.state.h_fov_rad, self.state.v_fov_rad);
    }

    fn project_meshes(&mut self) {
        self.projected_triangles.clear();
        for mesh in self.assets.meshes.values_mut() {
            mesh.update_all_view_bounds(self.state.camera.view_transform());

            for (i, instance) in mesh.instances.values().into_iter().enumerate() {
                // skip instance if bounding box is not in view frustum
                if self
                    .state
                    .view_frustum_bounds
                    .intersects(instance.view_bounds())
                {
                    for triangle in mesh.iter_instance_triangles(&self.state, i) {
                        // skip triangle if back facing
                        if !triangle.is_back_facing() {
                            self.projected_triangles.push(triangle);
                        }
                    }
                }
            }
        }
    }
}
