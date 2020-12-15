use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Handshake {
    Override {
        sender_actor_id: ActorId,
        receiver_actor_id: ActorId,
    },
    None,
}
