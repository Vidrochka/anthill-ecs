use std::{any::Any, fmt::Debug, collections::HashMap, iter::Enumerate, ops::Add, sync::Arc, slice::Iter};

use tokio::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard, Mutex};
use vec_map::VecMap;

use crate::types::{ArchetypeType, EntityId, ComponentId};

use super::entity_data::EntityData;

const CHUNK_ELEMENTS_COUNT: usize = 64;

pub (crate) trait IComponentsArray where Self: Sync + Send + Debug {
    fn set_component(&mut self, component: Box<dyn Any + Sync + Send>);
    fn remove_component(&mut self, position: usize) -> Box<dyn Any + Sync + Send>;
    fn get_array(&self) -> Arc<dyn Any + Sync + Send>;
}

#[derive(Debug)]
pub struct ComponentsArray<TComponent: Debug + Sync + Send + 'static> {
    components_collection: Arc<RwLock<Vec<TComponent>>>,
}

impl<TComponent: Debug + Sync + Send + 'static> ComponentsArray<TComponent> {
    pub (crate) fn new() -> Self {
        Self {
            components_collection: Arc::new(RwLock::new(Vec::with_capacity(CHUNK_ELEMENTS_COUNT))),
        }
    }
}

impl<TComponent: Debug + Sync + Send + 'static> IComponentsArray for ComponentsArray<TComponent> {
    fn set_component(&mut self, component: Box<dyn Any + Sync + Send>) {
        let component = unsafe { *component.downcast_unchecked::<TComponent>() };
        self.components_collection.blocking_write().push(component);
    }

    fn remove_component(&mut self, position: usize) -> Box<dyn Any + Sync + Send> {
        Box::new(self.components_collection.blocking_write().swap_remove(position))
    }

    fn get_array(&self) -> Arc<dyn Any + Sync + Send> {
        self.components_collection.clone()
    }
}

#[derive(Debug)]
pub struct ArchetypeChunk {
    pub (crate) entity_ids: Vec<EntityId>,
    pub (crate) archetype_components_map: HashMap<ComponentId, Box<dyn IComponentsArray>>,
    pub (crate) chunk_size: usize,
    pub (crate) components_count: usize,
}

impl ArchetypeChunk {
    pub (crate) fn new(archetype_components_map: HashMap<ComponentId, Box<dyn IComponentsArray>>) -> Self {
        Self {
            entity_ids: Vec::with_capacity(CHUNK_ELEMENTS_COUNT),
            archetype_components_map,
            chunk_size: CHUNK_ELEMENTS_COUNT,
            components_count: 0,
        }
    }

    pub (crate) fn is_filled(&self) -> bool {
        self.chunk_size == self.components_count
    }

    pub (crate) fn is_empty(&self) -> bool {
        self.components_count == 0
    }

    pub (crate) fn set_data(&mut self, mut entity_data: EntityData) {
        self.components_count += 1;

        self.entity_ids.push(entity_data.entity_id);

        self.archetype_components_map.iter_mut().for_each(|(component_id, archetype_component_array)| {
            archetype_component_array.set_component(entity_data.entity_components.remove(component_id).unwrap())
        });
    }

    pub (crate) fn remove_data(&mut self, position: usize) -> EntityData {
        self.components_count -= 1;

        let entity_components = self.archetype_components_map.iter_mut().map(|(component_id, archetype_component_array)| -> _ {
            (*component_id, archetype_component_array.remove_component(position))
        }).collect::<HashMap<_,_>>();

        let entity_id = self.entity_ids.swap_remove(position);

        EntityData::new(entity_id, entity_components)
    }

    pub (crate) fn contain_entity(&self, entity_id: &EntityId) -> bool {
        self.entity_ids.contains(entity_id)
    }

    pub (crate) fn entity_position(&self, entity_id: &EntityId) -> usize {
        self.entity_ids.iter().position(|x| *x == *entity_id).unwrap()
    }

    pub (crate) fn get_components_array(&self, component_id: &ComponentId) -> Option<&Box<dyn IComponentsArray>> {
        self.archetype_components_map.get(&component_id)
    }
}

pub trait ArchetypeChunkFabricClosure = Fn() -> ArchetypeChunk;

pub struct Archetype where Self: Sync + Send{
    pub (crate) archetype_type: ArchetypeType,
    pub (crate) chunks: Vec<ArchetypeChunk>,
    pub (crate) archetype_chunk_fabric: Box<dyn ArchetypeChunkFabricClosure + Sync + Send>,
}

impl Debug for Archetype
where Self: Sync + Send
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Archetype")
            .field("archetype_type", &self.archetype_type)
            .field("chunks", &self.chunks)
            .field("archetype_chunk_fabric", &"closure")
            .finish()
    }
}

impl Archetype where Self: Sync + Send {
    pub (crate) fn new(archetype_type: ArchetypeType, archetype_chunk_fabric: Box<dyn ArchetypeChunkFabricClosure + Sync + Send>) -> Self {
        Self {
            archetype_type,
            chunks: Default::default(),
            archetype_chunk_fabric,
        }
    }

    pub  (crate) fn archetype_type(&self) -> &ArchetypeType {
        &self.archetype_type
    }

    pub (crate) fn add_entity(&mut self, entity_data: EntityData) {
        assert_eq!(self.archetype_type.components_count(), entity_data.entity_components.len());

        if let Some(last_chunk) = self.chunks.last_mut() {
            if !last_chunk.is_filled() {
                last_chunk.set_data(entity_data);

                return;
            }
        }

        let mut new_archetype_chunk = (self.archetype_chunk_fabric)();
        new_archetype_chunk.set_data(entity_data);
        self.chunks.push(new_archetype_chunk);
    }

    pub (crate) fn remove_entity(&mut self, entity_id: EntityId) -> EntityData {
        let chunk_number = self.chunks.iter().position(|x| x.contain_entity(&entity_id)).unwrap();

        let entity_position = self.chunks[chunk_number].entity_position(&entity_id);

        let entity_data = self.chunks[chunk_number].remove_data(entity_position);

        // если чанк после удаления компонентов пуст, значит он последний, т.к. все чанки кроме последнего должны быть полностью заняты
        if self.chunks[chunk_number].is_empty() {
            self.chunks.pop();

            return entity_data;
        }

        let is_tail_chunk = self.chunks.len() == chunk_number + 1;

        // если чанк не последний, для более плотной упаковки перемещаем компоненты из последнего чанка в освободившееся место
        if !is_tail_chunk {
            let lust_chunk_number = self.chunks.len() - 1;

            let last_chunk_last_entity_position = self.chunks[lust_chunk_number].components_count - 1;
            let last_chunk_last_entity_data = self.chunks[lust_chunk_number].remove_data(last_chunk_last_entity_position);

            self.chunks[chunk_number].set_data(last_chunk_last_entity_data);

            // если последний чанк пустой, удаляем его
            if self.chunks[lust_chunk_number].is_empty() {
                self.chunks.pop();
            }
        }

        entity_data
    }

    pub (crate) fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub (crate) fn get_chunks(&self) -> Iter<ArchetypeChunk> {
        self.chunks.iter()
    }
}