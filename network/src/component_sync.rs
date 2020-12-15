use crate::*;
use bevy::{prelude::*, reflect::TypeRegistry};

pub struct ComponentSync<T: SyncableComponent> {
    should_sync: bool,
    ownership: NetworkTarget,
    phantom_data: std::marker::PhantomData<T>,
}

impl<T: SyncableComponent> ComponentSync<T> {
    pub fn new(network_target: NetworkTarget) -> Self {
        Self {
            should_sync: true,
            ownership: network_target,
            phantom_data: Default::default(),
        }
    }

    pub fn id(actor_id: ActorId) -> Self {
        Self::new(NetworkTarget::ActorId(actor_id))
    }

    pub fn ty(actor_ty: ActorTy) -> Self {
        Self::new(NetworkTarget::ActorTy(actor_ty))
    }

    pub fn all() -> Self {
        Self::new(NetworkTarget::All)
    }

    pub fn sync(&mut self) {
        self.should_sync = true;
    }
}

pub fn component_sync_receiving_system<T: SyncableComponent + Send + Sync + 'static>(
    network_entity_registry: Res<NetworkEntityRegistry>,
    mut event_reader: Local<EventReader<Message>>,
    events: Res<Events<Message>>,
    type_registry: Res<TypeRegistry>,
    mut query: Query<(&mut T, &ComponentSync<T>), With<NetworkEntity>>,
) {
    for message in event_reader.iter(&events) {
        if let Payload::ComponentUpdate {
            target_entity,
            network_type_uuid,
            data,
        } = &message.payload
        {
            if *network_type_uuid != T::UUID {
                continue;
            }

            let entity = if let Some(entity) = network_entity_registry.get(target_entity) {
                entity
            } else {
                error!("Could not find entity! {:?}", target_entity);
                continue;
            };

            if let Ok((mut component, component_sync)) = query.get_mut(*entity) {
                if !message.sender.targeted_by(&component_sync.ownership) {
                    error!(
                        "Asked to update component, by invalid sender {:?}!",
                        message.sender
                    );
                    continue;
                }

                *component = T::from_bytes(data, &*type_registry);
            }
        }
    }
}

pub fn component_sync_connect_system<T: SyncableComponent + Send + Sync + 'static>(
    mut event_reader: Local<EventReader<ConnectionEvent>>,
    events: Res<Events<ConnectionEvent>>,
    mut query: Query<&mut ComponentSync<T>>,
) {
    for connection_event in event_reader.iter(&events) {
        if let ConnectionEvent::Connected { .. } = connection_event {
            for mut component_sync in query.iter_mut() {
                component_sync.sync();
            }
        }
    }
}

pub fn component_sync_marking_system<T: SyncableComponent + Send + Sync + 'static>(
    mut query: Query<&mut ComponentSync<T>, Changed<T>>,
) {
    for mut component_sync in query.iter_mut() {
        component_sync.sync();
    }
}

pub fn component_sync_sending_system<T: SyncableComponent + Send + Sync + 'static>(
    mut network_handle: ResMut<NetworkHandle>,
    type_registry: Res<TypeRegistry>,
    connection_manager: Res<ConnectionManager>,
    network_settings: Res<NetworkSettings>,
    mut query: Query<(&T, &mut ComponentSync<T>, &NetworkEntity)>,
) {
    for (component, mut component_sync, network_entity) in query.iter_mut() {
        if !component_sync.should_sync {
            continue;
        }

        component_sync.should_sync = false;

        if let Some(actor) = connection_manager.get_local_actor() {
            if !actor.targeted_by(&component_sync.ownership) {
                continue;
            }

            let bytes = component.to_bytes(&*type_registry);

            for target in &network_settings.sync_components_with {
                network_handle.sync_component(
                    target.clone(),
                    *network_entity,
                    T::UUID,
                    bytes.clone(),
                );
            }
        } else {
            error!("Local actor not found!");
        }
    }
}
