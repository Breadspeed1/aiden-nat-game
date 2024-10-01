use std::time::Duration;

use crate::components::{CoyoteTime, Platform, Player, Vine};
use crate::physics::{Collider, Gravity, PhysicsSet, Solid, Velocity};
use bevy::prelude::*;
use bevy_ggrs::ggrs::P2PSession;
use bevy_ggrs::{ggrs, AddRollbackCommandExtension};
use bevy_matchbox::matchbox_socket::WebRtcChannel;
use bevy_matchbox::prelude::{PeerId, SingleChannel};
use bevy_matchbox::MatchboxSocket;

use crate::{despawn_all_but_camera, AppState, Config};

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

const CONNECTION_TIMEOUT: f32 = 10.;

const CONFIG_BEARER: u8 = 0;
const CONFIG_RECEIVER: u8 = 1;

const ROLE: u8 = 0;
const CONFIG: u8 = 1;
const OK: u8 = 2;

pub struct WaitingLobbyPlugin;

impl Plugin for WaitingLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::WaitingInLobby),
            (
                despawn_all_but_camera,
                spawn_background,
                spawn_platforms,
                spawn_player,
                spawn_vines,
                start_connection_manager,
            )
                .chain()
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

 

enum ConnectionManagerState {
    PreConnect,
    WaitingOnMetaConnection(MatchboxSocket<SingleChannel>),
    WaitingOnMetadata(WebRtcChannel, MetaConnectionState, PeerId),
    WaitingOnGGRSConnetion(MatchboxSocket<SingleChannel>, GameConfig),
    Ready(GameConfig, Option<P2PSession<Config>>),
    TimedOut,
    InvalidConnection
}

#[derive(Debug, Resource, Clone, Copy)]
pub enum CMRole {
    ConfigBearer(GameConfig),
    ConfigReciever
}

#[derive(Debug, Clone, Copy, Resource)]
pub struct GameConfig {
    pub seed: u32,
    pub difficulty: u32
}

impl GameConfig {
    pub fn from_u64(data: u64) -> Self {
        Self {
            seed: (data & u32::MAX as u64) as u32,
            difficulty: ((data >> 32) & u32::MAX as u64) as u32
        }
    }

    pub fn as_u64(&self) -> u64 {
        self.seed as u64 | ((self.difficulty as u64) << 32)
    }
}

#[derive(Resource)]
pub struct ConnectionManager {
    address: String,
    role: CMRole,
    state: ConnectionManagerState,
    timeout_timer: Timer
}

#[derive(Debug)]
enum MetaConnectionState {
    Start,
    WaitingOnRole,
    WaitingOnOK(GameConfig),
    WaitingOnConfig
}

impl ConnectionManager {
    fn new(address: String, room_id: u32, role: CMRole) -> Self {
        Self {
            address: format!("ws://{address}/{room_id}?next=2"),
            role,
            state: ConnectionManagerState::PreConnect,
            timeout_timer: Timer::from_seconds(CONNECTION_TIMEOUT, TimerMode::Once)
        }
    }

    fn start(&mut self) {
        if matches!(self.state, ConnectionManagerState::PreConnect) {
            let meta_conn = MatchboxSocket::new_reliable(&self.address);
            self.state = ConnectionManagerState::WaitingOnMetaConnection(meta_conn);
            info!("Connection manager transitioning to waiting on metadata connection");
        }
        else {
            warn!("Start called on already started connection manager");
        }
    }

    fn get_state(&mut self) -> &mut ConnectionManagerState {
        &mut self.state
    }

    fn check_advance(&mut self, dur: Duration) {
        self.timeout_timer.tick(dur);

        let address = self.address.clone();

        if self.timeout_timer.finished() && !matches!(self.state, ConnectionManagerState::Ready(_, _) | ConnectionManagerState::WaitingOnMetaConnection(_)) {
            self.state = ConnectionManagerState::TimedOut;
            return;
        }

        match &mut self.state {
            ConnectionManagerState::WaitingOnMetaConnection(matchbox_socket) => {
                matchbox_socket.update_peers();
                let peers = matchbox_socket.players().len();

                if peers == 2 {
                    let id = matchbox_socket.connected_peers().next().unwrap();
                    self.state = ConnectionManagerState::WaitingOnMetadata(matchbox_socket.take_channel(0).unwrap(), MetaConnectionState::Start, id);
                    info!("Connection manager transitioning to waiting on metadata")
                }
            },
            ConnectionManagerState::WaitingOnMetadata(channel, state, id) => {
                match state {
                    MetaConnectionState::Start => {
                        match self.role {
                            CMRole::ConfigBearer(_) => channel.send(Box::new([ROLE, CONFIG_BEARER]), *id),
                            CMRole::ConfigReciever => channel.send(Box::new([ROLE, CONFIG_RECEIVER]), *id),
                        }

                        info!("Connection manager transitioning to waiting on role");
                        *state = MetaConnectionState::WaitingOnRole;
                    },
                    MetaConnectionState::WaitingOnRole => {
                        let messages = channel.receive();

                        if messages.len() > 1 {
                            error!("Received too many meta responses while waiting on role");
                            self.state = ConnectionManagerState::InvalidConnection;
                            return;
                        }

                        if messages.len() < 1 {
                            return;
                        }

                        match self.role {
                            CMRole::ConfigBearer(game_config) => {
                                #[allow(unused_allocation)]
                                if messages[0].1 != Box::new([ROLE, CONFIG_RECEIVER]) {
                                    error!("Received invalid meta response");
                                    self.state = ConnectionManagerState::InvalidConnection;
                                    return;
                                }

                                channel.send([&[CONFIG][..], &game_config.as_u64().to_be_bytes()[..]].concat().into(), *id);

                                info!("Connection manager transitioning to waiting on OK");
                                *state = MetaConnectionState::WaitingOnOK(game_config);
                            },
                            CMRole::ConfigReciever => {
                                #[allow(unused_allocation)]
                                if messages[0].1 != Box::new([ROLE, CONFIG_BEARER]) {
                                    error!("Received invalid meta response");
                                    self.state = ConnectionManagerState::InvalidConnection;
                                    return;
                                }

                                info!("Connection manager transitioning to waiting on config");
                                *state = MetaConnectionState::WaitingOnConfig;
                            },
                        }
                    }
                    MetaConnectionState::WaitingOnOK(config) => {
                        let messages = channel.receive();

                        if messages.len() > 1 {
                            self.state = ConnectionManagerState::InvalidConnection;
                            error!("Received too many messages while waiting on OK");
                            return;
                        }

                        if messages.len() < 1 {
                            return;
                        }

                        #[allow(unused_allocation)]
                        if messages[0].1 != Box::new([OK]) {
                            self.state = ConnectionManagerState::InvalidConnection;
                            error!("Received invalid meta response while waiting on OK");
                            return;
                        }

                        info!("Received OK, closing meta channel");
                        channel.close();

                        let socket = MatchboxSocket::new_ggrs(address);
                        self.state = ConnectionManagerState::WaitingOnGGRSConnetion(socket, *config);

                        info!("Connection manager transitioning to waiting on ggrs connection");
                    },
                    MetaConnectionState::WaitingOnConfig => {
                        let messages = channel.receive();

                        if messages.len() > 1 {
                            self.state = ConnectionManagerState::InvalidConnection;
                            error!("Received too many messages while waiting on Config");
                            return;
                        }

                        if messages.len() < 1 {
                            return;
                        }

                        #[allow(unused_allocation)]
                        if messages[0].1[0] != CONFIG {
                            self.state = ConnectionManagerState::InvalidConnection;
                            error!("Received invalid meta response while waiting on Config");
                            return;
                        }

                        let config_value: Result<[u8; 8], _> = messages[0].1[1..].try_into();

                        if config_value.is_err() {
                            self.state = ConnectionManagerState::InvalidConnection;
                            error!("received invalid meta response while waiting on Config");
                            return;
                        }

                        let config_value = config_value.unwrap();

                        let config = GameConfig::from_u64(u64::from_be_bytes(config_value));

                        let socket = MatchboxSocket::new_ggrs(address);
                        self.state = ConnectionManagerState::WaitingOnGGRSConnetion(socket, config);

                        info!("Connection manager transitioning to waiting on ggrs connection");
                    },
                }
            },
            ConnectionManagerState::WaitingOnGGRSConnetion(socket, game_config) => {
                socket.update_peers();
                let players = socket.players();

                if players.len() < 2 {
                    return;
                }

                let mut session_builder = ggrs::SessionBuilder::<Config>::new()
                .with_num_players(2)
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

                self.state = ConnectionManagerState::Ready(*game_config, Some(ggrs_session));

                info!("Connection manager transitioing to ready");
            },
            _ => {
                info!("Connection manager finished");
            }
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum WaitingLobbySet {
    Setup,
    Update,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct RoomID(pub u32);

fn start_connection_manager(mut commands: Commands, room_id: Res<RoomID>, role: Res<CMRole>) {
    let mut cm = ConnectionManager::new(
        "3.128.79.14:3536".into(),
        room_id.0,
        *role.into_inner()
    );

    cm.start();

    commands.insert_resource(cm);
}


fn wait_for_players(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut connection_manager: ResMut<ConnectionManager>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
        return;
    }

    connection_manager.check_advance(time.delta());

    match connection_manager.get_state() {
        ConnectionManagerState::Ready(conf, session) => {
            commands.insert_resource(bevy_ggrs::Session::P2P(session.take().unwrap()));
            commands.insert_resource(*conf);
            info!("Connection manager finished, entering full lobby");
            next_state.set(AppState::FullLobby);
        },
        ConnectionManagerState::InvalidConnection | ConnectionManagerState::TimedOut => {
            error!("Connection manager had invalid conneciton or timed out");
            next_state.set(AppState::MainMenu)
        },
        _ => {}
    }
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
            Collider::new(Vec2::new((1. / 6.125) * 10., (1. / 6.125) * 10.)),
            Solid(true),
            Velocity::default(),
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(2., 2., 0.)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new((1. / 6.125) * 10., (1. / 6.125) * 10.)),
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
                0.,
            )),
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
                0.,
            )),
        ))
        .add_rollback();
}

fn spawn_vines(mut commands: Commands) {
    commands
        .spawn((
            Vine,
            Collider::new(Vec2::new(VINE_WIDTH, VINE_HEIGHT)),
            Velocity::default(),
            TransformBundle::from_transform(Transform::from_xyz(VINE_POS_X, VINE_POS_Y, -0.5)),
        ))
        .add_rollback();
}

fn spawn_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background_texture = asset_server.load("lobby_background.png");

    commands
        .spawn(SpriteBundle {
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
