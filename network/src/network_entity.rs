use crate::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct NetworkEntity(pub u64);

pub struct NetworkEntityRegistry {
    network_entities: HashMap<NetworkEntity, Entity>,
    next_network_entity: NetworkEntity,
}

impl Default for NetworkEntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkEntityRegistry {
    pub fn new() -> Self {
        Self {
            network_entities: HashMap::new(),
            next_network_entity: NetworkEntity(0),
        }
    }

    pub fn get(&self, network_entity: &NetworkEntity) -> Option<&Entity> {
        self.network_entities.get(network_entity)
    }

    pub fn generate_network_entity(&mut self) -> NetworkEntity {
        let entity = self.next_network_entity;
        self.next_network_entity.0 += 1;
        entity
    }

    pub fn insert(
        &mut self,
        network_entity: NetworkEntity,
        entity: Entity,
    ) -> Result<(), crate::Error> {
        if !self.network_entities.contains_key(&network_entity) {
            self.network_entities.insert(network_entity, entity);

            if network_entity.0 >= self.next_network_entity.0 {
                self.next_network_entity.0 = network_entity.0 + 1;
            }

            Ok(())
        } else {
            Err(crate::Error::DuplicateNetworkEntity)
        }
    }
}
