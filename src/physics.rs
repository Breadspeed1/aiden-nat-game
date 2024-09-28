use bevy::{prelude::*, utils::HashSet};
use enum_ordinalize::Ordinalize;

#[derive(Component, Default, Debug)]
pub struct Velocity(pub Vec2);

impl Velocity {
    pub fn calc_pos_delta(&self, delta_time: f32) -> Vec3 {
        (self.0 * delta_time).extend(0.)
    }

    pub fn reset(&mut self) {
        self.0 *= 0.;
    }

    pub fn add(&mut self, other: Vec2) {
        self.0 += other;
    }

    pub fn set(&mut self, other: Vec2) {
        self.0 = other;
    }
}

#[derive(Component, Debug)]
pub struct Gravity(pub f32);

impl Gravity {
    pub fn calc_velo_delta(&self, delta_time: f32) -> Vec2 {
        Vec2::new(0., self.0 * delta_time)
    }
}

#[derive(Debug, Ordinalize)]
pub enum CollidingSide {
    Top,
    Bottom,
    Left,
    Right
}

#[derive(Component, Debug)]
pub struct Collider {
    bounding_box: Vec2,
    movable: bool,
    colliding: u8
}

impl Collider {
    pub fn new(bounding_box: Vec2, movable: bool) -> Self {
        Self {
            bounding_box: bounding_box.abs(),
            movable,
            colliding: 0u8
        }
    }
    
    pub fn check_colliding_side(&self, side: CollidingSide) -> bool {
        let mask = 1u8 << side.ordinal();
        self.colliding & mask > 0
    }

    pub fn set_colliding_side(&mut self, side: CollidingSide) {
        let mask = 1u8 << side.ordinal();
        self.colliding |= mask;
    }

    pub fn clear_colliding_side(&mut self) {
        self.colliding = 0u8;
    }

    fn check_is_colliding(&self, other: &Self, distance_x: f32, distance_y: f32) -> bool {
        self.bounding_box.x / 2. + other.bounding_box.x / 2. > distance_x && self.bounding_box.y / 2. + other.bounding_box.y / 2. > distance_y
    }
}

pub fn handle_velocity(time: Res<Time>, mut objects: Query<(&Velocity, &mut Transform)>) {
    let delta = time.delta_seconds();

    for (velocity, mut transform) in &mut objects {
        transform.translation += velocity.calc_pos_delta(delta);
    }
}

pub fn handle_gravity(time: Res<Time>, mut objects: Query<(&Gravity, &mut Velocity, &Collider)>) {
    let delta = time.delta_seconds();

    for (gravity, mut velocity, collider) in &mut objects {
        if collider.check_colliding_side(CollidingSide::Bottom) {
            velocity.0.y = velocity.0.y.max(-0.1);
            continue;
        }

        velocity.add(gravity.calc_velo_delta(delta));
    }
}

pub fn handle_coliders(mut objects: Query<(&mut Velocity, &mut Transform, &mut Collider)>) {
    for (_, _, mut collider) in &mut objects {
        collider.clear_colliding_side();
    }

    let mut iter = objects.iter_combinations_mut();

    while let Some([(mut v1, mut t1, mut c1), (mut v2, mut t2, mut c2)]) = iter.fetch_next() {
        let distance = t2.translation - t1.translation;
        let colliding = c1.check_is_colliding(c2.as_ref(), distance.x.abs(), distance.y.abs());

        if colliding {
            let edge_distance = distance.xy().abs() - (c1.bounding_box + c2.bounding_box) / 2.;
            let x_min = edge_distance.x.abs() < edge_distance.y.abs();

            let move_amount = if x_min {
                edge_distance.with_y(0.).copysign(-distance.xy())
            }
            else {
                edge_distance.with_x(0.).copysign(-distance.xy())
            }.extend(0.);

            if move_amount.x < 0. {
                c1.set_colliding_side(CollidingSide::Right);
                c2.set_colliding_side(CollidingSide::Left);
            }
            else if move_amount.x > 0. {
                c1.set_colliding_side(CollidingSide::Left);
                c2.set_colliding_side(CollidingSide::Right);
            }
            else if move_amount.y < 0. {
                c1.set_colliding_side(CollidingSide::Top);
                c2.set_colliding_side(CollidingSide::Bottom);
            }
            else {
                c1.set_colliding_side(CollidingSide::Bottom);
                c2.set_colliding_side(CollidingSide::Top);
            }

            match (c1.movable, c2.movable) {
                (true, true) => {
                    t2.translation -= move_amount / 2.;
                    t1.translation += move_amount / 2.;

                    if x_min {
                        v2.0.x = 0.;
                        v1.0.x = 0.;
                    }
                    else {
                        v2.0.y = 0.;
                        v1.0.y = 0.;
                    }
                },
                (true, false) => {
                    t1.translation += move_amount;

                    if x_min {
                        v1.0.x = 0.;
                    }
                    else {
                        v1.0.y = 0.;
                    }
                },
                (false, true) => {
                    t2.translation -= move_amount;

                    if x_min {
                        v2.0.x = 0.;
                    }
                    else {
                        v2.0.y = 0.;
                    }
                },
                (false, false) => (),
            }
        }
    }
}
