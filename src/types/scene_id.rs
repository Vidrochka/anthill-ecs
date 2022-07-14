

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub struct SceneId{
    pub(crate) id: u32,
}

impl SceneId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}