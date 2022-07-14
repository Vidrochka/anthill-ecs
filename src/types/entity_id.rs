use std::ops::Deref;


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct EntityId {
    pub (crate) id: u32,
    pub (crate) version: u32,
}

impl EntityId {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            version: 0,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

impl Deref for EntityId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}