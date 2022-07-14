use std::{collections::HashMap, sync::Arc};

use crate::{types::SceneId, data::EcsDataManager, behavior::EcsBehaviorManager};

#[derive(Debug, Default)]
pub (crate) struct Scene {
    // sync + send
    pub (crate) ecs_data_manager: Arc<tokio::sync::RwLock<EcsDataManager>>,
    // !sync + !send
    pub (crate) ecs_behavior_manager: Arc<std::sync::RwLock<EcsBehaviorManager>>,
}

pub struct World {
    free_scene_id: Vec<SceneId>,
    scenes: HashMap<SceneId, Scene>,
    scene_count: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            free_scene_id: Default::default(),
            scenes: Default::default(),
            scene_count: 0,
        }
    }

    pub fn new_scene(&mut self) -> SceneId {
        let scene_id = self.free_scene_id.pop().unwrap_or_else(|| {
            let new_entity_id = SceneId::new(self.scene_count);
            self.scene_count += 1;
            new_entity_id
        });

        self.scenes.insert(scene_id.clone(), Default::default());

        scene_id
    }

    pub fn get_scene_data(&self, scene_id: &SceneId) -> Option<Arc<tokio::sync::RwLock<EcsDataManager>>> {
        self.scenes.get(scene_id).map(|scene| scene.ecs_data_manager.clone())
    }

    pub fn get_scene_behavior(&self, scene_id: &SceneId) -> Option<Arc<std::sync::RwLock<EcsBehaviorManager>>> {
        self.scenes.get(scene_id).map(|scene| scene.ecs_behavior_manager.clone())
    }
}