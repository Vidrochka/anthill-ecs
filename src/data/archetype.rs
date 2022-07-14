use std::{any::Any, fmt::Debug, collections::HashMap, iter::Enumerate};

use tokio::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard};
use vec_map::VecMap;

use crate::types::{ArchetypeType, EntityId, ComponentId};

pub (crate) trait IComponentsArray where Self: Sync + Send + Debug {
    fn add_component(&self, idx: usize, component: Box<dyn Any + Sync + Send>);
    fn get_collection_write_lock(&self) -> Box<dyn Any + Sync + Send>;
    fn get_collection_read_lock(&self) -> Box<dyn Any + Sync + Send>;
}

#[derive(Debug)]
pub struct ComponentsArray<TComponent: Clone + Debug + Sync + Send + 'static> {
    components_collection: RwLock<VecMap<TComponent>>,
}

impl<TComponent: Clone + Debug + Sync + Send + 'static> ComponentsArray<TComponent> {
    pub (crate) fn new() -> Self {
        Self {
            components_collection: RwLock::new(VecMap::<TComponent>::new())
        }
    }

    fn get_collection_write_lock(&self) -> RwLockWriteGuard<VecMap<TComponent>> {
        self.components_collection.blocking_write()
    }

    fn get_collection_read_lock(&self) -> RwLockReadGuard<VecMap<TComponent>> {
        self.components_collection.blocking_read()
    }

    fn set_component(&self, idx: usize, component: TComponent) {
        let mut wl = self.get_collection_write_lock();
        wl.insert(idx, component);
    }

    fn get_component(&self, idx: usize) -> TComponent {
        let rl = self.get_collection_read_lock();
        let component = rl.get(idx).unwrap();
        let component = (*component).clone();
        component
    }
}

impl<TComponent: Clone + Debug + Sync + Send + 'static> IComponentsArray for ComponentsArray<TComponent> {
    fn add_component(&self, idx: usize, component: Box<dyn Any + Sync + Send>) {
        let component: Box<TComponent> = unsafe { component.downcast_unchecked::<TComponent>() };
        self.get_collection_write_lock().insert(idx, *component);
    }

    fn get_collection_write_lock(&self) -> Box<dyn Any + Sync + Send> {
        Box::new(self.get_collection_write_lock())
    }

    fn get_collection_read_lock(&self) -> Box<dyn Any + Sync + Send> {
        Box::new(self.get_collection_read_lock())
    }
}

#[derive(Debug)]
pub struct Archetype where Self: Sync + Send{
    archetype_type: ArchetypeType,
    entity_ids: Vec<EntityId>,
    archetype_idx: HashMap<ComponentId, usize>,
    components_collection: Vec<Box<dyn IComponentsArray>>,
}

impl Archetype where Self: Sync + Send {
    pub (crate) fn new(archetype_type: ArchetypeType, components_collection: Vec<Box<dyn IComponentsArray>>) -> Self {
        assert_eq!(archetype_type.len(), components_collection.len());

        let archetype_idx = archetype_type.component_ids.iter().enumerate().map(|(idx, component_id)| {
            (*component_id, idx)
        }).collect::<HashMap<_,_>>();

        Self {
            archetype_type,
            entity_ids: Default::default(),
            archetype_idx,
            components_collection
        }
    }

    pub fn archetype_type(&self) -> &ArchetypeType {
        &self.archetype_type
    }

    pub fn get_component_write_lock(&self, component_id: &ComponentId) -> Box<dyn Any + Sync + Send> {
        self.components_collection[*self.archetype_idx.get(component_id).unwrap()].get_collection_write_lock()
    }

    pub fn get_component_read_lock(&self, component_id: &ComponentId) -> Box<dyn Any + Sync + Send> {
        self.components_collection[*self.archetype_idx.get(component_id).unwrap()].get_collection_read_lock()
    }

    pub (crate) fn add_entity(&mut self, entity_id: EntityId, components: Vec<Box<dyn Any + Sync + Send>>) {
        assert_eq!(self.archetype_type.components_count(), components.len());
        assert!(!self.entity_ids.contains(&entity_id));
        
        self.entity_ids.push(entity_id);
        let components_idx = self.entity_ids.len() - 1;

        let mut components = components;
        self.components_collection.iter().rev().for_each(|components_array|
            components_array.add_component(components_idx, components.pop().unwrap())
        )
    }
}