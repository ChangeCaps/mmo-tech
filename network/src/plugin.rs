use crate::*;
use bevy::prelude::*;
use std::net::{TcpListener, TcpStream};

pub mod stage {
    pub const NETWORK_SEND: &'static str = "network_send";
    pub const NETWORK_PRE_SEND: &'static str = "network_pre_send";
    pub const NETWORK_SYNC_MARK: &'static str = "network_sync_mark";
    pub const NETWORK_RECEIVE: &'static str = "network_receive";
    pub const NETWORK_POST_RECEIVE: &'static str = "network_post_receive";
}

pub enum ConnectionMethod {
    Stream(TcpStream),
    Listener(TcpListener),
}

pub trait AppBuilderExt {
    fn app_builder(&mut self) -> &mut AppBuilder;

    fn add_component_sync<T: SyncableComponent + Send + Sync + 'static>(
        &mut self,
    ) -> &mut AppBuilder {
        let app_builder = self.app_builder();

        app_builder.add_system_to_stage(
            stage::NETWORK_POST_RECEIVE,
            component_sync_receiving_system::<T>,
        );
        app_builder
            .add_system_to_stage(stage::NETWORK_PRE_SEND, component_sync_sending_system::<T>);

        app_builder
            .add_system_to_stage(stage::NETWORK_SYNC_MARK, component_sync_marking_system::<T>);
        app_builder
            .add_system_to_stage(stage::NETWORK_SYNC_MARK, component_sync_connect_system::<T>);

        app_builder
    }
}

impl AppBuilderExt for AppBuilder {
    fn app_builder(&mut self) -> &mut AppBuilder {
        self
    }
}

pub struct NetworkPlugin {
    settings: NetworkSettings,
    connection_method: ConnectionMethod,
}

impl NetworkPlugin {
    pub fn server(listener: TcpListener) -> Self {
        Self {
            settings: NetworkSettings::server(),
            connection_method: ConnectionMethod::Listener(listener),
        }
    }

    pub fn client(stream: TcpStream) -> Self {
        Self {
            settings: NetworkSettings::client(),
            connection_method: ConnectionMethod::Stream(stream),
        }
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_stage_after(
            bevy::app::stage::PRE_UPDATE,
            stage::NETWORK_POST_RECEIVE,
            SystemStage::parallel().with_run_criteria(bevy::core::FixedTimestep::step(1.0 / 20.0)),
        );
        app_builder.add_stage_before(
            stage::NETWORK_POST_RECEIVE,
            stage::NETWORK_RECEIVE,
            SystemStage::parallel().with_run_criteria(bevy::core::FixedTimestep::step(1.0 / 20.0)),
        );

        app_builder.add_stage_before(
            bevy::app::stage::POST_UPDATE,
            stage::NETWORK_SEND,
            SystemStage::parallel().with_run_criteria(bevy::core::FixedTimestep::step(1.0 / 20.0)),
        );
        app_builder.add_stage_before(
            stage::NETWORK_SEND,
            stage::NETWORK_PRE_SEND,
            SystemStage::parallel().with_run_criteria(bevy::core::FixedTimestep::step(1.0 / 20.0)),
        );
        app_builder.add_stage_before(
            stage::NETWORK_PRE_SEND,
            stage::NETWORK_SYNC_MARK,
            SystemStage::parallel(),
        );

        let mut connection_manager = ConnectionManager::new(self.settings.actor_ty);

        match &self.connection_method {
            ConnectionMethod::Stream(stream) => {
                let handshake = Handshake::None;

                connection_manager.add_connection(
                    stream.try_clone().unwrap(),
                    self.settings.connection_ty,
                    handshake,
                );
            }
            ConnectionMethod::Listener(listener) => {
                app_builder.add_resource(Listener::new(listener.try_clone().unwrap()));

                app_builder.add_system(listening_system);
            }
        }

        app_builder.add_resource(connection_manager);
        app_builder.add_resource(self.settings.clone());

        app_builder.init_resource::<NetworkHandle>();
        app_builder.init_resource::<NetworkEntityRegistry>();
        app_builder.init_resource::<SpawnSystemEventReader>();
        app_builder.init_resource::<SpawnManager>();

        app_builder.add_event::<ConnectionEvent>();
        app_builder.add_event::<Message>();

        app_builder.add_system_to_stage(stage::NETWORK_POST_RECEIVE, spawn_system);
        app_builder.add_system_to_stage(stage::NETWORK_RECEIVE, receiving_system);
        app_builder.add_system_to_stage(stage::NETWORK_SEND, sending_system);
        app_builder.add_system_to_stage(stage::NETWORK_POST_RECEIVE, disconnect_handler_system);
        app_builder.add_system_to_stage(stage::NETWORK_POST_RECEIVE, spawn_detection_system);
    }
}
