use bevy::prelude::*;
use bevy_ggrs::{LocalInputs, PlayerInputs};

use crate::{
    components::Player,
    input,
    physics::{self, Collider, Velocity},
    Config,
};

pub fn move_player_multiplayer(
    mut players: Query<(&mut Velocity, &Collider, &Player)>,
    inputs: Res<PlayerInputs<Config>>,
) {
    for (mut velocity, collider, player) in &mut players {
        let input = inputs.get(player.handle);

        if input.is_none() {
            continue;
        }

        let (input, _) = *input.unwrap();

        let direction = input::direction(input);

        velocity.0.x = direction.x;

        if direction.y > 0. && collider.check_colliding_side(physics::CollidingSide::Bottom) {
            velocity.0.y = direction.y;
        }
    }
}

pub fn move_player_singleplayer(
    mut players: Query<(&mut Velocity, &Collider, &Player)>,
    inputs: Res<LocalInputs<Config>>,
) {
    for (mut velocity, collider, player) in &mut players {
        let input = inputs.0.get(&player.handle);

        if input.is_none() {
            continue;
        }

        let input = *input.unwrap();

        let direction = input::direction(input);

        velocity.0.x = direction.x;

        if direction.y > 0. && collider.check_colliding_side(physics::CollidingSide::Bottom) {
            velocity.0.y = direction.y;
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
