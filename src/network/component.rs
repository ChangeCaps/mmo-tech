use super::*;
use bevy::{
    prelude::*,
    reflect::{TypeUuid, Uuid},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

pub struct Owner {
    connection: ConnectionId,
    connection_type: ConnectionType,
}

pub struct Owners {
    owned: bool,
    connections: HashSet<ConnectionId>,
    connection_types: HashSet<ConnectionType>,
}

impl Owners {
    pub fn new(owned: bool) -> Self {
        Self {
            connections: HashSet::new(),
            connection_types: HashSet::new(),
            owned,
        }
    }

    pub fn owned_by(&self, owner: &Owner) -> bool {
        self.connections.contains(&owner.connection)
            || self.connection_types.contains(&owner.connection_type)
    }

    pub fn owned(&self) -> bool {
        self.owned
    }

    pub fn add_connection_type<T: TypeUuid>(&mut self) {
        self.connection_types.insert(ConnectionType::from::<T>());
    }

    pub fn add_connection(&mut self, connection: ConnectionId) {
        self.connections.insert(connection);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkEntityId(u64);

pub struct NetworkEntityRegistry {
    network_entities: HashMap<NetworkEntityId, Entity>,
}

impl NetworkEntityRegistry {
    pub fn get(&self, key: &NetworkEntityId) -> Option<&Entity> {
        self.network_entities.get(key)
    }

    pub fn insert(&mut self, key: NetworkEntityId, value: Entity) {
        self.network_entities.insert(key, value);
    }
}

pub struct ComponentSync<T: TypeUuid> {
    owners: Owners,
    phantom_data: std::marker::PhantomData<T>,
}

pub struct ComponentUpdate {
    target_entity: NetworkEntityId,
    data: Vec<u8>,
}

pub struct ComponentUpdates {
    updates: HashMap<Uuid, Arc<Mutex<Vec<ComponentUpdate>>>>,
}

pub fn network_component_sync_system<T>(
    component_update: Res<ComponentUpdates>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    mut network_handle: ResMut<NetworkHandle>,
    mut query: Query<&mut T, With<ComponentSync<T>>>,
    mutated: Query<(&T, &ComponentSync<T>), Mutated<T>>,
) where
    T: TypeUuid + DeserializeOwned + Serialize + Send + Sync + 'static + Clone + Component,
    ComponentSync<T>: Component,
{
    for (component, component_sync) in mutated.iter() {
        if component_sync.owners.owned() {
            network_handle.update_component(component);
        }
    }

    if let Some(updates) = component_update.updates.get(&T::TYPE_UUID) {
        let mut updates = updates.lock().unwrap();

        for update in updates.drain(..) {
            if let Some(entity) = network_entity_registry.get(&update.target_entity) {
                if let Ok(mut component) = query.get_mut(*entity) {
                    let new_component: T = bincode::deserialize(&update.data).unwrap();

                    *component = new_component;
                }
            } else {
                log::warn!("entity not found");
            }
        }
    }
}
