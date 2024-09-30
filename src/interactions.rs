use bevy::prelude::*;
use bevy_ggrs::PlayerInputs;

use crate::{
    components::{Player, Vine},
    physics::{self, Collider, Gravity, Velocity},
    Config,
};

pub fn handle_vine_interactions(
    mut players: Query<(&mut Velocity, &Collider, &Player, &mut Gravity)>,
    vines: Query<(&Vine, Entity)>,
    inputs: Res<PlayerInputs<Config>>,
) {
    for (mut velocity, collider, player, mut gravity) in &mut players {
        let input = inputs.get(player.handle);

        if input.is_none() {
            continue;
        }

        let (input, _) = *input.unwrap();

        for (_, entity) in &vines {
            if collider.colliding_with(&entity).is_some() {
                if input & crate::input::INPUT_INTERACT != 0 {
                    velocity.0.y = 1.5;
                    gravity.temp_override();
                    continue;
                } else if !collider.check_colliding_solid_side(physics::CollidingSide::Bottom) {
                    velocity.0.y = -0.75;
                    gravity.temp_override();
                    continue;
                }
            }
        }
    }
}
