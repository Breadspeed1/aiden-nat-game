use args::Args;
use bevy::ecs::system::SystemId;
use bevy::render::camera::ScalingMode;
use bevy::window::EnabledButtons;
use bevy::{prelude::*, window::WindowResolution};
use bevy_ggrs::*;
use bevy_matchbox::matchbox_socket::PeerId;
use bevy_matchbox::prelude::{MultipleChannels, SingleChannel};
use bevy_matchbox::{CloseSocketExt, MatchboxSocket};
use bevy_roll_safe::RollApp;
use clap::Parser;
use components::{CoyoteTime, Player};
use input::handle_window_resize;
use physics::PhysicsPlugin;
use resources::WindowScale;
use states::full_lobby::FullLobbyPlugin;
use states::main_menu::MainMenuPlugin;
use states::waiting_lobby::{CMRole, GameConfig, RoomID, WaitingLobbyPlugin};

mod args;
mod components;
mod input;
mod interactions;
mod level;
mod movement;
mod physics;
mod resources;
mod states;

type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

pub const MIN_WINDOW_SIZE: f32 = 196.0;

fn main() {
    let args = Args::parse();
    info!("{args:?}");

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        resizable: false,
                        resolution: WindowResolution::new(MIN_WINDOW_SIZE, MIN_WINDOW_SIZE),
                        mode: bevy::window::WindowMode::Windowed,
                        enabled_buttons: EnabledButtons {
                            minimize: true,
                            maximize: false,
                            close: true,
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            GgrsPlugin::<Config>::default(),
            MainMenuPlugin,
            WaitingLobbyPlugin,
            FullLobbyPlugin,
        ))
        .init_state::<AppState>()
        .add_plugins(PhysicsPlugin)
        .insert_resource(args)
        .insert_resource(WindowScale::new())
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_window_resize)
        .add_systems(ReadInputs, input::read_local_inputs)
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_copy::<Player>()
        .rollback_component_with_clone::<CoyoteTime>()
        .init_ggrs_state::<MultiplayerGameState>()
        .add_systems(
            OnExit(AppState::FullLobby),
            (
                close_session.run_if(resource_exists::<Session<Config>>),
                close_socket.run_if(resource_exists::<MatchboxSocket<SingleChannel>>),
                remove_multiplayer_resources,
            )
                .chain(),
        )
        .add_systems(OnEnter(AppState::CreateGameMenu), add_bearer_role)
        .add_systems(OnEnter(AppState::JoinGameMenu), add_receive_role)
        .insert_resource(RoomID(20))
        .run();
}

#[derive(Debug, Resource)]
pub struct DespawnAllButCameraID(pub SystemId);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    SettingsMenu,
    PastGamesMenu,
    CreateGameMenu,
    JoinGameMenu,
    WaitingInLobby,
    FullLobby,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum MultiplayerGameState {
    #[default]
    Idle,
    InLobby,
}

fn remove_multiplayer_resources(mut commands: Commands) {
    commands.close_socket::<MultipleChannels>();
    commands.remove_resource::<GameConfig>();
    commands.remove_resource::<CMRole>();
    commands.remove_resource::<bevy_ggrs::Session<Config>>();
    info!("Removed multiplayer resources");
}

fn close_socket(mut socket: ResMut<MatchboxSocket<SingleChannel>>) {
    socket.close();
    info!("closed socket");
}

fn close_session(session: ResMut<Session<Config>>) {
    match session.into_inner() {
        Session::<_>::P2P(session) => {
            for player in session.remote_player_handles() {
                let _ = session.disconnect_player(player);
            }
        }
        _ => {}
    }
}

fn add_receive_role(mut commands: Commands, mut next_state: ResMut<NextState<AppState>>) {
    commands.insert_resource::<CMRole>(CMRole::ConfigReciever);
    next_state.set(AppState::WaitingInLobby);
}

fn add_bearer_role(mut commands: Commands, mut next_state: ResMut<NextState<AppState>>) {
    commands.insert_resource::<CMRole>(CMRole::ConfigBearer(GameConfig {
        seed: 10,
        difficulty: 1,
    }));
    next_state.set(AppState::WaitingInLobby);
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::Fixed {
        width: 10.,
        height: 10.,
    };
    commands.spawn(camera_bundle);
    let id = commands.register_one_shot_system(despawn_all_but_camera);
    commands.insert_resource(DespawnAllButCameraID(id));
}

pub fn despawn_all_but_camera(
    mut commands: Commands,
    query: Query<Entity, (Without<Camera>, Without<Window>)>,
) {
    for entity in &query {
        if let Some(e) = commands.get_entity(entity) {
            e.despawn_recursive();
        }
    }
}
