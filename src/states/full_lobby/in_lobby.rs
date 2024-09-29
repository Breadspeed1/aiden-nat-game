use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;

use crate::{physics::PhysicsSet, MultiplayerGameState};

pub struct InLobbyPlugin;

impl Plugin for InLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GgrsSchedule,
            (
                crate::movement::move_player_multiplayer,
                crate::movement::reset,
            )
                .chain()
                .in_set(InLobbySet::Update)
                .ambiguous_with_all()
                .before(PhysicsSet)
                .run_if(in_state(MultiplayerGameState::InLobby)),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(unused)]
pub enum InLobbySet {
    Setup,
    Update,
}
