use std::{any::{TypeId, type_name}, collections::{HashMap, LinkedList, VecDeque, HashSet}, sync::Arc};

// use crate::types::SystemId;

use crossbeam::channel::bounded;
use tokio::{runtime::Handle, task::JoinHandle, sync::{Mutex, RwLock}};

// use crate::{types::{AddSystemResult, AddSystemError, ComponentId, ArchetypeType, BuildSystemResult, BuildSystemError}, data::{EcsDataManager, entity_data_accessor::ArchetypeDataAccessorBuilder}};

use crate::{data::{EcsDataManager, entity_data_accessor::ChunkDataAccessor}, types::{ComponentId, BuildSystemError, BuildSystemResult}};

use self::{system::{SystemType, IBlockingSystemHandler, IMultithreadSystemHandler}/* , job::Job */};

pub mod system;
pub mod job;
pub mod query;

#[derive(Debug)]
pub struct SystemInfo {
    system_type_id: TypeId,
    system: SystemType,
    disabled: bool,
}

impl SystemInfo {
    pub fn new_blocking<TSystem: IBlockingSystemHandler + 'static>(system: TSystem) -> Self {
        Self {
            system_type_id: TypeId::of::<TSystem>(),
            system: SystemType::BlockingSystem(Box::new(system)),
            disabled: true,
        }
    }

    pub fn new_multithread<TSystem: IMultithreadSystemHandler + Sync + Send + 'static>(system: TSystem) -> Self {
        Self {
            system_type_id: TypeId::of::<TSystem>(),
            system: SystemType::MultithreadSystem(Arc::new(Mutex::new(system))),
            disabled: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct EcsBehaviorManager {
    systems_info: HashMap<TypeId, SystemInfo>,
    next_systems_links: HashMap<TypeId, HashSet<TypeId>>,
    prev_systems_links: HashMap<TypeId, HashSet<TypeId>>,
    pure_systems: HashSet<TypeId>,
}

// add cycle ref check
impl EcsBehaviorManager {
    pub fn check_disabled_systems(&mut self) {
        let disabled_systems_type_id = self.systems_info.keys()
            .filter_map(|system_type_id| {
                if let Some(prev_systems_links) = self.prev_systems_links.get(&system_type_id) {
                    if prev_systems_links.iter().all(|prev_system_link| self.systems_info.contains_key(prev_system_link)) {
                        return None;
                    }
                }

                Some(*system_type_id)
            }).collect::<Vec<_>>();

        self.systems_info.values_mut().for_each(|system_info| {
            if disabled_systems_type_id.contains(&system_info.system_type_id) {
                system_info.disabled = true;
            } else {
                system_info.disabled = false;
            }
        })
    }

    pub fn remove_system<TSystem: 'static>(&mut self) {
        _ = self.systems_info.remove(&TypeId::of::<TSystem>());
        _ = self.pure_systems.drain_filter(|item| *item == TypeId::of::<TSystem>());
        self.check_disabled_systems();
    }

    pub fn get_system_builder<'a>(&'a mut self) -> Option<SystemBuilder<'a>> {
        Some(SystemBuilder::<'a>::new(self))
    }

    /// циклически проверяет встречается ли предшествующих систем зависимые системы от текущей системы, если да, вернет false - циклическая зависимость 
    pub fn check_links(&mut self, check_system_prev_systems: &HashSet<TypeId>, check_system_next_systems: &HashSet<TypeId>) -> bool {    
        let mut system_prev_systems = check_system_prev_systems.iter().collect::<HashSet<&TypeId>>();

        while !system_prev_systems.is_empty() {
            for system_prev_system in system_prev_systems.iter() {
                if check_system_next_systems.contains(*system_prev_system) {
                    return false
                }
            }

            system_prev_systems = system_prev_systems.into_iter().filter_map(|system_prev_system| -> _ {
                    self.prev_systems_links.get(&system_prev_system)
                })
                .flat_map(|system_prev_system| system_prev_system.iter())
                .collect::<HashSet<_>>();
        }

        return true;
    }

    pub fn update(&mut self, ecs_data_manager: Arc<RwLock<EcsDataManager>>, rt_handle: Handle) {
        if self.pure_systems.is_empty() {
            return;
        }

        let ecs_data_manager_write_lock = ecs_data_manager.blocking_write(); 

        let mut system_requirements = self.prev_systems_links.clone();

        let (end_job_sender, end_job_receiver) = bounded::<TypeId>(1);

        let mut join_handlers = Vec::with_capacity(self.systems_info.len());

        let mut job_in_process_count = 0;

        let mut components_used_as_read = HashSet::<ComponentId>::new();
        let mut components_used_as_write = HashSet::<ComponentId>::new();
        let mut pending_services = HashMap::<ComponentId, Vec<TypeId>>::new();

        self.pure_systems.iter().for_each(|pure_system_type_id| {
            let system_info = self.systems_info.get_mut(pure_system_type_id).unwrap();
            
            match &mut system_info.system {
                SystemType::BlockingSystem(system) => {
                    let query = system.archetype_query();

                    ecs_data_manager_write_lock.archetype_map.values().for_each(|archetype| {
                        if query.is_archetype_match(archetype.archetype_type()) {
                            let archetype_data_accessor = ChunkDataAccessor::default();
                            
                            query.required

                            archetype.get_chunks().for_each(|chunk| {
                                archetype_data_accessor.fill_data_from_chunk(, chunk)
                            });
                            
                        }
                    });

                    rt_handle.block_on(async {
                        system.handle().await;
                    })
                },
                SystemType::MultithreadSystem(system) => {
                    let system_end_sender = end_job_sender.clone();
                    let system_type_id = *pure_system_type_id;

                    let system = system.clone();

                    let join_handler = rt_handle.spawn(async move {
                        system.lock().await.handle().await;
    
                        system_end_sender.send(system_type_id).unwrap();
                    });

                    join_handlers.push(join_handler);
                
                    job_in_process_count += 1;
                },
            }
        });

        while job_in_process_count != 0 {
            job_in_process_count -= 1;
            let finished_system_type_id = end_job_receiver.recv().unwrap();

            let finished_system_info = self.systems_info.get(&finished_system_type_id).unwrap();

            let systems_ready_to_start_opt = self.next_systems_links.get(&finished_system_type_id).map(|next_systems| {
                next_systems.iter().filter_map(|next_system| {
                    if let Some(next_system_requirements) = system_requirements.get_mut(next_system) {
                        next_system_requirements.remove(&finished_system_type_id);
    
                        if next_system_requirements.is_empty() {
                            return Some(*next_system);
                        }
                    }
    
                    return None;
                }).collect::<Vec<_>>()
            });
            
            systems_ready_to_start_opt.map(|systems_ready_to_start| {
                systems_ready_to_start.iter().for_each(|next_system| {
                    let system_info = self.systems_info.get_mut(&next_system).unwrap();
    
                    match &mut system_info.system {
                        SystemType::BlockingSystem(system) => {
                            rt_handle.block_on(async {
                                system.handle().await;
                            })
                        },
                        SystemType::MultithreadSystem(system) => {
                            let system_end_sender = end_job_sender.clone();
                            let system_type_id = *next_system;
        
                            let system = system.clone();

                            let join_handler = rt_handle.spawn(async move {
                                system.lock().await.handle().await;
            
                                system_end_sender.send(system_type_id).unwrap();
                            });
        
                            join_handlers.push(join_handler);
                        
                            job_in_process_count += 1;
                        },
                    }
                })
            });
        }

        rt_handle.block_on(async move {
            for system_job in join_handlers.into_iter() {
                system_job.await.unwrap();
            }
        });
    }
}

pub struct SystemBuilder<'a> {
    ecs_behavior_manager: &'a mut EcsBehaviorManager,

    prev_systems: HashSet<TypeId>,
    next_systems: HashSet<TypeId>,
}

impl<'a> SystemBuilder<'a> {
    pub (crate) fn new(ecs_behavior_manager: &'a mut EcsBehaviorManager) -> Self {
        Self {
            ecs_behavior_manager,
            prev_systems: Default::default(),
            next_systems: Default::default(),
        }
    }

    pub fn need_system_result<TPrevSystem: 'static>(&mut self) -> &mut Self {
        self.prev_systems.insert(TypeId::of::<TPrevSystem>());
        self
    }

    pub fn result_for_system<TNextSystem: 'static>(&mut self) -> &mut Self {
        self.prev_systems.insert(TypeId::of::<TNextSystem>());
        self
    }

    pub fn build_with_sync_handler<TSystem: IBlockingSystemHandler + 'static>(self, system: TSystem) -> BuildSystemResult<()> {
        if self.ecs_behavior_manager.check_links(&self.prev_systems, &self.next_systems) {
            return Err(BuildSystemError::CycledSystemLinks { system_name: type_name::<TSystem>().to_string(), system_type_id: TypeId::of::<TSystem>() })
        }

        if self.prev_systems.is_empty() {
            self.ecs_behavior_manager.pure_systems.insert(TypeId::of::<TSystem>());
        }

        self.prev_systems.iter().for_each(|prev_system| {
            self.ecs_behavior_manager.next_systems_links.entry(*prev_system)
                .or_default()
                .insert(TypeId::of::<TSystem>());

            self.ecs_behavior_manager.prev_systems_links.entry(TypeId::of::<TSystem>())
                .or_default()
                .insert(*prev_system);
        });

        self.next_systems.iter().for_each(|next_system| {
            // если система зависима от резульатат новой системы, то система уже не "чистая"
            self.ecs_behavior_manager.pure_systems.remove(next_system);

            self.ecs_behavior_manager.next_systems_links.entry(TypeId::of::<TSystem>())
                .or_default()
                .insert(*next_system);

            self.ecs_behavior_manager.prev_systems_links.entry(*next_system)
                .or_default()
                .insert(TypeId::of::<TSystem>());
        });
        
        self.ecs_behavior_manager.systems_info.insert(
            TypeId::of::<TSystem>(),
            SystemInfo::new_blocking::<TSystem>(system)
        );

        self.ecs_behavior_manager.check_disabled_systems();

        Ok(())
    }

    pub fn build_with_muitithread_handler<TSystem: IMultithreadSystemHandler + Sync + Send + 'static>(self, system: TSystem) -> BuildSystemResult<()> {
        if self.ecs_behavior_manager.check_links(&self.prev_systems, &self.next_systems) {
            return Err(BuildSystemError::CycledSystemLinks { system_name: type_name::<TSystem>().to_string(), system_type_id: TypeId::of::<TSystem>() })
        }

        if self.prev_systems.is_empty() {
            self.ecs_behavior_manager.pure_systems.insert(TypeId::of::<TSystem>());
        }

        self.prev_systems.iter().for_each(|prev_system| {
            self.ecs_behavior_manager.next_systems_links.entry(*prev_system)
                .or_default()
                .insert(TypeId::of::<TSystem>());

            self.ecs_behavior_manager.prev_systems_links.entry(TypeId::of::<TSystem>())
                .or_default()
                .insert(*prev_system);
        });

        self.next_systems.iter().for_each(|next_system| {
            // если система зависима от резульатат новой системы, то система уже не "чистая"
            self.ecs_behavior_manager.pure_systems.remove(next_system);

            self.ecs_behavior_manager.next_systems_links.entry(TypeId::of::<TSystem>())
                .or_default()
                .insert(*next_system);

            self.ecs_behavior_manager.prev_systems_links.entry(*next_system)
                .or_default()
                .insert(TypeId::of::<TSystem>());
        });
        
        self.ecs_behavior_manager.systems_info.insert(
            TypeId::of::<TSystem>(),
            SystemInfo::new_multithread::<TSystem>(system)
        );

        self.ecs_behavior_manager.check_disabled_systems();

        Ok(())
    }
}
