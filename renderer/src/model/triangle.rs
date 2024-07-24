use std::{array, slice::ChunksExact};

use maths::{
    geometry::{Segment, Shape, Triangle, AABB},
    linear::{Vec2f, Vec3f},
};

use crate::{asset_manager::AssetId, renderer::RendererState, texture::Texture, NEAR};

use super::{
    mesh::{Mesh, MeshInstance},
    vertex::{clip_edge, transform_point, Vertex},
};

#[derive(Default)]
pub struct ProjectedTriangle {
    pub vertices: Triangle<Vec2f>,
    pub depth_inv: Vec3f,
    pub col_depth: [Vec3f; 3],
    pub tex_coords_depth: [Vec2f; 3],

    pub two_area_inv: f32,
    pub sat_edges: [Vec2f; 3],
    pub texture_id: Option<AssetId<Texture>>
}

impl ProjectedTriangle {
    pub fn bounds(&self) -> AABB<Vec2f> {
        self.vertices.extents()
    }

    pub fn is_back_facing(&self) -> bool {
        self.two_area_inv.is_sign_negative()
    }

    pub fn is_textured(&self) -> bool {
        self.texture_id.is_some()
    }
}

pub struct TriangleProjector<'a> {
    state: &'a RendererState,
    mesh: &'a Mesh,
    instance: &'a MeshInstance,

    indices_iter: ChunksExact<'a, usize>,
    split_triangle: Option<ProjectedTriangle>,
}

impl<'a> TriangleProjector<'a> {
    pub fn new(state: &'a RendererState, mesh: &'a Mesh, instance: &'a MeshInstance) -> Self {
        Self {
            state,
            mesh,
            instance,

            indices_iter: mesh.indices.chunks_exact(3),
            split_triangle: None,
        }
    }
}

impl<'a> Iterator for TriangleProjector<'a> {
    type Item = ProjectedTriangle;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(triangle) = self.split_triangle.take() {
            return Some(triangle);
        }

        let indices = self.indices_iter.next()?;
        let vertices = array::from_fn(|i| Vertex {
            position: transform_point(
                self.instance.world_positions[indices[i]],
                self.state.camera.view_transform(),
            ),
            colour: self.mesh.vertices[indices[i]].colour,
            tex_coord: self.mesh.vertices[indices[i]].tex_coord,
        });

        match clip_triangle(vertices) {
            ClipResult::None => self.next(),

            ClipResult::One(vertices) => Some(project_triangle(self.state, vertices, self.mesh.texture_id)),

            ClipResult::Two(triangle1, triangle2) => {
                self.split_triangle = Some(project_triangle(self.state, triangle2, self.mesh.texture_id));
                Some(project_triangle(self.state, triangle1, self.mesh.texture_id))
            }
        }
    }
}

enum ClipResult {
    None,
    One([Vertex; 3]),
    Two([Vertex; 3], [Vertex; 3]),
}

fn clip_triangle(vertices: [Vertex; 3]) -> ClipResult {
    let v0_out_bounds = ((vertices[0].position.z < NEAR) as usize) << 2;
    let v1_out_bounds = ((vertices[1].position.z < NEAR) as usize) << 1;
    let v2_out_bounds = (vertices[2].position.z < NEAR) as usize;
    let out_bounds = v0_out_bounds | v1_out_bounds | v2_out_bounds;

    match out_bounds {
        0b000 => ClipResult::One(vertices),
        0b111 => ClipResult::None,

        0b001 => {
            let v2 = clip_edge(vertices[1], vertices[2]);
            let v3 = clip_edge(vertices[0], vertices[2]);
            ClipResult::Two([vertices[0], vertices[1], v2], [v2, v3, vertices[0]])
        }

        0b010 => {
            let v1 = clip_edge(vertices[0], vertices[1]);
            let v3 = clip_edge(vertices[2], vertices[1]);
            ClipResult::Two([vertices[2], vertices[0], v1], [v1, v3, vertices[2]])
        }

        0b100 => {
            let v0 = clip_edge(vertices[2], vertices[0]);
            let v3 = clip_edge(vertices[1], vertices[0]);
            ClipResult::Two([vertices[1], vertices[2], v0], [v0, v3, vertices[1]])
        }

        0b011 => {
            let v1 = clip_edge(vertices[0], vertices[1]);
            let v2 = clip_edge(vertices[0], vertices[2]);
            ClipResult::One([vertices[0], v1, v2])
        }

        0b101 => {
            let v0 = clip_edge(vertices[1], vertices[0]);
            let v2 = clip_edge(vertices[1], vertices[2]);
            ClipResult::One([v0, vertices[1], v2])
        }

        0b110 => {
            let v0 = clip_edge(vertices[2], vertices[0]);
            let v1 = clip_edge(vertices[2], vertices[1]);
            ClipResult::One([v0, v1, vertices[2]])
        }

        _ => unreachable!(),
    }
}

fn project_triangle(state: &RendererState, vertices: [Vertex; 3], texture_id: Option<AssetId<Texture>>) -> ProjectedTriangle {
    let depth_inv = Vec3f::from(array::from_fn(|i| 1.0 / vertices[i].position.z));
    let col_depth = array::from_fn(|i| vertices[i].colour * depth_inv[i]);
    let tex_coords_depth = array::from_fn(|i| vertices[i].tex_coord * depth_inv[i]);

    let triangle = Triangle::from(array::from_fn(|i| {
        Vec2f::new(
            (state.focal_width() * vertices[i].position.x) * depth_inv[i]
                + (state.framebuffer.half_width()),
            (-state.focal_height() * vertices[i].position.y) * depth_inv[i]
                + (state.framebuffer.half_height()),
        )
    }));

    let two_area_inv = 1.0 / Segment::new(triangle.b, triangle.a).edge_side(triangle.c);

    let sat_edges = [
        (triangle.b - triangle.a).perpendicular(),
        (triangle.c - triangle.b).perpendicular(),
        (triangle.a - triangle.c).perpendicular(),
    ];

    ProjectedTriangle {
        vertices: triangle,
        depth_inv,
        col_depth,
        tex_coords_depth,

        two_area_inv,
        sat_edges,
        texture_id
    }
}
