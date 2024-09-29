use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_matchbox::{prelude::SingleChannel, MatchboxSocket};
use components::Player;
use physics::{Collider, Gravity, PhysicsPlugin, Solid, Velocity};
use bevy_ggrs::*;
use bevy_matchbox::matchbox_socket::PeerId;

mod physics;
mod input;
mod components;

type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }),
        GgrsPlugin::<Config>::default(),
    ))
        .add_plugins(PhysicsPlugin)
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        .add_systems(Startup, (setup, spawn_floor, spawn_player, spawn_object, start_matchbox_socket))
        .add_systems(Update, wait_for_players)
        .add_systems(ReadInputs, input::read_local_inputs)
        .add_systems(GgrsSchedule, (move_player, reset)
            .chain()
            .before(physics::handle_gravity)
            .before(physics::handle_velocity)
            .before(physics::handle_colliders)
            .before(physics::handle_solids)
        )
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_copy::<Player>()
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.);
    commands.spawn(camera_bundle);
}

fn move_player(
    mut players: Query<(&mut Velocity, &Collider, &Player)>,
    inputs: Res<PlayerInputs<Config>>
) {
    for (mut velocity, collider, player) in &mut players {
        let (input, _) = inputs[player.handle];

        let direction = input::direction(input);

        velocity.0.x = direction.x;
        
        if direction.y > 0. && collider.check_colliding_side(physics::CollidingSide::Bottom) {
            velocity.0.y = direction.y;
        }
    }
}

fn reset(
    mut objects: Query<(&mut Transform, &mut Velocity)>,
) {
    for (mut t, mut v) in &mut objects {
        if t.translation.y < -5. {
            t.translation = Vec3::new(0., 3., 0.);
            v.0 = Vec2::ZERO;
        }
    }
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/aidennat?next=2";
    info!("Connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
}

fn wait_for_players(mut commands: Commands, mut socket: ResMut<MatchboxSocket<SingleChannel>>) {
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
}

fn spawn_player(mut commands: Commands) {
    // Player 1
    commands.spawn((
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
    }))
    .add_rollback();

    // Player 2
    commands
        .spawn((
            Player {handle: 1 },
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
    commands.spawn((
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
        }
    ))
    .add_rollback();
}

fn spawn_floor(mut commands: Commands) {
    commands.spawn((
        Collider::new(Vec2::new(10., 1.)),
        Solid(false),
        Velocity::default(),
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1., 1., 1.,),
                custom_size: Some(Vec2::new(10., 1.)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., -2., 0.)),
            ..default()
        }
    ))
    .add_rollback();
}
