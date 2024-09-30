use bevy::prelude::Component;

#[derive(Component, Clone, Copy, Debug)]
pub struct Player {
    pub handle: usize,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Vine;

#[derive(Component, Clone, Copy, Debug)]
pub struct Platform;
