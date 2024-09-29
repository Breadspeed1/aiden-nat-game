pub mod in_lobby;

use bevy::prelude::*;
use bevy_ggrs::Session;

use crate::{AppState, Config, DespawnAllButCameraID, MultiplayerGameState};

pub struct FullLobbyPlugin;

impl Plugin for FullLobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::FullLobby), start_ggrs_state)
            .add_systems(Update, handle_ggrs_events.run_if(in_state(AppState::FullLobby)))
            .add_plugins(in_lobby::InLobbyPlugin);
    }
}

fn start_ggrs_state(mut multiplayer_state: ResMut<NextState<MultiplayerGameState>>) {
    multiplayer_state.set(MultiplayerGameState::InLobby);
    info!("Starting multiplayer schedule");
}

fn handle_ggrs_events(mut commands: Commands, mut session: ResMut<Session<Config>>, abc_id: Res<DespawnAllButCameraID>, mut next_state: ResMut<NextState<AppState>>) {
    match session.as_mut() {
        Session::P2P(s) => {
            for event in s.events() {
                match event {
                    bevy_ggrs::ggrs::GgrsEvent::Disconnected { ..  } => {
                        error!("Disconnect, sending back to main menu");

                        commands.run_system(abc_id.0);
                        next_state.set(AppState::MainMenu);

                    },
                    bevy_ggrs::ggrs::GgrsEvent::DesyncDetected { frame, local_checksum, remote_checksum, .. } => {
                        error!("Desync on frame {frame}. Local checksum: {local_checksum:X}, remote checksum: {remote_checksum:X}");
                    },
                    _ => {}
                }
            }
        },
        _ => {}
    }
}