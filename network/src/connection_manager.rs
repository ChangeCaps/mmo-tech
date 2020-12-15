use crate::*;
use bevy::{prelude::*, reflect::Uuid};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{self, prelude::*},
    net::{SocketAddr, TcpListener, TcpStream},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ConnectionId(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ActorId(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ActorTy(pub Uuid);

impl ActorTy {
    pub fn new<T: NetworkTypeUuid>() -> Self {
        ActorTy(T::UUID)
    }

    pub fn is<T: NetworkTypeUuid>(&self) -> bool {
        self.0 == T::UUID
    }
}

#[derive(Clone, Debug)]
pub struct Actor {
    id: ActorId,
    ty: ActorTy,
}

impl Actor {
    pub fn new(id: ActorId, ty: ActorTy) -> Self {
        Self { id, ty }
    }

    pub fn targeted_by(&self, network_target: &NetworkTarget) -> bool {
        match network_target {
            NetworkTarget::All => true,
            NetworkTarget::ActorId(actor_id) => self.id == *actor_id,
            NetworkTarget::ActorTy(actor_ty) => self.ty == *actor_ty,
        }
    }

    pub fn id(&self) -> ActorId {
        self.id
    }

    pub fn ty(&self) -> ActorTy {
        self.ty
    }
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Connected {
        actor: Actor,
        connection_id: ConnectionId,
    },
    Disconnected {
        actor: Actor,
        connection_id: ConnectionId,
        cause: crate::Error,
    },
}

pub enum ConnectionInner {
    External {
        addr: SocketAddr,
        stream: TcpStream,
        next_len: Option<usize>,
    },
    Internal {
        payloads: Vec<Payload>,
    },
}

impl ConnectionInner {
    pub fn send(&mut self, mut payloads: Vec<Payload>) -> Result<(), crate::Error> {
        match self {
            ConnectionInner::External { stream, .. } => {
                let bytes = serde_cbor::to_vec(&payloads)?;
                let len = (bytes.len() as u64).to_be_bytes();

                stream.write(&len)?;
                stream.write(&bytes)?;

                Ok(())
            }
            ConnectionInner::Internal {
                payloads: internal_payloads,
            } => {
                internal_payloads.append(&mut payloads);

                Ok(())
            }
        }
    }

    pub fn receive(&mut self) -> Result<Vec<Payload>, crate::Error> {
        match self {
            ConnectionInner::External {
                stream, next_len, ..
            } => {
                let len = match next_len {
                    Some(v) => *v,
                    None => {
                        let mut buf = [0u8; 8];

                        stream.read(&mut buf)?;

                        u64::from_be_bytes(buf) as usize
                    }
                };

                if len > 10000 {
                    warn!(len);
                }

                let mut buf = vec![0u8; len];

                if stream.peek(&mut buf)? < len {
                    *next_len = Some(len);
                    return Err(crate::Error::Io(std::io::Error::new(std::io::ErrorKind::WouldBlock, "would block")));
                }

                match stream.read(&mut buf) {
                    Ok(_) => *next_len = None,
                    Err(e) => {
                        *next_len = Some(len);
                        return Err(e.into());
                    }
                }

                Ok(serde_cbor::from_slice(&buf)?)
            }
            ConnectionInner::Internal { payloads } => Ok(std::mem::replace(payloads, Vec::new())),
        }
    }
}

pub struct Connection {
    inner: ConnectionInner,
    actor: Actor,
}

impl Connection {
    pub fn send(&mut self, payloads: Vec<Payload>) -> Result<(), crate::Error> {
        match self.inner.send(payloads) {
            Ok(_) => Ok(()),
            Err(crate::Error::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn receive(&mut self) -> Result<Vec<Payload>, crate::Error> {
        let mut payloads = Vec::new();

        loop {
            match self.inner.receive() {
                Ok(mut v) => {
                    payloads.append(&mut v);

                    if let ConnectionInner::Internal { .. } = self.inner {
                        return Ok(payloads);
                    }
                }
                Err(crate::Error::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Ok(payloads);
                }
                Err(e) => return Err(e),
            }
        }
    }
}

pub struct ConnectionManager {
    connections: HashMap<ConnectionId, Connection>,
    connection_ids: HashMap<ActorId, ConnectionId>,

    next_connection_id: ConnectionId,
    next_actor_id: ActorId,

    local_connection_id: ConnectionId,
    local_actor_id: ActorId,
}

impl ConnectionManager {
    pub fn new(actor_ty: ActorTy) -> Self {
        let internal_connection = Connection {
            inner: ConnectionInner::Internal {
                payloads: Vec::new(),
            },
            actor: Actor {
                id: ActorId(0),
                ty: actor_ty,
            },
        };

        let mut connections = HashMap::new();
        connections.insert(ConnectionId(0), internal_connection);

        let mut connection_ids = HashMap::new();
        connection_ids.insert(ActorId(0), ConnectionId(0));

        Self {
            connections,
            connection_ids,

            next_connection_id: ConnectionId(1),
            next_actor_id: ActorId(1),

            local_connection_id: ConnectionId(0),
            local_actor_id: ActorId(0),
        }
    }

    pub fn get_targeted_actor_ids(&self, target: &NetworkTarget) -> Vec<ActorId> {
        match target {
            NetworkTarget::All => self
                .connections
                .iter()
                .map(|(_, connection)| connection.actor.id)
                .collect(),
            NetworkTarget::ActorId(actor_id) => {
                if let Some(connection) = self.get(*actor_id) {
                    vec![connection.actor.id]
                } else {
                    Vec::new()
                }
            }
            NetworkTarget::ActorTy(actor_ty) => self
                .connections
                .iter()
                .filter(|(_, connection)| connection.actor.ty == *actor_ty)
                .map(|(_, connection)| connection.actor.id)
                .collect(),
        }
    }

    pub fn get_targeted_connection_ids(&self, target: &NetworkTarget) -> Vec<ConnectionId> {
        match target {
            NetworkTarget::All => self
                .connections
                .iter()
                .map(|(connection_id, _)| *connection_id)
                .collect(),
            NetworkTarget::ActorId(actor_id) => {
                if let Some(connection_id) = self.get_connection_id(actor_id) {
                    vec![*connection_id]
                } else {
                    Vec::new()
                }
            }
            NetworkTarget::ActorTy(actor_ty) => self
                .connections
                .iter()
                .filter(|(_, connection)| connection.actor.ty == *actor_ty)
                .map(|(connection_id, _)| *connection_id)
                .collect(),
        }
    }

    pub fn send(
        &mut self,
        targeted_payloads: Vec<(NetworkTarget, Payload)>,
    ) -> Vec<ConnectionEvent> {
        let mut connection_id_payloads: HashMap<ConnectionId, Vec<Payload>> = HashMap::new();

        for (target, payload) in targeted_payloads {
            for connection_id in self.get_targeted_connection_ids(&target) {
                connection_id_payloads
                    .entry(connection_id)
                    .or_insert(Vec::new())
                    .push(payload.clone());
            }
        }

        let mut connection_events = Vec::new();

        for (connection_id, payloads) in connection_id_payloads {
            if let Some(connection) = self.get_mut(connection_id) {
                match connection.send(payloads) {
                    Ok(_) => (),
                    Err(e) => connection_events.push(ConnectionEvent::Disconnected {
                        connection_id,
                        actor: connection.actor.clone(),
                        cause: e,
                    }),
                }
            }
        }

        connection_events
    }

    pub fn receive(&mut self) -> (Vec<Message>, Vec<ConnectionEvent>) {
        let mut connection_events = Vec::new();
        let mut messages = Vec::new();

        let local_actor = self
            .get_actor(self.local_actor_id)
            .expect("Local internal connection does for some reason not exist.")
            .clone();

        for (connection_id, connection) in self.connections_mut() {
            match connection.receive() {
                Ok(payloads) => {
                    let mut connection_messages: Vec<_> = payloads
                        .into_iter()
                        .map(|payload| Message {
                            payload,
                            sender: connection.actor.clone(),
                            receiver: local_actor.clone(),
                        })
                        .collect();

                    messages.append(&mut connection_messages);
                }
                Err(e) => {
                    connection_events.push(ConnectionEvent::Disconnected {
                        connection_id: *connection_id,
                        actor: connection.actor.clone(),
                        cause: e,
                    });
                }
            }
        }

        (messages, connection_events)
    }

    pub fn connections(&self) -> impl Iterator<Item = (&ConnectionId, &Connection)> {
        self.connections.iter()
    }

    pub fn connections_mut(&mut self) -> impl Iterator<Item = (&ConnectionId, &mut Connection)> {
        self.connections.iter_mut()
    }

    pub fn get_actor(&self, get: impl GetFromConnectionManager) -> Option<&Actor> {
        self.get(get).map(|connection| &connection.actor)
    }

    pub fn get(&self, get: impl GetFromConnectionManager) -> Option<&Connection> {
        get.get(self)
    }

    pub fn get_mut(&mut self, get: impl GetFromConnectionManager) -> Option<&mut Connection> {
        get.get_mut(self)
    }

    pub fn remove(&mut self, actor_id: ActorId) {
        if let Some(connection_id) = self.connection_ids.remove(&actor_id) {
            self.connections.remove(&connection_id);
        }
    }

    pub fn get_connection_id(&self, actor_id: &ActorId) -> Option<&ConnectionId> {
        self.connection_ids.get(actor_id)
    }

    pub fn get_local_actor(&self) -> Option<&Actor> {
        self.get(self.local_actor_id)
            .map(|connection| &connection.actor)
    }

    pub fn generate_connection_id(&mut self) -> ConnectionId {
        let id = self.next_connection_id;
        self.next_connection_id.0 += 1;
        id
    }

    pub fn generate_actor_id(&mut self) -> ActorId {
        let id = self.next_actor_id;
        self.next_actor_id.0 += 1;
        id
    }

    pub fn set_local_actor_id(&mut self, actor_id: ActorId) {
        let connection_id = self
            .connection_ids
            .remove(&self.local_actor_id)
            .expect("Welp, apparently an interal local connection does not exist");

        self.connection_ids.insert(actor_id, connection_id);

        self.get_mut(connection_id)
            .expect("Welp, apparently an interal local connection does not exist")
            .actor
            .id = actor_id;

        self.local_actor_id = actor_id;
    }

    pub fn add_connection(
        &mut self,
        mut stream: TcpStream,
        actor_ty: ActorTy,
        send_handshake: Handshake,
    ) -> ConnectionEvent {
        stream.set_nodelay(true).unwrap();
        stream.set_nonblocking(false).unwrap();

        let bytes = serde_cbor::to_vec(&send_handshake).unwrap();
        let len = (bytes.len() as u64).to_be_bytes();
        stream.write(&len).unwrap();
        stream.write(&bytes).unwrap();

        let mut buf = [0u8; 8];
        stream.read(&mut buf).unwrap();
        let len = u64::from_be_bytes(buf) as usize;

        let mut buf = vec![0u8; len];
        stream.read(&mut buf).unwrap();
        let handshake: Handshake = serde_cbor::from_slice(&buf).unwrap();

        let actor_id = match handshake {
            Handshake::Override {
                sender_actor_id,
                receiver_actor_id,
            } => {
                self.set_local_actor_id(receiver_actor_id);
                sender_actor_id
            }
            Handshake::None => match send_handshake {
                Handshake::Override {
                    receiver_actor_id, ..
                } => receiver_actor_id,
                Handshake::None => self.generate_actor_id(),
            },
        };

        let actor = Actor::new(actor_id, actor_ty);

        stream.set_nonblocking(true).unwrap();

        let connection_id = self.generate_connection_id();
        let connection = Connection {
            inner: ConnectionInner::External {
                addr: stream.peer_addr().unwrap(),
                stream,
                next_len: None,
            },
            actor: actor.clone(),
        };

        self.connections.insert(connection_id, connection);
        self.connection_ids.insert(actor.id(), connection_id);

        ConnectionEvent::Connected {
            actor,
            connection_id,
        }
    }
}

pub fn disconnect_handler_system(
    mut connection_manager: ResMut<ConnectionManager>,
    mut event_reader: Local<EventReader<ConnectionEvent>>,
    events: Res<Events<ConnectionEvent>>,
) {
    for event in event_reader.iter(&events) {
        if let ConnectionEvent::Disconnected { actor, .. } = event {
            connection_manager.remove(actor.id());
        }
    }
}

pub trait GetFromConnectionManager {
    fn get<'a>(&self, connection_manager: &'a ConnectionManager) -> Option<&'a Connection>;
    fn get_mut<'a>(
        &self,
        connection_manager: &'a mut ConnectionManager,
    ) -> Option<&'a mut Connection>;
}

impl GetFromConnectionManager for ConnectionId {
    fn get<'a>(&self, connection_manager: &'a ConnectionManager) -> Option<&'a Connection> {
        connection_manager.connections.get(self)
    }

    fn get_mut<'a>(
        &self,
        connection_manager: &'a mut ConnectionManager,
    ) -> Option<&'a mut Connection> {
        connection_manager.connections.get_mut(self)
    }
}

impl GetFromConnectionManager for ActorId {
    fn get<'a>(&self, connection_manager: &'a ConnectionManager) -> Option<&'a Connection> {
        if let Some(connection_id) = connection_manager.connection_ids.get(self) {
            connection_manager.connections.get(connection_id)
        } else {
            None
        }
    }

    fn get_mut<'a>(
        &self,
        connection_manager: &'a mut ConnectionManager,
    ) -> Option<&'a mut Connection> {
        if let Some(connection_id) = connection_manager.connection_ids.get(self) {
            connection_manager.connections.get_mut(connection_id)
        } else {
            None
        }
    }
}
