// use std::ops::Deref;

// #[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
// pub struct SystemId{
//     pub(crate) id: u32,
// }

// impl SystemId {
//     pub fn new(id: u32) -> Self {
//         Self { id }
//     }

//     pub fn id(&self) -> u32 {
//         self.id
//     }
// }

// impl Deref for SystemId {
//     type Target = u32;

//     fn deref(&self) -> &Self::Target {
//         &self.id
//     }
// }