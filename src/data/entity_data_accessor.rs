use std::{collections::{HashMap, HashSet}, any::{TypeId, Any}, sync::Arc};

use tokio::sync::{RwLockReadGuard, RwLockWriteGuard, RwLock};
use vec_map::VecMap;

use crate::types::ComponentId;

use super::EcsDataManager;


#[derive(Debug, Default)]
pub struct ArchetypeDataAccessor {
    data: HashMap<ComponentId, Box<dyn Any + Send + Sync>>,
}

impl ArchetypeDataAccessor {
    pub (crate) fn add_data(&mut self, components_type: ComponentId, components: Box<dyn Any + Send + Sync>) {
        self.data.insert(components_type, components);
    }

    pub fn resolve_readonly<TComponent: 'static>(&mut self) -> Option<Box<RwLockReadGuard<VecMap<TComponent>>>> {
        self.data.remove(&TypeId::of::<TComponent>().into())
            .map(|components| unsafe { components.downcast_unchecked::<RwLockReadGuard<VecMap<TComponent>>>() })
    }

    pub fn resolve<TComponent: 'static>(&mut self) -> Option<Box<RwLockWriteGuard<VecMap<TComponent>>>> {
        self.data.remove(&TypeId::of::<TComponent>().into())
            .map(|components| unsafe { components.downcast_unchecked::<RwLockWriteGuard<VecMap<TComponent>>>() })
    }

    pub fn contains<TComponent: 'static>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<TComponent>().into())
    }
}

pub struct ArchetypeDataAccessorBuilder {
    ecs_data_manager: Arc<RwLock<EcsDataManager>>,
    required_components: HashSet<(ComponentId, bool)>,
    addition_components: HashSet<(ComponentId, bool)>,
    except_components: HashSet<ComponentId>,
}

impl ArchetypeDataAccessorBuilder {
    pub (crate) fn new(ecs_data_manager: Arc<RwLock<EcsDataManager>>) -> Self {
        Self {
            ecs_data_manager,
            required_components: Default::default(),
            addition_components: Default::default(),
            except_components: Default::default(),
        }
    }

    pub fn required_component<TComponent: 'static>(&mut self, readonly: bool) -> &mut Self {
        self.required_components.insert((TypeId::of::<TComponent>().into(), readonly));
        self
    }

    pub fn addition_component<TComponent: 'static>(&mut self, readonly: bool) -> &mut Self {
        self.addition_components.insert((TypeId::of::<TComponent>().into(), readonly));
        self
    }

    pub fn except_component<TComponent: 'static>(&mut self) -> &mut Self {
        self.except_components.insert(TypeId::of::<TComponent>().into());
        self
    }

    pub async fn build(self) -> Vec<ArchetypeDataAccessor> {
        self.ecs_data_manager.write().await.archetype_map.iter().filter_map(|(archetype_type, archetype)| {
            if !self.required_components.iter().all(|(component_id, _)| archetype_type.contains(&component_id)) {
                return None;
            }

            if self.except_components.iter().any(|component_id| archetype_type.contains(&component_id)) {
                return None;
            }

            let mut archetype_data_accessor = ArchetypeDataAccessor::default();

            self.required_components.iter().for_each(|(component_id, readonly)| {
                if *readonly {
                    archetype_data_accessor.add_data(component_id.clone(), archetype.get_component_read_lock(&component_id))
                } else {
                    archetype_data_accessor.add_data(component_id.clone(), archetype.get_component_write_lock(&component_id))
                }
            });

            self.addition_components.iter().for_each(|(component_id, readonly)| {
                if !archetype_type.contains(&component_id) {
                    return;
                }

                if *readonly {
                    archetype_data_accessor.add_data(component_id.clone(), archetype.get_component_read_lock(&component_id))
                } else {
                    archetype_data_accessor.add_data(component_id.clone(), archetype.get_component_write_lock(&component_id))
                }
            });
        
            return Some(archetype_data_accessor);
        }).collect::<Vec<_>>()
    }
}
