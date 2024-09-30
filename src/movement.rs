use bevy::prelude::*;
use bevy_ggrs::PlayerInputs;

use crate::{
    components::{CoyoteTime, Player},
    input,
    physics::Velocity,
    Config,
};

pub fn move_player_multiplayer(
    mut players: Query<(&mut Velocity, &mut CoyoteTime, &Player, &mut Sprite)>,
    inputs: Res<PlayerInputs<Config>>,
) {
    for (mut velocity, mut ct, player, mut sprite) in &mut players {
        let input = inputs.get(player.handle);

        if input.is_none() {
            continue;
        }

        let (input, _) = *input.unwrap();

        let direction = input::direction(input);

        velocity.0.x = direction.x;

        if direction.x < 0. {
            sprite.flip_x = true;
        } else if direction.x > 0. {
            sprite.flip_x = false;
        }

        if direction.y <= 0. {
            continue;
        }

        if ct.get() {
            velocity.0.y = direction.y;
            ct.clear();
            continue;
        }
    }
}

pub fn reset(mut objects: Query<(&mut Transform, &mut Velocity)>) {
    for (mut t, mut v) in &mut objects {
        if t.translation.y < -5. {
            t.translation = Vec3::new(0., 3., 0.);
            v.0 = Vec2::ZERO;
        }
    }
}
