use bevy::{prelude::*, utils::HashMap};
use bevy_ggrs::{LocalInputs, LocalPlayers};

const INPUT_LEFT: u8 = 1 << 0;
const INPUT_RIGHT: u8 = 1 << 1;
const INPUT_JUMP: u8 = 1 << 2;

pub fn read_local_inputs(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    local_players: Res<LocalPlayers>
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
        if keys.any_pressed([KeyCode::Space, KeyCode::ArrowUp]) {
            input |= INPUT_JUMP;
        }

        local_inputs.insert(*handle, input);
    }

    commands.insert_resource(LocalInputs::<super::Config>(local_inputs));
}

pub fn direction(input: u8) -> Vec2 {
    let mut direction = Vec2::ZERO;

    if input & INPUT_LEFT != 0 {
        direction.x -= 7.;
    }

    if input & INPUT_RIGHT != 0 {
        direction.x += 7.;
    }

    if input & INPUT_JUMP != 0 {
        direction.y += 20.;
    }

    direction
}