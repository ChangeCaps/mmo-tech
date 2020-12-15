use crate::*;
use bevy::prelude::*;
use std::net::TcpListener;

pub struct Listener {
    inner: TcpListener,
}

impl Listener {
    pub fn new(listener: TcpListener) -> Self {
        listener.set_nonblocking(true).unwrap();

        Self { inner: listener }
    }
}

pub fn listening_system(
    listener: Res<Listener>,
    network_settings: Res<NetworkSettings>,
    mut connection_manager: ResMut<ConnectionManager>,
    mut connection_events: ResMut<Events<ConnectionEvent>>,
) {
    for stream in listener.inner.incoming() {
        match stream {
            Ok(stream) => {
                let handshake = Handshake::Override {
                    receiver_actor_id: connection_manager.generate_actor_id(),
                    sender_actor_id: connection_manager.get_local_actor().unwrap().id(),
                };

                let event = connection_manager.add_connection(
                    stream,
                    network_settings.connection_ty,
                    handshake,
                );

                connection_events.send(event);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return,
            Err(e) => log::warn!("{:?}", e),
        }
    }
}
