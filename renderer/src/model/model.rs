use std::{
    collections::{HashMap, VecDeque},
    fmt,
    path::{Path, PathBuf},
};

use collections::SparseMap;
use maths::linear::{Vec2f, Vec3f};

use crate::{
    asset_manager::{AssetId, Named},
    texture::Texture,
    util::normalise_path,
};

use super::{mesh::Mesh, vertex::Vertex, MeshInstance};

pub struct Model {
    name: String,
    pub mesh_ids: Vec<AssetId<Mesh>>,

    pub instances: SparseMap<ModelInstance>,
    pub free_ids: VecDeque<usize>,
}

impl Named for Model {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Model {
    pub fn new(name: String, mesh_ids: Vec<AssetId<Mesh>>) -> Self {
        Self {
            name,
            mesh_ids,
            instances: SparseMap::new(),
            free_ids: VecDeque::new(),
        }
    }
}

pub struct ModelInstance {
    pub mesh_instance_ids: Vec<usize>,
}

pub struct Obj {
    pub meshes: Vec<(Mesh, Option<usize>)>,
    pub textures: Vec<Texture>,
}

pub fn load_obj(
    path: impl AsRef<Path> + fmt::Debug,
    triangulate: bool,
    reverse_winding: bool,
    flip_uv_y: bool,
) -> Result<Obj, anyhow::Error> {
    let (obj_models, mtls) = tobj::load_obj(
        path.as_ref(),
        &tobj::LoadOptions {
            single_index: true,
            triangulate,
            ignore_points: false,
            ignore_lines: false,
        },
    )?;
    let mtls = mtls?;

    let textures = load_mtls(&mtls, path.as_ref().parent().unwrap());
    let meshes = load_meshes(&obj_models, reverse_winding, flip_uv_y);

    Ok(Obj { meshes, textures })
}

fn load_mtls(mtls: &[tobj::Material], dir: impl AsRef<Path>) -> Vec<Texture> {
    let mut textures = Vec::with_capacity(mtls.len());

    for material in mtls.iter() {
        let Some(texture_name) = &material.diffuse_texture else {
            continue;
        };

        let texture_path = dir.as_ref().join(texture_name);
        let texture_path = normalise_path(texture_path);

        let texture = Texture::from_path_png(texture_path).unwrap();
        textures.push(texture);
    }

    textures
}

fn load_meshes(
    obj_models: &[tobj::Model],
    reverse_winding: bool,
    flip_uv_y: bool,
) -> Vec<(Mesh, Option<usize>)> {
    let mut meshes = Vec::with_capacity(obj_models.len());
    let mut names = HashMap::new();

    for obj_model in obj_models.iter() {
        let mut indices = obj_model
            .mesh
            .indices
            .iter()
            .map(|i: &u32| *i as usize)
            .collect::<Vec<usize>>();

        if reverse_winding {
            for i in indices.chunks_exact_mut(3) {
                i.swap(0, 2);
            }
        }

        let vertices = obj_model
            .mesh
            .positions
            .chunks_exact(3)
            .zip(obj_model.mesh.texcoords.chunks_exact(2))
            .map(|(position, tex_coord)| {
                let tex_coord = Vec2f::new(
                    tex_coord[0],
                    if flip_uv_y {
                        1.0 - tex_coord[1]
                    } else {
                        tex_coord[1]
                    },
                );

                Vertex {
                    position: Vec3f::new(position[0], position[1], position[2]),
                    colour: Vec3f::new(position[0], position[1], position[2]),
                    tex_coord,
                }
            })
            .collect();

        let duplicates = names.entry(&obj_model.name).or_insert(0);
        let name = if *duplicates == 0 {
            obj_model.name.to_owned()
        } else {
            format!("{} ({})", obj_model.name, duplicates)
        };
        *duplicates += 1;

        meshes.push((
            Mesh::new(name, vertices, indices, None),
            obj_model.mesh.material_id,
        ));
    }

    meshes
}
