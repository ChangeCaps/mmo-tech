use crate::*;
use bevy::reflect::Uuid;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Payload {
    ComponentUpdate {
        target_entity: NetworkEntity,
        network_type_uuid: Uuid,
        data: Vec<u8>,
    },
    Spawn {
        network_entity: NetworkEntity,
        data: Vec<u8>,
    },
}

#[derive(Clone, Debug)]
pub struct Message {
    pub payload: Payload,
    pub sender: Actor,
    pub receiver: Actor,
}

#[derive(Clone, Debug)]
pub enum NetworkTarget {
    ActorId(ActorId),
    ActorTy(ActorTy),
    All,
}

#[derive(Default)]
pub struct NetworkHandle {
    payloads: Vec<(NetworkTarget, Payload)>,
    spawn_messages: Vec<(NetworkTarget, Vec<u8>)>,
}

impl NetworkHandle {
    pub fn new() -> Self {
        Self {
            payloads: Vec::new(),
            spawn_messages: Vec::new(),
        }
    }

    pub fn spawn<T: Spawnable>(&mut self, target: NetworkTarget, spawnable: T) {
        let spawnable: Box<dyn Spawnable> = Box::new(spawnable);
        let data = serde_cbor::to_vec(&spawnable).unwrap();
        self.spawn_messages.push((target, data));
    }

    pub fn sync_component(
        &mut self,
        target: NetworkTarget,
        target_entity: NetworkEntity,
        network_type_uuid: Uuid,
        data: Vec<u8>,
    ) {
        self.payloads.push((
            target,
            Payload::ComponentUpdate {
                target_entity,
                network_type_uuid,
                data,
            },
        ));
    }

    pub fn convert_spawn_messages(
        &mut self,
        network_entity_registry: &mut NetworkEntityRegistry,
        spawn_manager: &mut SpawnManager,
        connection_manager: &ConnectionManager,
    ) {
        for (target, data) in std::mem::replace(&mut self.spawn_messages, Vec::new()) {
            let network_entity = network_entity_registry.generate_network_entity();

            let payload = Payload::Spawn {
                network_entity,
                data,
            };

            spawn_manager.register_spawn(network_entity, target.clone(), payload.clone());

            for connection_id in connection_manager.get_targeted_connection_ids(&target) {
                spawn_manager.confirm_spawn(connection_id, network_entity);
            }

            self.add_payload(target, payload);
        }
    }

    pub fn add_payload(&mut self, target: NetworkTarget, payload: Payload) {
        self.payloads.push((target, payload));
    }

    pub fn clear_payloads(&mut self) -> Vec<(NetworkTarget, Payload)> {
        std::mem::replace(&mut self.payloads, Vec::new())
    }
}
