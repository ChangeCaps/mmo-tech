use crate::*;
use bevy::prelude::*;

pub fn receiving_system(
    mut connection_manager: ResMut<ConnectionManager>,
    mut message_events: ResMut<Events<Message>>,
    mut connection_events_resource: ResMut<Events<ConnectionEvent>>,
) {
    let (messages, connection_events) = connection_manager.receive();

    message_events.extend(messages.into_iter());
    connection_events_resource.extend(connection_events.into_iter());
}

pub fn sending_system(
    mut connection_manager: ResMut<ConnectionManager>,
    mut network_handle: ResMut<NetworkHandle>,
    mut connection_events: ResMut<Events<ConnectionEvent>>,
    mut network_entity_registry: ResMut<NetworkEntityRegistry>,
    mut spawn_manager: ResMut<SpawnManager>,
) {
    network_handle.convert_spawn_messages(
        &mut *network_entity_registry,
        &mut *spawn_manager,
        &*connection_manager,
    );

    let payloads = network_handle.clear_payloads();
    let events = connection_manager.send(payloads);

    connection_events.extend(events.into_iter());
}
