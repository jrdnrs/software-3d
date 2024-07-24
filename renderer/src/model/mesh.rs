use std::collections::VecDeque;

use collections::SparseMap;
use maths::{
    geometry::AABB,
    linear::{Mat4f, Vec3f, Vec4f},
};

use crate::{
    asset_manager::{AssetId, Named},
    renderer::RendererState,
    texture::Texture,
};

use super::{
    triangle::{ProjectedTriangle, TriangleProjector},
    vertex::{transform_point, Vertex},
};

pub struct Mesh {
    name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<usize>,
    pub texture_id: Option<AssetId<Texture>>,
    local_bounds: AABB<Vec3f>,

    pub instances: SparseMap<MeshInstance>,
    pub free_ids: VecDeque<usize>,
}

impl Named for Mesh {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Mesh {
    pub fn new(
        name: String,
        vertices: Vec<Vertex>,
        indices: Vec<usize>,
        texture_id: Option<AssetId<Texture>>,
    ) -> Self {
        println!("Mesh created with {} triangles", indices.len() / 3);

        let local_bounds = find_bounds(&vertices);

        Self {
            name,
            vertices,
            indices,
            texture_id,
            local_bounds,

            instances: SparseMap::new(),
            free_ids: VecDeque::new(),
        }
    }

    pub fn spawn_instance(&mut self, local_transform: &Mat4f) -> usize {
        let world_bounds = update_bounding_box(&self.local_bounds, local_transform);
        let world_positions = self
            .vertices
            .iter()
            .map(|vertex| transform_point(vertex.position, local_transform))
            .collect();

        let id = self
            .free_ids
            .pop_front()
            .unwrap_or_else(|| self.instances.len());

        self.instances.insert(
            id,
            MeshInstance {
                world_positions,
                world_bounds,
                view_bounds: world_bounds,
            },
        );

        id
    }

    pub fn remove_instance(&mut self, instance_id: usize) {
        self.instances.remove(instance_id).unwrap();
        self.free_ids.push_back(instance_id);
    }

    pub fn update_instance_world_space(&mut self, instance_id: usize, local_transform: &Mat4f) {
        let instance = self.instances.get_mut(instance_id).unwrap();

        instance.world_bounds = update_bounding_box(&self.local_bounds, local_transform);

        for (world_position, vertex) in instance
            .world_positions
            .iter_mut()
            .zip(self.vertices.iter())
        {
            *world_position = transform_point(vertex.position, local_transform);
        }
    }

    pub fn update_all_view_bounds(&mut self, view_transform: &Mat4f) {
        for instance in self.instances.values_mut() {
            instance.view_bounds = update_bounding_box(&instance.world_bounds, view_transform);
        }
    }

    pub fn iter_instance_triangles<'a>(
        &'a self,
        state: &'a RendererState,
        instance_index: usize,
    ) -> impl Iterator<Item = ProjectedTriangle> + 'a {
        TriangleProjector::new(state, self, self.instances.get(instance_index).unwrap())
    }
}

pub struct MeshInstance {
    pub world_positions: Vec<Vec3f>,
    pub world_bounds: AABB<Vec3f>,
    pub view_bounds: AABB<Vec3f>,
}

impl MeshInstance {
    pub fn view_bounds(&self) -> &AABB<Vec3f> {
        &self.view_bounds
    }
}

fn find_bounds(vertices: &[Vertex]) -> AABB<Vec3f> {
    let mut min = Vec3f::uniform(f32::MAX);
    let mut max = Vec3f::uniform(f32::MIN);

    for vertex in vertices.iter() {
        min = min.min(&vertex.position);
        max = max.max(&vertex.position);
    }

    AABB::new(min, max)
}

/// Performs transformation on current bounding box, and derives a new bounding box from the result
fn update_bounding_box(bounds: &AABB<Vec3f>, transform: &Mat4f) -> AABB<Vec3f> {
    let points = bounds
        .points()
        .map(|point| transform_point(point, transform));

    let mut min = Vec3f::uniform(f32::MAX);
    let mut max = Vec3f::uniform(f32::MIN);

    for point in points.iter() {
        min = min.min(point);
        max = max.max(point);
    }

    AABB::new(min, max)
}
