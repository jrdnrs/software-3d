use maths::linear::{Mat4f, Vec2f, Vec3f, Vec4f};

use crate::NEAR;

#[derive(Clone, Copy, Default, Debug)]
pub struct Vertex {
    pub position: Vec3f,
    pub colour: Vec3f,
    pub tex_coord: Vec2f,
}

pub fn transform_point(point: Vec3f, transform: &Mat4f) -> Vec3f {
    Vec3f::from(*transform * Vec4f::from(point))
}

pub fn clip_edge(in_bounds: Vertex, out_bounds: Vertex) -> Vertex {
    let t = (NEAR - in_bounds.position.z) / (out_bounds.position.z - in_bounds.position.z);

    Vertex {
        position: in_bounds.position.lerp(out_bounds.position, t),
        colour: in_bounds.colour.lerp(out_bounds.colour, t),
        tex_coord: in_bounds.tex_coord.lerp(out_bounds.tex_coord, t),
    }
}
