pub mod archetype;
pub mod entity_builder;
pub mod entity_data_accessor;

use std::{
    collections::{HashMap, HashSet},
    any::TypeId,
    fmt::Debug, sync::Arc
};

use tokio::sync::RwLock;

use crate::types::{
    EntityId,
    ArchetypeType,
    ComponentId
};

use self::{archetype::Archetype, entity_builder::EntityBuilder, entity_data_accessor::ArchetypeDataAccessor};

#[derive(Debug, Default)]
pub struct EcsDataManager where Self: Sync + Send{
    free_entity_id: Vec<EntityId>,
    entity_index: HashMap<EntityId, ArchetypeType>,
    index_count: u32,

    pub (crate) archetype_map: HashMap<ArchetypeType, Archetype>,
    component_type_id_to_id_mapping: HashMap<TypeId, ComponentId>,
    //components_count: u32,
}

impl EcsDataManager where Self: Sync + Send {
    pub fn new() -> Self { 
        Self { ..Default::default() }
    }

    pub fn register_component<TComponent: 'static>(&mut self) -> ComponentId {
        let component_id = ComponentId::new(TypeId::of::<TComponent>());
        //self.components_count += 1;

        self.component_type_id_to_id_mapping.insert(TypeId::of::<TComponent>(), component_id.clone());
        component_id
    }

    pub fn get_component_id<TComponent: 'static>(&self) -> ComponentId {
        self.component_type_id_to_id_mapping[&TypeId::of::<TComponent>()]
    } 

    pub fn new_entity<'a, const COMPONENTS_COUNT: usize>(&'a mut self, archetype: [ComponentId; COMPONENTS_COUNT]) -> EntityBuilder<'a, COMPONENTS_COUNT> {
        EntityBuilder::new(archetype, self)
    }

    // pub fn get_components<'a>(&'a mut self) -> ArchetypeDataAccessorBuilder<'a> {
    //     ArchetypeDataAccessorBuilder::<'a>::new(self)
    // }
}
