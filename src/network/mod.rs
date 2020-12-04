use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    any::{Any, TypeId},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};

#[derive(Debug)]
pub enum NetworkError {
    Io(std::io::Error),
    MissingConnection,
}

pub struct Server;
pub struct Client;

pub mod connection;

use connection::*;

#[derive(Serialize, Deserialize)]
pub enum NetworkMessagePayload {
    SyncComponent { type_id: TypeId },
}

pub struct NetworkMessage {
    pub sender: ConnectionType,
    pub id: ConnectionId,
    pub payload: NetworkMessagePayload,
}

pub struct NetworkResource {
    pub stream: TcpStream,
}

#[derive(Default)]
pub struct NetworkHandle {
    messages: Vec<(ConnectionId, NetworkMessagePayload)>,
}

pub fn network_sending_system(
    mut connections: ResMut<Connections>,
    mut network_handle: ResMut<NetworkHandle>,
) {
    for (target, message) in network_handle.messages.drain(..) {
        connections.send(&message, &target).unwrap();
    }
}

pub struct NetworkPlugin(ConnectionMethod);

impl NetworkPlugin {
    pub fn server(addr: impl ToSocketAddrs) -> Self {
        Self(ConnectionMethod::Listen(
            addr.to_socket_addrs().unwrap().next().unwrap(),
        ))
    }

    pub fn client(addr: impl ToSocketAddrs) -> Self {
        Self(ConnectionMethod::Connect(
            addr.to_socket_addrs().unwrap().collect(),
        ))
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(network_sending_system);
        app.init_resource::<Connections>();
        app.init_resource::<NetworkHandle>();

        match &self.0 {
            ConnectionMethod::Connect(targets) => {
                app.add_system(client_connection_system.system());
                app.add_resource(ClientConnectionResource {
                    server_addrs: targets.clone(),
                    connection_threads: Default::default(),
                });
            }
            ConnectionMethod::Listen(target) => {
                let listener = TcpListener::bind(target).unwrap();
                listener.set_nonblocking(true).unwrap();

                app.add_system(server_connection_system.system());
                app.add_resource(ServerConnectionResource { listener });
            }
        }
    }
}
