use bevy::{prelude::*, utils::HashMap};
use bevy_ggrs::{LocalInputs, LocalPlayers};

use crate::{resources::WindowScale, MIN_WINDOW_SIZE};

pub const INPUT_LEFT: u8 = 1 << 0;
pub const INPUT_RIGHT: u8 = 1 << 1;
pub const INPUT_JUMP: u8 = 1 << 2;
pub const INPUT_INTERACT: u8 = 1 << 3;

pub fn read_local_inputs(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    local_players: Res<LocalPlayers>,
) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let mut input = 0u8;

        if keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
            input |= INPUT_LEFT
        }
        if keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
            input |= INPUT_RIGHT;
        }
        if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
            input |= INPUT_JUMP;
        }
        if keys.any_pressed([KeyCode::Space, KeyCode::KeyE]) {
            input |= INPUT_INTERACT;
        }

        local_inputs.insert(*handle, input);
    }

    commands.insert_resource(LocalInputs::<super::Config>(local_inputs));
}

pub fn handle_window_resize(
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    mut resolution_settings: ResMut<WindowScale>,
) {
    let mut window = windows.single_mut();

    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::Equal) {
        let scale = resolution_settings.increase();
        window.resolution.set(
            MIN_WINDOW_SIZE * scale as f32,
            MIN_WINDOW_SIZE * scale as f32,
        )
    }

    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::Minus) {
        let scale = resolution_settings.decrease();
        window.resolution.set(
            MIN_WINDOW_SIZE * scale as f32,
            MIN_WINDOW_SIZE * scale as f32,
        )
    }
}

pub fn direction(input: u8) -> Vec2 {
    let mut direction = Vec2::ZERO;

    if input & INPUT_LEFT != 0 {
        direction.x -= 4.;
    }

    if input & INPUT_RIGHT != 0 {
        direction.x += 4.;
    }

    if input & INPUT_JUMP != 0 {
        direction.y += 15.;
    }

    direction
}
