use std::collections::HashMap;

use bevy_ecs::system::Resource;
use rand::seq::IteratorRandom;

use crate::{model::ModelHandle, object_types::BlockType};

#[derive(Resource, Clone)]
pub struct ModelManager {
    handle_store: HashMap<BlockType, Vec<ModelHandle>>,
}

impl ModelManager {
    pub fn new(handle_store: HashMap<BlockType, Vec<ModelHandle>>) -> Self {
        Self { handle_store }
    }

    pub fn get_handle(&self, block_type: &BlockType) -> Option<ModelHandle> {
        self.handle_store
            .get(block_type)
            .and_then(|handles|
                handles
                    .iter()
                    .choose(&mut rand::thread_rng())
                    .cloned()
            )
    }
}
