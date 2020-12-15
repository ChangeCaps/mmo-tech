mod connection_manager;
mod message;
#[macro_use]
mod network_type_uuid;
mod communication;
mod component_sync;
mod error;
mod handshake;
mod listener;
mod network_entity;
mod plugin;
mod settings;
mod spawnable;
mod syncable_component;
pub use communication::*;
pub use component_sync::*;
pub use connection_manager::*;
pub use error::*;
pub use handshake::*;
pub use listener::*;
pub use message::*;
pub use network_entity::*;
pub use network_type_uuid::*;
pub use plugin::*;
pub use serde::{Deserialize, Serialize};
pub use settings::*;
pub use spawnable::*;
pub use syncable_component::*;

pub struct Server;
pub struct Client;
network_uuid!(Server = 481231321231324654321324);
network_uuid!(Client = 1213214233146531133716878546);
