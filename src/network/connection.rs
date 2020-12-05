use super::*;
use bevy::{
    prelude::*,
    reflect::{TypeUuid, Uuid},
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    io::prelude::*,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub u64);

pub enum ConnectionEvent {
    Connected(ConnectionId),
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionType(Uuid);

impl ConnectionType {
    pub fn is<T: TypeUuid>(&self) -> bool {
        self.0 == T::TYPE_UUID
    }

    pub fn from<T: TypeUuid>() -> Self {
        Self(T::TYPE_UUID)
    }
}

pub struct Connection {
    addr: SocketAddr,
    ty: ConnectionType,
    stream: TcpStream,
}

impl Connection {
    pub fn send(&mut self, payload: &NetworkMessagePayload) -> Result<(), std::io::Error> {
        self.stream.write(&bincode::serialize(payload).unwrap())?;
        Ok(())
    }

    pub fn receive() {
        
    }
}

pub enum ConnectionMethod {
    Connect(Vec<SocketAddr>),
    Listen(SocketAddr),
}

#[derive(Default)]
pub struct Connections {
    ids: HashMap<SocketAddr, ConnectionId>,
    connections: HashMap<ConnectionId, Connection>,
    next_id: ConnectionId,
}

impl Connections {
    pub fn iter(&self) -> impl Iterator<Item = (&ConnectionId, &Connection)> {
        self.connections.iter()
    }

    pub fn send(
        &mut self,
        payload: &NetworkMessagePayload,
        target: &NetworkTarget,
    ) -> Result<(), NetworkError> {
        match target {
            NetworkTarget::Connection(target) => {
                if let Some(connection) = self.connections.get_mut(target) {
                    connection.send(payload).map_err(|e| NetworkError::Io(e))
                } else {
                    Err(NetworkError::MissingConnection)
                }
            }
            NetworkTarget::ConnectionType(connection_type) => {
                for connection in self.connections.values_mut() {
                    if connection.ty == *connection_type {
                        connection.send(payload).map_err(|e| NetworkError::Io(e))?;
                    }
                }

                Ok(())
            }
            NetworkTarget::All => {
                for connection in self.connections.values_mut() {
                    connection.send(payload).map_err(|e| NetworkError::Io(e))?;
                }

                Ok(())
            }
        }
    }

    pub fn add_connection<T: TypeUuid>(&mut self, stream: TcpStream) {
        let id = self.next_id;
        self.next_id.0 += 1;
        let addr = stream.peer_addr().unwrap();
        log::info!("Connected to: {}", addr);
        self.ids.insert(addr, id);
        self.connections.insert(
            id,
            Connection {
                stream,
                ty: ConnectionType::from::<T>(),
                addr,
            },
        );
    }
}

pub struct ServerConnectionResource {
    pub listener: TcpListener,
}

pub struct ClientConnectionResource {
    pub server_addrs: Vec<SocketAddr>,
    pub connection_threads: HashMap<
        SocketAddr,
        (
            JoinHandle<Result<TcpStream, std::io::Error>>,
            Arc<AtomicBool>,
        ),
    >,
}

pub fn server_connection_system(
    server_connection_resource: Res<ServerConnectionResource>,
    mut connections: ResMut<Connections>,
) {
    for stream in server_connection_resource.listener.incoming() {
        match stream {
            Ok(stream) => connections.add_connection::<super::Client>(stream),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return;
            }
            Err(e) => log::warn!("{}", e),
        }
    }
}

pub fn client_connection_system(
    mut client_connection_resource: ResMut<ClientConnectionResource>,
    mut connections: ResMut<Connections>,
) {
    let client_connection_resource = &mut *client_connection_resource;

    for addr in &client_connection_resource.server_addrs {
        if client_connection_resource
            .connection_threads
            .contains_key(addr)
        {
            continue;
        }

        let connected = Arc::new(AtomicBool::new(false));
        let connected_thread = connected.clone();
        let addr = *addr;

        client_connection_resource.connection_threads.insert(
            addr,
            (
                thread::spawn(move || {
                    let stream =
                        TcpStream::connect_timeout(&addr, std::time::Duration::from_secs_f32(10.0));
                    connected_thread.store(true, std::sync::atomic::Ordering::SeqCst);
                    stream
                }),
                connected,
            ),
        );
    }

    let server_addrs = &mut client_connection_resource.server_addrs;
    let connection_threads = &mut client_connection_resource.connection_threads;

    server_addrs.retain(|server_addr| {
        let connected = connection_threads[server_addr]
            .1
            .load(std::sync::atomic::Ordering::SeqCst);

        if connected {
            let (join_handle, _) = connection_threads.remove(server_addr).unwrap();

            match join_handle.join().unwrap() {
                Ok(stream) => connections.add_connection::<super::Server>(stream),
                Err(e) => panic!("failed to connect to server {}", e),
            }
        }

        !connected
    });
}
