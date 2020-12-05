use bevy::{
    prelude::*,
    reflect::{TypeUuid, Uuid},
};
use std::{
    any::{Any, TypeId},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};
use serde::{Serialize, Deserialize, de::DeserializeOwned};

pub mod component;
pub mod connection;

use component::*;
use connection::*;

#[derive(Debug)]
pub enum NetworkError {
    Io(std::io::Error),
    MissingConnection,
}

pub enum NetworkTarget {
    Connection(ConnectionId),
    ConnectionType(ConnectionType),
    All,
}

#[derive(TypeUuid)]
#[uuid = "2524ed5a-3395-41f3-b65b-5a2a09fc411f"]
pub struct Server;
#[derive(TypeUuid)]
#[uuid = "e44759f0-de6b-4d14-806d-b1485afbd6eb"]
pub struct Client;

#[derive(Serialize, Deserialize)]
pub enum NetworkMessagePayload {
    SyncComponent { type_id: Uuid, data: Vec<u8> },
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
    messages: Vec<(NetworkTarget, NetworkMessagePayload)>,
}

impl NetworkHandle {
    pub fn update_component<T: TypeUuid + Serialize + DeserializeOwned>(&mut self, component: &T) {
        self.messages.push((
            NetworkTarget::All,
            NetworkMessagePayload::SyncComponent {
                type_id: T::TYPE_UUID,
                data: bincode::serialize(component).unwrap(),
            },
        ));
    }
}

pub fn netowrk_receiving_system(
    mut connections: ResMut<Connections>,
) {
    
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

pub trait AppBuilderExt {
    fn app_builder(&mut self) -> &mut AppBuilder;

    fn register_component_sync<
        T: TypeUuid + Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
    >(
        &mut self,
    ) -> &mut Self {
        self.app_builder()
            .add_system(network_component_sync_system::<T>);
        self
    }
}

impl AppBuilderExt for AppBuilder {
    fn app_builder(&mut self) -> &mut AppBuilder {
        self
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
