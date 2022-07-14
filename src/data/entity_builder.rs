use std::{any::{Any, TypeId}, fmt::Debug};

use crate::types::{ArchetypeType, ComponentId, BuildEntityResult, EntityId, BuildEntityError};

use super::{EcsDataManager, archetype::{IComponentsArray, ComponentsArray, Archetype}};

pub struct EntityBuilder<'a, const COMPONENTS_COUNT: usize> {
    expected_archetype_type: ArchetypeType,
    actual_archetype_type: ArchetypeType,
    ecs_data_manager: &'a mut EcsDataManager,
    archetype_components_collections: Option<Vec<Box<dyn IComponentsArray>>>,
    components: Vec<Box<dyn Any + Sync + Send>>
}

impl<'a, const COMPONENTS_COUNT: usize> EntityBuilder<'a, COMPONENTS_COUNT> {
    pub (crate) fn new(expected_archetype: [ComponentId; COMPONENTS_COUNT], ecs_data_manager: &'a mut EcsDataManager) -> Self {
        let expected_archetype_type = ArchetypeType { component_ids: Vec::from(expected_archetype) };
        let archetype_components_collections = if ecs_data_manager.archetype_map.contains_key(&expected_archetype_type) { None } else { Some(Vec::new()) };

        Self {
            expected_archetype_type,
            actual_archetype_type: ArchetypeType::empty(),
            ecs_data_manager,
            archetype_components_collections,
            components: Vec::new(),
        }
    }

    pub fn add_component<TComponent: Clone + Debug + Sync + Send + 'static>(&mut self, component: TComponent) -> &mut Self {
        self.components.push(Box::new(component));
        let component_id = self.ecs_data_manager.component_type_id_to_id_mapping[&TypeId::of::<TComponent>()];
        self.actual_archetype_type.component_ids.push(component_id);
        self.archetype_components_collections.as_mut().map(|archetype_components| archetype_components.push(Box::new(ComponentsArray::<TComponent>::new())));
        self
    }

    pub fn add_default_component<TComponent: Default + Clone + Debug + Sync + Send + 'static>(&mut self) -> &mut Self {
        self.components.push(Box::new(TComponent::default()));
        let component_id = self.ecs_data_manager.component_type_id_to_id_mapping[&TypeId::of::<TComponent>()];
        self.actual_archetype_type.component_ids.push(component_id);
        self.archetype_components_collections.as_mut().map(|archetype_components| archetype_components.push(Box::new(ComponentsArray::<TComponent>::new())));
        self
    }

    pub fn build(self) -> BuildEntityResult<EntityId> {
        if self.expected_archetype_type != self.actual_archetype_type {
            return Err(BuildEntityError::MismatchedArchetype { expected: self.expected_archetype_type, actual: self.actual_archetype_type })
        }

        if let Some(archetype_components_collections) = self.archetype_components_collections {
            self.ecs_data_manager.archetype_map.insert(self.expected_archetype_type, Archetype::new(self.actual_archetype_type.clone(), archetype_components_collections));
        }

        let entity_id = self.ecs_data_manager.free_entity_id.pop().unwrap_or_else(|| {
            let new_entity_id = EntityId::new(self.ecs_data_manager.index_count);
            self.ecs_data_manager.index_count += 1;
            new_entity_id
        });

        self.ecs_data_manager.archetype_map.get_mut(&self.actual_archetype_type).unwrap().add_entity(entity_id.clone(), self.components);
        

        Ok(entity_id)
    }
}