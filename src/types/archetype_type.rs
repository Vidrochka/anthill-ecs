use std::ops::Deref;

use super::ComponentId;

#[derive(Debug, Default, Hash, Clone, PartialEq, Eq)]
pub struct ArchetypeType {
    pub (crate) component_ids: Vec<ComponentId>
}

impl ArchetypeType {
    pub fn empty() -> Self { Self { component_ids: Default::default() } }
    pub fn components_count(&self) -> usize { self.component_ids.len()}
}

impl Deref for ArchetypeType {
    type Target = Vec<ComponentId>;

    fn deref(&self) -> &Self::Target {
        &self.component_ids
    }
}