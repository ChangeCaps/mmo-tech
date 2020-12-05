use bevy::{prelude::*};
use clap::*;

pub mod component;
pub mod network;
use network::AppBuilderExt;

#[derive(Clap)]
enum Mode {
    Server { ip: std::net::SocketAddr },
    Client { ip: std::net::SocketAddr },
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
        Mode::Server { ip } => {
            bevy::prelude::App::build()
                .add_resource(bevy::app::ScheduleRunnerSettings::run_loop(
                    std::time::Duration::from_secs_f32(1.0 / 60.0),
                ))
                .add_plugins(MinimalPlugins)
                .add_plugin(network::NetworkPlugin::server(ip))
                .add_startup_system(setup.system())
                .run();
        }
        Mode::Client { ip } => {
            bevy::prelude::App::build()
                .add_resource(bevy::app::ScheduleRunnerSettings::run_loop(
                    std::time::Duration::from_secs_f32(1.0 / 60.0),
                ))
                .add_plugins(DefaultPlugins)
                .add_plugin(network::NetworkPlugin::client(ip))
                .add_startup_system(setup.system())
                .run();
        }
    }
}

fn setup(commands: &mut Commands) {
    commands.spawn(CameraUiBundle {
        ..Default::default()
    });
}
