use std::{fmt::Debug, any::TypeId, marker::PhantomData, sync::Arc};

use tokio::sync::RwLock;

use crate::data::{EcsDataManager, entity_data_accessor::ArchetypeDataAccessorBuilder};

use super::{/* entity_data_accessor::ArchetypeDataAccessor, */ job::Job};

// pub struct ComponentInfo {
//     pub (crate) component_type_id: TypeId,
//     pub (crate) readonly: bool, 
// }

// impl ComponentInfo {
//     pub fn new<TComponent: 'static>(readonly: bool) -> Self {
//         Self {
//             component_type_id: TypeId::of::<TComponent>(),
//             readonly
//         }
//     }
// }

#[async_trait::async_trait]
pub trait ISystem: Debug {
    // if any system not found, skip system
    //const REQUIRED_SYSTEMS: &'a [TypeId] = &[];
    // fn required_systems<'a>(&mut self) -> &'a [TypeId] {
    //     &[]
    // }
    // run after systems
    // fn after_systems<'a>(&mut self) -> &'a [TypeId] {
    //     &[]
    // }

    // fn required_components<'a>(&mut self) -> &'a [ComponentInfo] {
    //     &[]
    // }
    // fn addition_components<'a>(&mut self) -> &'a [ComponentInfo] {
    //     &[]
    // }
    // fn except_components<'a>(&mut self) -> &'a [TypeId] {
    //     &[]
    // }

    fn on_create(&mut self) {}

    fn on_destroy(&mut self) {}

    fn get_job(&mut self, job: &mut Job, data_manager: ArchetypeDataAccessorBuilder) {}

} 

// pub trait IMainThreadJobSystem: Debug {
//     // fn after_systems<'a>(&mut self) -> &'a [TypeId] {
//     //     &[]
//     // }

//     fn on_create(&mut self) {}

//     fn on_destroy(&mut self) {}

//     fn on_update(&mut self) {}
// }

// impl ISystem for dyn IMainThreadJobSystem {
//     // fn after_systems<'a>(&mut self) -> &'a [TypeId] {
//     //     <Self as IMainThreadJobSystem>::after_systems(self)
//     // }

//     fn on_create<'a>(&mut self) {
//         <Self as IMainThreadJobSystem>::on_create(self)
//     }

//     fn on_destroy<'a>(&mut self) {
//         <Self as IMainThreadJobSystem>::on_destroy(self)
//     }

//     fn get_job(&mut self, job: &mut Job) {
//         job.set_main_thread_job(Box::new(|| self.on_update()))
//     }
// } 

// #[async_trait::async_trait]
// pub trait IMultiThreadJobSystem : Debug where Self: Send{
//     // fn after_systems<'a>(&mut self) -> &'a [TypeId] {
//     //     &[]
//     // }

//     fn on_create(&mut self) {}

//     fn on_destroy(&mut self) {}

//     async fn on_update(&mut self/* , archetype_data_accessor: ArchetypeDataAccessor */) {}
// }

// impl ISystem for dyn IMultiThreadJobSystem {
//     // fn after_systems<'a>(&mut self) -> &'a [TypeId] {
//     //     <Self as IMultiThreadJobSystem>::after_systems(self)
//     // }

//     fn on_create<'a>(&mut self) {
//         <Self as IMultiThreadJobSystem>::on_create(self)
//     }

//     fn on_destroy<'a>(&mut self) {
//         <Self as IMultiThreadJobSystem>::on_destroy(self)
//     }

//     fn get_job(&mut self, job: &mut Job) {
//         job.set_multi_thread_job(Box::pin(async { self.on_update().await }))
//     }
// } 

#[derive(Debug, Default)]
struct System {
}

impl ISystem for System {}