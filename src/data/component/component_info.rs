use crate::types::ComponentId;

use crate::data::archetype::{IComponentsArray, ComponentsArray};

use std::fmt::Debug;
use std::sync::Arc;

trait ComponentArrayBuildClosure = Fn(usize) -> Box<dyn IComponentsArray>;

pub struct ComponentInfo {
    pub (crate) component_id: ComponentId,
    pub (crate) component_array_fabric_cloure: Arc<dyn ComponentArrayBuildClosure + Sync + Send>,
    pub (crate) size: usize,
}

impl Debug for ComponentInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentInfo")
            .field("component_id", &self.component_id)
            .field("component_array_fabric_cloure", &"closure").finish()
    }
}

impl ComponentInfo {
    pub fn new<TComponent: Debug + Sync + Send + 'static>() -> Self {
        Self {
            component_id: ComponentId::from_type::<TComponent>(),
            component_array_fabric_cloure: Arc::new(|size: usize| Box::new(ComponentsArray::<TComponent>::new()) as Box<dyn IComponentsArray>),
            size: std::mem::size_of::<TComponent>(),
        }
    }
}