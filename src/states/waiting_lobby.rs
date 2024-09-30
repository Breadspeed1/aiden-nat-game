use crate::components::{CoyoteTime, Platform, Player, Vine};
use crate::physics::{Collider, Gravity, PhysicsSet, Solid, Velocity};
use bevy::prelude::*;
use bevy_ggrs::{ggrs, AddRollbackCommandExtension};
use bevy_matchbox::prelude::SingleChannel;
use bevy_matchbox::MatchboxSocket;

use crate::{AppState, Config};

const BOTTOM_PLATFORM_HEIGHT: f32 = ((196. - 178.) / 196.) * 10.;
const BOTTOM_PLATFORM_WIDTH: f32 = 10.;

const VINE_HEIGHT: f32 = ((177. - 93.) / 196.) * 10.;
const VINE_WIDTH: f32 = ((74. - 69.) / 196.) * 10.;
const VINE_POS_Y: f32 = (((177. - 93.) / 2. + (196. - 177.)) / 196.) * 10. - 5.;
const VINE_POS_X: f32 = (((74. - 69.) / 2. + 69.) / 196.) * 10. - 5.;

const MIDDLE_PLATFORM_HEIGHT: f32 = ((144. - 127.) / 196.) * 10.;
const MIDDLE_PLATFORM_LENGTH: f32 = ((196. - 122.) / 196.) * 10.;
const MIDDLE_PLATFORM_POS_Y: f32 = (((144. - 127.) / 2. + (196. - 144.)) / 196.) * 10. - 5.;
const MIDDLE_PLATFORM_POS_X: f32 = (((196. - 122.) / 2. + 122.) / 196.) * 10. - 5.;

const TOP_PLATFORM_HEIGHT: f32 = ((110. - 93.) / 196.) * 10.;
const TOP_PLATFORM_LENGTH: f32 = ((70.) / 196.) * 10.;
const TOP_PLATFORM_POS_Y: f32 = (((110. - 93.) / 2. + (196. - 110.)) / 196.) * 10. - 5.;
const TOP_PLATFORM_POS_X: f32 = (((70.) / 2.) / 196.) * 10. - 5.;

pub struct WaitingLobbyPlugin;

impl Plugin for WaitingLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::WaitingInLobby),
            (
                spawn_background,
                spawn_platforms,
                spawn_player,
                spawn_vines,
                start_matchbox_socket,
            )
                .in_set(WaitingLobbySet::Setup),
        )
        .add_systems(
            Update,
            wait_for_players
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
    let room_url = "ws://ec2-3-145-94-96.us-east-2.compute.amazonaws.com:3536/aidennat?next=2";
    //let room_url = "ws://localhost:3536/aidennat?next=2";
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

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_1_handle: Handle<Image> = asset_server.load("characters/nat.png");
    let player_2_handle: Handle<Image> = asset_server.load("characters/aiden.png");

    // Player 1
    commands
        .spawn((
            Player { handle: 0 },
            Gravity(-9.8 * 10., false),
            CoyoteTime::new(0.125),
            Collider::new(Vec2::new((1. / 8.167) * 10., (1. / 6.125) * 10.)),
            Solid(true),
            Velocity::default(),
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new((1. / 6.125) * 10., (1. / 6.125) * 10.)),
                    ..default()
                },
                texture: player_1_handle,
                transform: Transform::from_translation(Vec3::new(-2., 2., 0.)),
                ..default()
            },
        ))
        .add_rollback();

    // Player 2
    commands
    .spawn((
        Player { handle: 1 },
        Gravity(-9.8 * 10., false),
        CoyoteTime::new(0.125),
        Collider::new(Vec2::new((1./6.125) * 10., (1./6.125) * 10.)),
        Solid(true),
        Velocity::default(),
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(2., 2., 0.)),
            sprite: Sprite {
                custom_size: Some(Vec2::new((1./6.125) * 10., (1./6.125) * 10.)),
                ..default()
            },
            texture: player_2_handle,
            ..default()
        },
    ))
    .add_rollback();
}

fn spawn_platforms(mut commands: Commands) {
    commands
        .spawn((
            Platform,
            Collider::new(Vec2::new(BOTTOM_PLATFORM_WIDTH, BOTTOM_PLATFORM_HEIGHT)),
            Solid(false),
            Velocity::default(),
            TransformBundle::from_transform(Transform::from_xyz(
                0.,
                (BOTTOM_PLATFORM_HEIGHT / 2.) - 5.,
                0.,
            )),
        ))
        .add_rollback();

    commands
        .spawn((
            Platform,
            Collider::new(Vec2::new(MIDDLE_PLATFORM_LENGTH, MIDDLE_PLATFORM_HEIGHT)),
            Solid(false),
            Velocity::default(),
            TransformBundle::from_transform(Transform::from_xyz(
                MIDDLE_PLATFORM_POS_X,
                MIDDLE_PLATFORM_POS_Y,
                0.
            ))
        ))
        .add_rollback();

    commands
        .spawn((
            Platform,
            Collider::new(Vec2::new(TOP_PLATFORM_LENGTH, TOP_PLATFORM_HEIGHT)),
            Solid(false),
            Velocity::default(),
            TransformBundle::from_transform(Transform::from_xyz(
                TOP_PLATFORM_POS_X,
                TOP_PLATFORM_POS_Y,
                0.
            ))
        ))
        .add_rollback();
}

fn spawn_vines(mut commands: Commands) {
    commands.spawn((
        Vine,
        Collider::new(Vec2::new(VINE_WIDTH, VINE_HEIGHT)),
        Velocity::default(),
        TransformBundle::from_transform(Transform::from_xyz(VINE_POS_X, VINE_POS_Y, -0.5)),
    ))
    .add_rollback();
}

fn spawn_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background_texture = asset_server.load("lobby_background.png");

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(10., 10.)),
            ..default()
        },
        texture: background_texture,
        transform: Transform::from_xyz(0., 0., -1.),
        ..default()
    })
    .add_rollback();
}
