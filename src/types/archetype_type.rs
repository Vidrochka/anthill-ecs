use std::{ops::Deref, collections::HashMap};

use crate::data::component::component_info::ComponentInfo;

use super::ComponentId;

#[derive(Debug, Default, Hash, Clone, PartialEq, Eq)]
pub struct ArchetypeType {
    pub (crate) component_ids: Vec<ComponentId>
}

impl ArchetypeType {
    pub fn empty() -> Self { Self { component_ids: Default::default() } }
    pub fn components_count(&self) -> usize { self.component_ids.len()}
    pub fn contain<TComponent>(&self, component_id: ComponentId) -> bool { self.component_ids.contains(&component_id) }
    pub (crate) fn check(&self, registered_components_map: &HashMap<ComponentId, ComponentInfo>) -> Option<ComponentId> {
        self.component_ids.iter()
            .find(|x| !registered_components_map.contains_key(*x))
            .map(|x| *x)
    }
}

impl Deref for ArchetypeType {
    type Target = Vec<ComponentId>;

    fn deref(&self) -> &Self::Target {
        &self.component_ids
    }
}

impl From<Vec<ComponentId>> for ArchetypeType {
    fn from(component_ids: Vec<ComponentId>) -> Self {
        Self { component_ids: component_ids }
    }
}