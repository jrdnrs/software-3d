use std::{
    collections::{HashMap, VecDeque},
    fmt,
    marker::PhantomData,
    path::Path,
};

use ahash::RandomState;
use collections::SparseMap;
use maths::linear::Mat4f;

use crate::{
    model::{load_obj, Mesh, MeshInstance, Model, ModelInstance},
    texture::Texture,
    util::file_name,
};

pub struct AssetId<T> {
    inner: usize,
    _marker: PhantomData<T>,
}

impl<T> AssetId<T> {
    fn new(id: usize) -> Self {
        Self {
            inner: id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for AssetId<T> {}
impl<T> Clone for AssetId<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

pub trait Named {
    fn name(&self) -> &str;
}

pub struct AssetStore<T: Named> {
    values: SparseMap<T>,
    name_to_id: HashMap<String, AssetId<T>, RandomState>,
    free_ids: VecDeque<usize>,
}

impl<T: Named> AssetStore<T> {
    pub fn new() -> Self {
        Self {
            values: SparseMap::new(),
            name_to_id: HashMap::default(),
            free_ids: VecDeque::new(),
        }
    }

    pub fn get(&self, id: AssetId<T>) -> Option<&T> {
        self.values.get(id.inner)
    }

    pub fn get_mut(&mut self, id: AssetId<T>) -> Option<&mut T> {
        self.values.get_mut(id.inner)
    }

    pub fn get_id(&self, name: &str) -> Option<AssetId<T>> {
        self.name_to_id.get(name).copied()
    }

    pub fn insert(&mut self, asset: T) -> AssetId<T> {
        let id = self
            .free_ids
            .pop_front()
            .unwrap_or_else(|| self.values.len());
        let id = AssetId::new(id);
        self.name_to_id.insert(asset.name().to_owned(), id);
        self.values.insert(id.inner, asset);

        id
    }

    pub fn remove(&mut self, id: AssetId<T>) -> Option<T> {
        self.values.remove(id.inner).and_then(|item| {
            self.name_to_id.remove(item.name());
            self.free_ids.push_back(id.inner);
            Some(item)
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.values.values().into_iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.values.values_mut().into_iter()
    }

    pub fn as_slice(&self) -> &[T] {
        self.values.values()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.values.values_mut()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.name_to_id.clear();
        self.free_ids.clear();
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.name_to_id.contains_key(name)
    }
}

pub struct AssetManager {
    pub(crate) models: AssetStore<Model>,
    pub(crate) meshes: AssetStore<Mesh>,
    pub(crate) textures: AssetStore<Texture>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            models: AssetStore::new(),
            meshes: AssetStore::new(),
            textures: AssetStore::new(),
        }
    }

    pub fn spawn_mesh_instance(
        &mut self,
        mesh_id: AssetId<Mesh>,
        local_transform: &Mat4f,
    ) -> AssetId<MeshInstance> {
        let mesh = self.meshes.get_mut(mesh_id).unwrap();
        let instance_id = mesh.spawn_instance(local_transform);
        AssetId::new(instance_id)
    }

    pub fn spawn_model_instance(
        &mut self,
        model_id: AssetId<Model>,
        local_transform: &Mat4f,
    ) -> AssetId<ModelInstance> {
        let model = self.models.get_mut(model_id).unwrap();

        let model_instance_id = model
            .free_ids
            .pop_front()
            .unwrap_or_else(|| model.instances.len());

        let mesh_instance_ids = model
            .mesh_ids
            .iter()
            .map(|id| {
                let mesh = self.meshes.get_mut(*id).unwrap();
                mesh.spawn_instance(local_transform)
            })
            .collect();

        let model_instance = ModelInstance { mesh_instance_ids };

        model.instances.insert(model_instance_id, model_instance);

        AssetId::new(model_instance_id)
    }

    pub fn model_from_obj_path(
        &mut self,
        path: impl AsRef<Path> + fmt::Debug,
        triangulate: bool,
        reverse_winding: bool,
        flip_uv_y: bool,
    ) -> Result<AssetId<Model>, anyhow::Error> {
        let obj = load_obj(path.as_ref(), triangulate, reverse_winding, flip_uv_y)?;

        let mut texture_ids = Vec::with_capacity(obj.textures.len());
        for texture in obj.textures {
            if let Some(id) = self.textures.get_id(texture.name()) {
                texture_ids.push(id);
            } else {
                texture_ids.push(self.textures.insert(texture));
            }
        }

        let mut mesh_ids = Vec::with_capacity(obj.meshes.len());
        for (mut mesh, texture_index) in obj.meshes {
            if self.meshes.contains_name(mesh.name()) {
                continue;
            };

            mesh.texture_id = texture_index.map(|i| texture_ids[i]);
            let mesh_id = self.meshes.insert(mesh);
            mesh_ids.push(mesh_id);
        }

        let name = file_name(path).unwrap();
        let model = Model::new(name, mesh_ids);

        Ok(self.models.insert(model))
    }
}
