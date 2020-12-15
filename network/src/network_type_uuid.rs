use bevy::reflect::Uuid;

pub trait NetworkTypeUuid {
    const UUID: Uuid;
}

#[macro_export]
macro_rules! network_uuid {
    ($ident:path = $uuid:expr) => {
        impl NetworkTypeUuid for $ident {
            const UUID: bevy::reflect::Uuid = bevy::reflect::Uuid::from_u128($uuid);
        }
    };
}
