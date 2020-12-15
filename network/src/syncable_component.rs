use crate::*;
use bevy::{prelude::*, reflect::TypeRegistry};

pub trait SyncableComponent: NetworkTypeUuid {
    fn from_bytes(bytes: &[u8], type_registry: &TypeRegistry) -> Self;
    fn to_bytes(&self, type_registry: &TypeRegistry) -> Vec<u8>;
}

#[macro_export]
macro_rules! serde_sync {
    ($ident:path) => {
        impl SyncableComponent for $ident {
            fn from_bytes(bytes: &[u8], _type_registry: &bevy::reflect::TypeRegistry) -> Self {
                serde_cbor::from_slice(bytes).unwrap()
            }

            fn to_bytes(&self, _type_registry: &bevy::reflect::TypeRegistry) -> Vec<u8> {
                serde_cbor::to_vec(self).unwrap()
            }
        }
    };

    ($ident:path = $uuid:expr) => {
        serde_sync!($ident);
        network_uuid!($ident = $uuid);
    };
}

#[macro_export]
macro_rules! reflect_sync {
    ($ident:path) => {
        impl SyncableComponent for $ident {
            fn from_bytes(bytes: &[u8], type_registry: &bevy::reflect::TypeRegistry) -> Self {
                use serde::de::DeserializeSeed;

                let type_registry = type_registry.read();

                let reflect_deserializer =
                    bevy::reflect::serde::ReflectDeserializer::new(&type_registry);
                let mut deserializer = serde_cbor::Deserializer::from_slice(bytes);
                let reflect_value = reflect_deserializer.deserialize(&mut deserializer).unwrap();

                let mut value = Self::default();

                value.apply(&*reflect_value);

                value
            }

            fn to_bytes(&self, type_registry: &bevy::reflect::TypeRegistry) -> Vec<u8> {
                let type_registry = type_registry.read();
                let serializer = bevy::reflect::serde::ReflectSerializer::new(self, &type_registry);
                serde_cbor::to_vec(&serializer).unwrap()
            }
        }
    };

    ($ident:path = $uuid:expr) => {
        reflect_sync!($ident);
        network_uuid!($ident = $uuid);
    };
}

reflect_sync!(Transform = 65786718953123561420596132);
reflect_sync!(GlobalTransform = 153143145317462349853894);
