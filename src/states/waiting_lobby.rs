use crate::components::Player;
use crate::physics::{Collider, Gravity, PhysicsSet, Solid, Velocity};
use bevy::prelude::*;
use bevy_ggrs::{ggrs, AddRollbackCommandExtension};
use bevy_matchbox::prelude::SingleChannel;
use bevy_matchbox::MatchboxSocket;

use crate::{AppState, Config};

pub struct WaitingLobbyPlugin;

impl Plugin for WaitingLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::WaitingInLobby),
            (
                spawn_floor,
                spawn_player,
                spawn_object,
                start_matchbox_socket,
            )
                .in_set(WaitingLobbySet::Setup),
        )
        .add_systems(
            Update,
            (
                wait_for_players,
                crate::input::read_local_inputs,
                crate::movement::move_player_singleplayer,
                crate::movement::reset,
            )
                .chain()
                .before(PhysicsSet)
                .in_set(WaitingLobbySet::Update)
                .run_if(in_state(AppState::WaitingInLobby)),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum WaitingLobbySet {
    Setup,
    Update,
}

fn start_matchbox_socket(mut commands: Commands) {
    //let room_url = "ws://ec2-3-145-94-96.us-east-2.compute.amazonaws.com:3536/aidennat?next=2";
    let room_url = "ws://localhost:3536/aidennat?next=2";
    info!("Connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
}

fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if socket.get_channel(0).is_err() {
        return;
    }

    socket.update_peers();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return;
    }

    info!("All peers have joined, going in-game");

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<Config>::new()
        .with_num_players(num_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));

    next_state.set(AppState::FullLobby);
}

fn spawn_player(mut commands: Commands) {
    // Player 1
    commands
        .spawn((
            Player { handle: 0 },
            Gravity(-9.8 * 10.),
            Collider::new(Vec2::new(1., 1.)),
            Solid(true),
            Velocity::default(),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0., 0.47, 1.),
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(-2., 2., 0.)),
                ..default()
            },
        ))
        .add_rollback();

    // Player 2
    commands
        .spawn((
            Player { handle: 1 },
            Gravity(-9.8 * 10.),
            Collider::new(Vec2::new(1., 1.)),
            Solid(true),
            Velocity::default(),
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(2., 2., 0.)),
                sprite: Sprite {
                    color: Color::srgb(0., 0.4, 0.),
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                ..default()
            },
        ))
        .add_rollback();
}

fn spawn_object(mut commands: Commands) {
    commands
        .spawn((
            Gravity(-9.8 * 10.),
            Collider::new(Vec2::new(1., 1.)),
            Solid(true),
            Velocity::default(),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0., 0.47, 0.47),
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0., 15., 0.)),
                ..default()
            },
        ))
        .add_rollback();
}

fn spawn_floor(mut commands: Commands) {
    commands
        .spawn((
            Collider::new(Vec2::new(10., 1.)),
            Solid(false),
            Velocity::default(),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1., 1., 1.),
                    custom_size: Some(Vec2::new(10., 1.)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0., -2., 0.)),
                ..default()
            },
        ))
        .add_rollback();
}
