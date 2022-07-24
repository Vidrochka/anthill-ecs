use std::{fmt::Debug, any::TypeId, marker::PhantomData, sync::{Arc}, cell::RefCell};

use tokio::sync::{RwLock, Mutex};

use crate::{data::{EcsDataManager, entity_data_accessor::ChunkDataAccessor}, types::ArchetypeType};

use super::{/* entity_data_accessor::ArchetypeDataAccessor, */ /* job::{Job, JobType} ,*/ query::ArchetypeQuery};

// #[async_trait::async_trait]
// pub trait ISystem: Debug {
//     fn on_create(&mut self) {}

//     fn on_destroy(&mut self) {}

//     fn get_job(&mut self, job: &mut Job, data_manager: ArchetypeDataAccessorBuilder) {}

// } 

// #[derive(Debug, Default)]
// struct System {
// }

// impl ISystem for System {}

#[derive(Debug)]
pub enum SystemType {
    BlockingSystem(Box<dyn IBlockingSystemHandler>),
    MultithreadSystem(Arc<Mutex<dyn IMultithreadSystemHandler + Sync + Send>>),
}

pub trait ISystemInfo {
    fn get_system_name(&self) -> String;
}

pub trait ISystemType {
    fn get_system_type() -> String;
}

impl ISystemType for dyn IBlockingSystemHandler {
    fn get_system_type() -> String {
        "BlockingSystemHandler".to_string()
    }
}

#[async_trait::async_trait(?Send)]
pub trait IBlockingSystemHandler: Debug {
    async fn handle(&mut self, archetype_data_accessor: ChunkDataAccessor);
    fn archetype_query(&self) -> ArchetypeQuery;
}

impl ISystemType for dyn IMultithreadSystemHandler {
    fn get_system_type() -> String {
        "MultithreadSystemHandler".to_string()
    }
}

#[async_trait::async_trait]
pub trait IMultithreadSystemHandler: Debug {
    async fn handle(&mut self, archetype_data_accessor: ChunkDataAccessor);
    fn archetype_query(&self) -> ArchetypeQuery;
}