use crate::*;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct SpawnManager {
    spawnables: HashMap<NetworkEntity, (NetworkTarget, Payload)>,
    connections: HashMap<ConnectionId, HashSet<NetworkEntity>>,
}

impl SpawnManager {
    pub fn new() -> Self {
        Self {
            spawnables: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    pub fn register_spawn(
        &mut self,
        network_entity: NetworkEntity,
        target: NetworkTarget,
        payload: Payload,
    ) {
        self.spawnables.insert(network_entity, (target, payload));
    }

    pub fn confirm_spawn(&mut self, connection_id: ConnectionId, network_entity: NetworkEntity) {
        self.connections
            .entry(connection_id)
            .or_insert(HashSet::new())
            .insert(network_entity);
    }

    pub fn get_not_spawned(
        &self,
        connection_id: ConnectionId,
    ) -> Vec<(NetworkEntity, NetworkTarget, Payload)> {
        let mut not_spawned = Vec::new();

        if let Some(spawned) = self.connections.get(&connection_id) {
            for (network_entity, (target, payload)) in &self.spawnables {
                if !spawned.contains(network_entity) {
                    not_spawned.push((*network_entity, target.clone(), payload.clone()));
                }
            }
        }

        not_spawned
    }
}

#[derive(Bundle)]
pub struct SpawnBundle {
    network_entity: NetworkEntity,
}

pub struct SpawnContext {
    local: Actor,
    sender: Actor,
}

impl SpawnContext {
    pub fn new(local: Actor, sender: Actor) -> Self {
        Self { local, sender }
    }

    pub fn local_id(&self) -> ActorId {
        self.local.id()
    }

    pub fn local_ty(&self) -> ActorTy {
        self.local.ty()
    }

    pub fn sender_id(&self) -> ActorId {
        self.sender.id()
    }

    pub fn sender_ty(&self) -> ActorTy {
        self.sender.ty()
    }
}

#[typetag::serde]
pub trait Spawnable {
    fn spawn(
        &self,
        commands: &mut Commands,
        resources: &Resources,
        ctx: &SpawnContext,
        bundle: SpawnBundle,
    ) -> Entity;
}

#[derive(Default)]
pub struct SpawnSystemEventReader {
    reader: EventReader<Message>,
}

pub fn spawn_system(world: &mut World, resources: &mut Resources) {
    let mut commands = Commands::default();
    commands.set_entity_reserver(world.get_entity_reserver());

    {
        let mut network_entity_registry = resources.get_mut::<NetworkEntityRegistry>().unwrap();
        let mut event_reader = resources.get_mut::<SpawnSystemEventReader>().unwrap();
        let events = resources.get::<Events<Message>>().unwrap();

        for message in event_reader.reader.iter(&events) {
            if let Payload::Spawn {
                network_entity,
                data,
            } = &message.payload
            {
                let spawnable: Box<dyn Spawnable> = serde_cbor::from_slice(data).unwrap();

                let context = SpawnContext::new(message.receiver.clone(), message.sender.clone());

                let bundle = SpawnBundle {
                    network_entity: *network_entity,
                };

                let entity = spawnable.spawn(&mut commands, resources, &context, bundle);

                network_entity_registry
                    .insert(*network_entity, entity)
                    .unwrap();
            }
        }
    }

    commands.apply(world, resources);
}

pub fn spawn_detection_system(
    connection_manager: Res<ConnectionManager>,
    mut spawn_manager: ResMut<SpawnManager>,
    mut network_handle: ResMut<NetworkHandle>,
) {
    // TODO: optimize
    for (connection_id, _) in connection_manager.connections() {
        if !spawn_manager.connections.contains_key(connection_id) {
            spawn_manager
                .connections
                .insert(*connection_id, HashSet::new());
        }
    }

    let SpawnManager {
        spawnables,
        connections,
    } = &mut *spawn_manager;

    for (network_id, (target, payload)) in spawnables {
        for connection_id in connection_manager.get_targeted_connection_ids(target) {
            let actor_id = connection_manager.get_actor(connection_id).unwrap().id();
            let spawned = &mut connections.get_mut(&connection_id).unwrap();

            if !spawned.contains(network_id) {
                info!(
                    "Connection {:?}, doesn't have a copy of {:?}",
                    connection_id, network_id
                );

                spawned.insert(*network_id);
                network_handle.add_payload(NetworkTarget::ActorId(actor_id), payload.clone());
            }
        }
    }
}
