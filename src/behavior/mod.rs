use std::{any::{TypeId, type_name}, collections::{HashMap, LinkedList, VecDeque, HashSet}, sync::Arc};

// use crate::types::SystemId;

use crossbeam::channel::bounded;
use tokio::{runtime::Handle, task::JoinHandle, sync::{Mutex, RwLock}};

use crate::{types::{AddSystemResult, AddSystemError}, data::{EcsDataManager, entity_data_accessor::ArchetypeDataAccessorBuilder}};

use self::{system::ISystem, job::Job};

pub mod system;
pub mod job;

#[derive(Debug)]
pub struct SystemInfo {
    system_type_id: TypeId,
    system: Box<dyn ISystem>,

    prev_systems: HashSet<TypeId>,
    next_systems: HashSet<TypeId>,

    disabled: bool,
}

impl SystemInfo {
    pub fn new<TSystem: ISystem + 'static>(system: TSystem, prev_systems: HashSet<TypeId>, next_systems: HashSet<TypeId>) -> Self {
        Self {
            system_type_id: TypeId::of::<TSystem>(),
            system: Box::new(system),
            prev_systems: prev_systems,
            next_systems: next_systems,
            disabled: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct EcsBehaviorManager {
    systems_info: HashMap<TypeId, SystemInfo>,
    pure_systems: HashSet<TypeId>,
}

// add cycle ref check
impl EcsBehaviorManager {
    pub fn check_disabled_systems(&mut self) {
        let disabled_systems_type_id = self.systems_info.values()
            .filter_map(|system_info| {
                if system_info.prev_systems.iter().all(|prev_system| self.systems_info.contains_key(&prev_system)) {
                    None
                } else {
                    Some(system_info.system_type_id)
                }
            }).collect::<Vec<_>>();

        self.systems_info.values_mut().for_each(|system_info| {
            if disabled_systems_type_id.contains(&system_info.system_type_id) {
                system_info.disabled = true;
            } else {
                system_info.disabled = false;
            }
        })
    }

    pub fn remove_system<TSystem: ISystem + 'static>(&mut self) {
        _ = self.systems_info.remove(&TypeId::of::<TSystem>());
        _ = self.pure_systems.drain_filter(|item| *item == TypeId::of::<TSystem>());
        self.check_disabled_systems();
    }

    pub fn get_system_builder<'a, TSystem: ISystem + 'static>(&'a mut self, system: TSystem) -> Option<SystemBuilder<'a, TSystem>> {
        if self.systems_info.contains_key(&TypeId::of::<TSystem>()) {
            return None;
        }

        Some(SystemBuilder::<'a, TSystem>::new(self, system))
    }

    pub fn update(&mut self, ecs_data_manager: Arc<RwLock<EcsDataManager>>, rt_handle: Handle) {
        if self.pure_systems.is_empty() {
            return;
        }

        let mut system_requirements = self.systems_info.values().map(|system_info| {
            (system_info.system_type_id, system_info.prev_systems.clone())
        }).collect::<HashMap<_,_>>();

        let (end_job_sender, end_job_receiver) = bounded::<TypeId>(1);

        let mut join_handlers = Vec::with_capacity(self.systems_info.len());

        let mut job_in_process_count = 0;

        self.pure_systems.iter().for_each(|pure_system_type_id| {
            let system_info = self.systems_info.get_mut(pure_system_type_id).unwrap();
            let mut job = Job::default();
            system_info.system.get_job(&mut job, ArchetypeDataAccessorBuilder::new(ecs_data_manager.clone()));

            if !job.multi_thread_job.is_empty() {
                let system_end_sender = end_job_sender.clone();

                let system_type_id_response = system_info.system_type_id;
                let join_handler = rt_handle.spawn(async move {
                    let system_job_parts = job.multi_thread_job.into_iter().map(|async_job_part| {
                        tokio::spawn(async_job_part)
                    });

                    for job_part in system_job_parts {
                        job_part.await.unwrap();
                    }

                    system_end_sender.send(system_type_id_response).unwrap();
                });

                join_handlers.push(join_handler);
                
                job_in_process_count += 1;
            }

            job.main_thread_job.into_iter().for_each(|main_thread_job_part| {
                rt_handle.block_on(async {
                    main_thread_job_part()
                })
            });
        });

        while job_in_process_count != 0 {
            job_in_process_count -= 1;
            let finished_system_type_id = end_job_receiver.recv().unwrap();

            let finished_system_info = self.systems_info.get(&finished_system_type_id).unwrap();

            let systems_ready_to_start = finished_system_info.next_systems.iter().filter_map(|next_system| {
                if let Some(next_system_requirements) = system_requirements.get_mut(&next_system) {
                    next_system_requirements.remove(&finished_system_type_id);

                    if next_system_requirements.is_empty() {
                        return Some(*next_system);
                    }
                }

                return None;
            }).collect::<Vec<_>>();
            
            systems_ready_to_start.iter().for_each(|next_system| {
                let system_info = self.systems_info.get_mut(&next_system).unwrap();
                let mut job = Job::default();
                system_info.system.get_job(&mut job, ArchetypeDataAccessorBuilder::new(ecs_data_manager.clone()));

                if !job.multi_thread_job.is_empty() {
                    let system_end_sender = end_job_sender.clone();

                    let system_type_id_response = system_info.system_type_id;
                    let join_handler = rt_handle.spawn(async move {
                        let system_job_parts = job.multi_thread_job.into_iter().map(|async_job_part| {
                            tokio::spawn(async_job_part)
                        });

                        for job_part in system_job_parts {
                            job_part.await.unwrap();
                        }

                        system_end_sender.send(system_type_id_response).unwrap();
                    });

                    join_handlers.push(join_handler);
                            
                    job_in_process_count += 1;
                }

                job.main_thread_job.into_iter().for_each(|main_thread_job_part| {
                    rt_handle.block_on(async {
                        main_thread_job_part()
                    })
                });
            });
        }

        rt_handle.block_on(async move {
            for system_job in join_handlers.into_iter() {
                system_job.await.unwrap();
            }
        });
    }
}

pub struct SystemBuilder<'a, TSystem: ISystem + 'static> {
    ecs_behavior_manager: &'a mut EcsBehaviorManager,

    system: TSystem,
    system_type_id: TypeId,

    prev_systems: HashSet<TypeId>,
    next_systems: HashSet<TypeId>,
}

impl<'a, TSystem: ISystem + 'static> SystemBuilder<'a, TSystem> {
    pub (crate) fn new(ecs_behavior_manager: &'a mut EcsBehaviorManager, system: TSystem) -> Self {
        Self {
            ecs_behavior_manager,
            system,
            system_type_id: TypeId::of::<TSystem>(),
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

    pub fn build(self) {
        if self.prev_systems.is_empty() {
            self.ecs_behavior_manager.pure_systems.insert(self.system_type_id);
        }

        self.prev_systems.iter().for_each(|prev_system| {
            self.ecs_behavior_manager.systems_info.get_mut(prev_system).map(|prev_system_info| {
                prev_system_info.next_systems.insert(self.system_type_id);
            });
        });

        self.next_systems.iter().for_each(|next_system| {
            self.ecs_behavior_manager.systems_info.get_mut(next_system).map(|next_system_info| {
                next_system_info.prev_systems.insert(self.system_type_id);
            });
        });
        
        self.ecs_behavior_manager.systems_info.insert(
            self.system_type_id,
            SystemInfo::new::<TSystem>(self.system, self.prev_systems, self.next_systems)
        );

        self.ecs_behavior_manager.check_disabled_systems();
    }
}
