use std::{ops::Deref, any::TypeId};

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub struct ComponentId{
    pub(crate) id: TypeId,
}

impl ComponentId {
    pub fn new(id: TypeId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> TypeId {
        self.id
    }
}

impl Deref for ComponentId {
    type Target = TypeId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl From<TypeId> for ComponentId {
    fn from(type_id: TypeId) -> Self {
        Self { id: type_id }
    }
}