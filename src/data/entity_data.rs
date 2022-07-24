use std::{collections::HashMap, any::Any};

use crate::types::{EntityId, ComponentId, ArchetypeType};



pub struct EntityData {
    pub (crate) entity_id: EntityId,
    pub (crate) entity_components: HashMap<ComponentId, Box<dyn Any + Send + Sync>>
}

impl EntityData {
    #[inline(always)]
    pub (crate) fn new(entity_id: EntityId, entity_components: HashMap<ComponentId, Box<dyn Any + Send + Sync>>) -> Self {
        Self {
            entity_id,
            entity_components,
        }
    }

    #[inline(always)]
    pub (crate) fn add_component_with_type<TComponent: Sync + Send + 'static>(&mut self, component: TComponent) -> &mut Self {
        self.add_component(ComponentId::from_type::<TComponent>(), Box::new(component))
    }

    #[inline(always)]
    pub (crate) fn add_component(&mut self, component_id: ComponentId, component: Box<dyn Any + Send + Sync>) -> &mut Self {
        self.entity_components.insert(component_id, Box::new(component));
        self
    }

    #[inline(always)]
    pub (crate) fn remove_component_with_type<TComponent: Sync + Send + 'static>(&mut self, component: TComponent) -> &mut Self {
        self.remove_component(&ComponentId::from_type::<TComponent>())
    }

    #[inline(always)]
    pub (crate) fn remove_component(&mut self, omponent_id: &ComponentId) -> &mut Self {
        self.entity_components.remove(omponent_id);
        self
    }

    #[inline(always)]
    pub (crate) fn build_archetype_type(&self) -> ArchetypeType {
        self.entity_components.keys().map(|component_id| *component_id).collect::<Vec<_>>().into()
    }
}

