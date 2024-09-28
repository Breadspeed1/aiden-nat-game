use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use physics::{Collider, Gravity, Velocity};

mod physics;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        .add_systems(Startup, (setup, spawn_floor, spawn_player, spawn_object))
        .add_systems(Update, (
            move_player,
            physics::handle_gravity,
            physics::handle_velocity,
            physics::handle_coliders
        ).chain())
        .run();
}

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.);
    commands.spawn(camera_bundle);
}

fn move_player(
    mut players: Query<(&mut Velocity, &Collider), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>
) {
    let mut direction = Vec2::ZERO;

    if keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction.x -= 7.;
    }

    if keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction.x += 7.;
    }

    if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::Space]) {
        direction.y += 20.;
    }

    for (mut velocity, collider) in &mut players {
        velocity.0.x = direction.x;
        
        if direction.y > 0. && collider.check_colliding_side(physics::CollidingSide::Bottom) {
            velocity.0.y = direction.y;
        }
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Gravity(-9.8 * 10.),
        Collider::new(Vec2::new(1., 1.), true),
        Velocity::default(),
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0., 0.47, 1.),
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 2., 0.)),
            ..default()
    }));
}

fn spawn_object(mut commands: Commands) {
    commands.spawn((
        Gravity(-9.8 * 10.),
        Collider::new(Vec2::new(1., 1.), true),
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
    ));
}

fn spawn_floor(mut commands: Commands) {
    commands.spawn((
        Collider::new(Vec2::new(10., 1.), false),
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
    ));
}
