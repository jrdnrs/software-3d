use maths::linear::{Vec2f, Vec3f};

use crate::model::{Mesh, Vertex};


pub fn unit_quad_mesh() -> Mesh {
    let vertices = vec![
        Vertex {
            position: Vec3f::new(1.0, -1.0, 0.0),
            colour: Vec3f::new(1.0, -1.0, 0.0),
            tex_coord: Vec2f::new(1.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(-1.0, -1.0, 0.0),
            colour: Vec3f::new(-1.0, -1.0, 0.0),
            tex_coord: Vec2f::new(0.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(-1.0, 1.0, 0.0),
            colour: Vec3f::new(-1.0, 1.0, 0.0),
            tex_coord: Vec2f::new(0.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(1.0, 1.0, 0.0),
            colour: Vec3f::new(1.0, 1.0, 0.0),
            tex_coord: Vec2f::new(1.0, 1.0),
        },
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    Mesh::new(String::from("Quad"), vertices, indices, None)
}

pub fn unit_cube_mesh() -> Mesh {
    let vertices = vec![
        Vertex {
            position: Vec3f::new(0.5, -0.5, -0.5),
            colour: Vec3f::new(0.5, -0.5, -0.5),
            tex_coord: Vec2f::new(0.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, -0.5, -0.5),
            colour: Vec3f::new(-0.5, -0.5, -0.5),
            tex_coord: Vec2f::new(1.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, -0.5, 0.5),
            colour: Vec3f::new(-0.5, -0.5, 0.5),
            tex_coord: Vec2f::new(1.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(0.5, -0.5, 0.5),
            colour: Vec3f::new(0.5, -0.5, 0.5),
            tex_coord: Vec2f::new(0.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, -0.5, -0.5),
            colour: Vec3f::new(-0.5, -0.5, -0.5),
            tex_coord: Vec2f::new(0.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(0.5, -0.5, -0.5),
            colour: Vec3f::new(0.5, -0.5, -0.5),
            tex_coord: Vec2f::new(1.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(0.5, 0.5, -0.5),
            colour: Vec3f::new(0.5, 0.5, -0.5),
            tex_coord: Vec2f::new(1.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, 0.5, -0.5),
            colour: Vec3f::new(-0.5, 0.5, -0.5),
            tex_coord: Vec2f::new(0.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, 0.5, -0.5),
            colour: Vec3f::new(-0.5, 0.5, -0.5),
            tex_coord: Vec2f::new(0.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(0.5, 0.5, -0.5),
            colour: Vec3f::new(0.5, 0.5, -0.5),
            tex_coord: Vec2f::new(1.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(0.5, 0.5, 0.5),
            colour: Vec3f::new(0.5, 0.5, 0.5),
            tex_coord: Vec2f::new(1.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, 0.5, 0.5),
            colour: Vec3f::new(-0.5, 0.5, 0.5),
            tex_coord: Vec2f::new(0.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(0.5, -0.5, 0.5),
            colour: Vec3f::new(0.5, -0.5, 0.5),
            tex_coord: Vec2f::new(0.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, -0.5, 0.5),
            colour: Vec3f::new(-0.5, -0.5, 0.5),
            tex_coord: Vec2f::new(1.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, 0.5, 0.5),
            colour: Vec3f::new(-0.5, 0.5, 0.5),
            tex_coord: Vec2f::new(1.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(0.5, 0.5, 0.5),
            colour: Vec3f::new(0.5, 0.5, 0.5),
            tex_coord: Vec2f::new(0.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(0.5, -0.5, -0.5),
            colour: Vec3f::new(0.5, -0.5, -0.5),
            tex_coord: Vec2f::new(0.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(0.5, -0.5, 0.5),
            colour: Vec3f::new(0.5, -0.5, 0.5),
            tex_coord: Vec2f::new(1.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(0.5, 0.5, 0.5),
            colour: Vec3f::new(0.5, 0.5, 0.5),
            tex_coord: Vec2f::new(1.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(0.5, 0.5, -0.5),
            colour: Vec3f::new(0.5, 0.5, -0.5),
            tex_coord: Vec2f::new(0.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, -0.5, 0.5),
            colour: Vec3f::new(-0.5, -0.5, 0.5),
            tex_coord: Vec2f::new(0.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, -0.5, -0.5),
            colour: Vec3f::new(-0.5, -0.5, -0.5),
            tex_coord: Vec2f::new(1.0, 0.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, 0.5, -0.5),
            colour: Vec3f::new(-0.5, 0.5, -0.5),
            tex_coord: Vec2f::new(1.0, 1.0),
        },
        Vertex {
            position: Vec3f::new(-0.5, 0.5, 0.5),
            colour: Vec3f::new(-0.5, 0.5, 0.5),
            tex_coord: Vec2f::new(0.0, 1.0),
        },
    ];

    let indices = vec![
        0, 2, 1, 2, 0, 3, 4, 6, 5, 6, 4, 7, 8, 10, 9, 10, 8, 11, 12, 14, 13, 14, 12, 15, 16, 18,
        17, 18, 16, 19, 20, 22, 21, 22, 20, 23,
    ];

    Mesh::new(String::from("Cube"), vertices, indices, None)
}

pub fn unit_sphere_mesh(resolution: usize) -> Mesh {
    const ORIGINS: [Vec3f; 6] = [
        Vec3f {
            x: -1.0,
            y: -1.0,
            z: -1.0,
        },
        Vec3f {
            x: 1.0,
            y: -1.0,
            z: -1.0,
        },
        Vec3f {
            x: 1.0,
            y: -1.0,
            z: 1.0,
        },
        Vec3f {
            x: -1.0,
            y: -1.0,
            z: 1.0,
        },
        Vec3f {
            x: -1.0,
            y: 1.0,
            z: -1.0,
        },
        Vec3f {
            x: -1.0,
            y: -1.0,
            z: 1.0,
        },
    ];

    const RIGHTS: [Vec3f; 6] = [
        Vec3f {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        Vec3f {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        Vec3f {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        },
        Vec3f {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        },
        Vec3f {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        Vec3f {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
    ];

    const UPS: [Vec3f; 6] = [
        Vec3f {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Vec3f {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Vec3f {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Vec3f {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Vec3f {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        Vec3f {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        },
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let step = 2.0 / resolution as f32;

    for face in 0..6 {
        let origin = ORIGINS[face];
        let right = RIGHTS[face];
        let up = UPS[face];

        for u in 0..resolution {
            for v in 0..resolution {
                indices.push(vertices.len() + v + u * (resolution + 1));
                indices.push(vertices.len() + (v + 1) + (u + 1) * (resolution + 1));
                indices.push(vertices.len() + v + (u + 1) * (resolution + 1));
                indices.push(vertices.len() + (v + 1) + (u + 1) * (resolution + 1));
                indices.push(vertices.len() + v + u * (resolution + 1));
                indices.push(vertices.len() + (v + 1) + u * (resolution + 1));
            }
        }

        for u in 0..=resolution {
            for v in 0..=resolution {
                let p = origin + (right * u as f32 + up * v as f32) * step;

                let x2 = p.x * p.x;
                let y2 = p.y * p.y;
                let z2 = p.z * p.z;

                let n = Vec3f::new(
                    p.x * (1.0 - (y2 + z2) / 2.0 + y2 * z2 / 3.0).sqrt(),
                    p.y * (1.0 - (x2 + z2) / 2.0 + x2 * z2 / 3.0).sqrt(),
                    p.z * (1.0 - (x2 + y2) / 2.0 + x2 * y2 / 3.0).sqrt(),
                );

                vertices.push(Vertex {
                    position: n,
                    colour: n,
                    tex_coord: Vec2f::new(u as f32, v as f32) / resolution as f32,
                });
            }
        }
    }

    Mesh::new(String::from("Sphere"), vertices, indices, None)
}
