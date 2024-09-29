pub mod in_lobby;

use bevy::prelude::*;

use crate::{AppState, MultiplayerGameState};

pub struct FullLobbyPlugin;

impl Plugin for FullLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::FullLobby), start_ggrs_state)
            .add_plugins(in_lobby::InLobbyPlugin);
    }
}

fn start_ggrs_state(mut multiplayer_state: ResMut<NextState<MultiplayerGameState>>) {
    multiplayer_state.set(MultiplayerGameState::InLobby);
    info!("Starting multiplayer schedule");
}
