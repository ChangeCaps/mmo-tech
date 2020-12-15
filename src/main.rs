use bevy::prelude::*;
use clap::*;
use std::net::{TcpListener, TcpStream};

pub mod animation;
pub mod component;
pub mod map;
pub mod player;
pub mod target_position;
pub mod z_sort;
pub use animation::*;
pub use component::*;
pub use map::*;
pub use network::*;
pub use player::*;
pub use target_position::*;
pub use z_sort::*;

#[derive(Clap)]
enum Mode {
    Server(Server),
    Client(Client),
}

#[derive(Clap)]
struct Server {
    ip: std::net::SocketAddr,
}

impl Server {
    pub fn run(&self) {
        let listener = TcpListener::bind(self.ip).unwrap();

        bevy::prelude::App::build()
            // resources
            .init_resource::<Map>()
            .add_resource(bevy::app::ScheduleRunnerSettings::run_loop(
                std::time::Duration::from_secs_f32(1.0 / 20.0),
            ))
            // plugins
            .add_plugin(network::NetworkPlugin::server(listener))
            .add_plugins(MinimalPlugins)
            .add_plugin(bevy::log::LogPlugin)
            // component sync
            .add_component_sync::<MovementDirection>()
            .add_component_sync::<Transform>()
            .add_component_sync::<TargetPosition>()
            .add_component_sync::<Tile>()
            .add_component_sync::<Animator>()
            // startup systems
            .add_startup_system(setup_server)
            // systems
            .add_system(server_connection_handler)
            .add_system(player_spawn_system)
            .add_system(player_movement_system)
            .add_system(target_position_system)
            .add_system(tile_transform_system)
            .add_system(animator_system)
            // run
            .run();
    }
}

#[derive(Clap)]
struct Client {
    ip: String,
}

impl Client {
    pub fn run(&self) {
        let stream = TcpStream::connect(self.ip.clone()).unwrap();

        bevy::prelude::App::build()
            // resources
            .init_resource::<player::Player>()
            .init_resource::<Map>()
            .add_resource(WindowDescriptor {
                vsync: true,
                ..Default::default()
            })
            // plugins
            .add_plugin(network::NetworkPlugin::client(stream))
            .add_plugins(DefaultPlugins)
            // component sync
            .add_component_sync::<MovementDirection>()
            .add_component_sync::<Transform>()
            .add_component_sync::<TargetPosition>()
            .add_component_sync::<Tile>()
            .add_component_sync::<Animator>()
            // startup systems
            .add_startup_system(setup_client)
            // systems
            .add_system(player_input_system)
            .add_system(player_camera_system)
            .add_system(target_position_system)
            .add_system(tile_transform_system)
            .add_system(animator_system)
            .add_system(animator_sprite_system)
            .add_system(z_sort_system)
            // run
            .run();
    }
}

#[derive(Clap)]
#[clap(version = crate_version!(), author = "Hjalte Nannestad")]
struct Options {
    #[clap(subcommand)]
    mode: Mode,
}

fn main() {
    let options = Options::parse();

    match options.mode {
        Mode::Server(server) => server.run(),
        Mode::Client(client) => client.run(),
    }
}

fn server_connection_handler(
    mut event_reader: Local<EventReader<ConnectionEvent>>,
    events: Res<Events<ConnectionEvent>>,
) {
    for event in event_reader.iter(&events) {
        match event {
            ConnectionEvent::Connected {
                actor,
                connection_id,
            } => {
                println!(
                    "Player Connected: {:?}, given actor id: {:?}",
                    connection_id,
                    actor.id()
                );
            }
            ConnectionEvent::Disconnected {
                actor,
                connection_id,
                cause,
            } => {
                println!(
                    "Player Disconnected: {:?}, {:?}, {:?}",
                    connection_id,
                    actor.id(),
                    cause
                );
            }
        }
    }
}

fn setup_server(commands: &mut Commands) {
    commands.spawn(Camera2dBundle {
        ..Default::default()
    });
}

fn setup_client(commands: &mut Commands, mut player: ResMut<Player>) {
    info!("yeet");

    let camera_entity = commands
        .spawn(Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            ..Default::default()
        })
        .current_entity()
        .unwrap();

    player.camera = Some(camera_entity);
}
