use bevy::prelude::Component;

#[derive(Component, Clone, Copy)]
pub struct Player {
    pub handle: usize,
}
