use leafwing_input_manager::{prelude::{ActionState, InputMap}, Actionlike, InputManagerBundle};
use lightyear::{prelude::*, utils::avian2d::{position, rotation}};
use bevy::prelude::*;
use avian2d::prelude::*;

pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::new_id(1);
pub const PLAYER_SIZE: (f32, f32) = (20., 30.);

#[derive(Bundle)]
pub struct PlayerBundle {
    pub id: PlayerId,
    pub position: Position,
    pub replicate: client::Replicate,
    pub physics: PhysicsBundle,
    pub inputs: InputManagerBundle<PlayerActions>,
    pub pre_predicted: PrePredicted,
}

impl PlayerBundle {
    pub fn new(id: ClientId, position: Vec2, input_map: InputMap<PlayerActions>) -> Self {
        Self {
            id: PlayerId(id),
            position: Position(position),
            replicate: client::Replicate {
                group: REPLICATION_GROUP,
                ..default()
            },
            physics: PhysicsBundle::player(),
            inputs: InputManagerBundle::<PlayerActions> {
                action_state: ActionState::default(),
                input_map,
            },
            pre_predicted: PrePredicted::default(),
        }
    }
}

#[derive(Bundle)]
pub struct PhysicsBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub locked_axis: LockedAxes,
}

impl PhysicsBundle {
    fn player() -> Self {
        Self {
            collider: Collider::rectangle(PLAYER_SIZE.0, PLAYER_SIZE.1),
            rigid_body: RigidBody::Dynamic,
            locked_axis: LockedAxes::ROTATION_LOCKED
        }
    }
}

#[derive(Actionlike, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
enum PlayerActions {
    Jump,
    Left,
    Right,
    Interact
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
struct PlayerId(ClientId);

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LeafwingInputPlugin::<PlayerActions>::default());

        app.register_component::<PlayerId>(ChannelDirection::Bidirectional)
            .add_prediction(client::ComponentSyncMode::Once)
            .add_interpolation(client::ComponentSyncMode::Once);

        app.register_component::<Position>(ChannelDirection::Bidirectional)
            .add_prediction(client::ComponentSyncMode::Full)
            .add_interpolation(client::ComponentSyncMode::Full)
            .add_interpolation_fn(position::lerp)
            .add_correction_fn(position::lerp);

        app.register_component::<Rotation>(ChannelDirection::Bidirectional)
            .add_prediction(client::ComponentSyncMode::Full)
            .add_interpolation(client::ComponentSyncMode::Full)
            .add_interpolation_fn(rotation::lerp)
            .add_correction_fn(rotation::lerp);

        app.register_component::<LinearVelocity>(ChannelDirection::Bidirectional)
            .add_prediction(client::ComponentSyncMode::Full);

        app.register_component::<AngularVelocity>(ChannelDirection::Bidirectional)
            .add_prediction(client::ComponentSyncMode::Full);
    }
}