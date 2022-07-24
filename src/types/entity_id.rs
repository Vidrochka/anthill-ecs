use std::ops::Deref;


#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct EntityId {
    pub (crate) id: usize,
    pub (crate) version: usize,
}

impl EntityId {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            version: 0,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn version(&self) -> usize {
        self.version
    }
}

impl Deref for EntityId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}