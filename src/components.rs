use std::time::Duration;

use bevy::{prelude::*, time::Timer};

use crate::physics::Collider;

#[derive(Component, Clone, Copy, Debug)]
pub struct Player {
    pub handle: usize,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Vine;

#[derive(Component, Clone, Copy, Debug)]
pub struct Platform;

#[derive(Component, Clone, Debug)]
pub struct CoyoteTime {
    timer: Timer,
}

impl CoyoteTime {
    pub fn new(buffer_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(buffer_secs, bevy::time::TimerMode::Once),
        }
    }

    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
    }

    pub fn set_on_platform(&mut self) {
        self.timer.reset();
    }

    pub fn get(&self) -> bool {
        !self.timer.finished()
    }

    pub fn clear(&mut self) {
        let d = self.timer.duration();
        self.timer.tick(d.mul_f32(2.));
    }
}

pub fn handle_coyote_time(
    mut query: Query<(&mut CoyoteTime, &Collider), With<Player>>,
    platforms: Query<&Platform>,
    time: Res<Time>,
) {
    for (mut ct, collider) in &mut query {
        ct.tick(time.delta());

        for entity in collider.get_all_colliding_side(crate::physics::CollidingSide::Bottom) {
            if platforms.contains(entity) {
                ct.set_on_platform();
                continue;
            }
        }
    }
}
