pub mod archetype;
pub mod entity_builder;
pub mod entity_data_accessor;
pub mod entity_data;
pub mod new_entity_components_info;
pub mod component;

use std::{
    collections::{HashMap, HashSet},
    any::{TypeId, Any},
    fmt::Debug, sync::Arc
};

use tokio::sync::RwLock;
use vec_map::VecMap;

use crate::types::{
    EntityId,
    ArchetypeType,
    ComponentId, AddEntityResult, AddEntityError
};

use self::{archetype::{Archetype, ArchetypeChunk}, /* entity_builder::EntityBuilder,  */entity_data_accessor::ChunkDataAccessor, entity_data::EntityData, component::component_info::ComponentInfo};

// байты
const CHUNK_SIZE: usize = 16_000;

#[derive(Debug, Default)]
pub struct EcsDataManager where Self: Sync + Send{
    free_entity_id: Vec<EntityId>,
    entity_index: VecMap<ArchetypeType>,
    index_count: usize,

    pub (crate) archetype_map: HashMap<ArchetypeType, Archetype>,
    components_info: HashMap<ComponentId, ComponentInfo>,
    //components_count: u32,
}

impl EcsDataManager where Self: Sync + Send {
    pub fn new() -> Self { 
        Self { ..Default::default() }
    }

    pub fn register_component<TComponent: Debug + Sync + Send + 'static>(&mut self) -> ComponentId {
        let component_id = ComponentId::new(TypeId::of::<TComponent>());
        //self.components_count += 1;

        self.components_info.insert(ComponentId::from_type::<TComponent>(), ComponentInfo::new::<TComponent>());
        component_id
    }

    // pub fn new_entity<'a, const COMPONENTS_COUNT: usize>(&'a mut self, archetype: [ComponentId; COMPONENTS_COUNT]) -> EntityBuilder<'a, COMPONENTS_COUNT> {
    //     EntityBuilder::new(archetype, self)
    // }

    // pub fn get_components<'a>(&'a mut self) -> ArchetypeDataAccessorBuilder<'a> {
    //     ArchetypeDataAccessorBuilder::<'a>::new(self)
    // }

    pub fn remove_entity(&mut self, entity_id: EntityId) {
        let archetype_type = self.entity_index.remove(*entity_id).unwrap();
        self.archetype_map.get_mut(&archetype_type).unwrap().remove_entity(entity_id);
    }

    // можно упростить если сделать проверку на наличие архетипа. если есть, не делать новое замыкание
    pub fn add_entity(&mut self, components: Vec<Box<dyn Any + Send + Sync>>) -> AddEntityResult<EntityId> {
        let archetype_type: ArchetypeType = components.iter().map(|component| (**component).type_id().into()).collect::<Vec<ComponentId>>().into();
        
        if let Some(component_id) = archetype_type.check(&self.components_info) {
            return Err(AddEntityError::ComponentNotRegistered { component_id });
        }

        let entity_id = self.free_entity_id.pop().unwrap_or_else(|| {
            let new_entity_id = EntityId::new(self.index_count);
            self.index_count += 1;
            new_entity_id
        });

        let components_map = components.into_iter().map(|component| ((*component).type_id().into(), component)).collect::<HashMap<_,_>>();
        let entity_data = EntityData::new(entity_id, components_map);
        let archetype_type = entity_data.build_archetype_type();

        let components_array_build_closure_collection = archetype_type.iter().map(|component_id| {
            let components_array_build_closure = self.components_info.get(component_id).unwrap().component_array_fabric_cloure.clone();
            (*component_id, components_array_build_closure)
        }).collect::<Vec<(_,_)>>();

        let single_entity_size: usize = archetype_type.iter().map(|x| self.components_info.get(x).unwrap().size).sum();
        let chunk_components_count = single_entity_size / CHUNK_SIZE;

        let build_archetype_chunk_clousre = move || -> ArchetypeChunk {
            let components_array_collection = components_array_build_closure_collection.iter().map(|(component_id, components_array_build_closure)| {
                (*component_id, (components_array_build_closure)(chunk_components_count))
            }).collect::<HashMap<_,_>>();

            ArchetypeChunk::new(components_array_collection)
        };

        let archetype = self.archetype_map.entry(archetype_type.clone())
            .or_insert(Archetype::new(archetype_type.clone(), Box::new(build_archetype_chunk_clousre)));

        archetype.add_entity(entity_data);

        self.entity_index.insert(entity_id.id(), archetype_type);

        Ok(entity_id)
    }
}
