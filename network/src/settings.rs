use crate::*;

#[derive(Clone)]
pub struct NetworkSettings {
    pub actor_ty: ActorTy,

    /// Sets all new connections type to this.
    pub connection_ty: ActorTy,

    pub sync_components_with: Vec<NetworkTarget>,
}

impl NetworkSettings {
    pub fn server() -> Self {
        Self {
            actor_ty: ActorTy::new::<Server>(),
            connection_ty: ActorTy::new::<Client>(),
            sync_components_with: vec![NetworkTarget::ActorTy(ActorTy::new::<Client>())],
        }
    }

    pub fn client() -> Self {
        Self {
            actor_ty: ActorTy::new::<Client>(),
            connection_ty: ActorTy::new::<Server>(),
            sync_components_with: vec![NetworkTarget::ActorTy(ActorTy::new::<Server>())],
        }
    }
}
