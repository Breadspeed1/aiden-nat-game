use bevy::{prelude::*, utils::HashSet};
use bevy_ggrs::{GgrsApp, GgrsSchedule};
use enum_ordinalize::Ordinalize;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(GgrsSchedule, (
                handle_gravity,
                handle_velocity,
                handle_colliders,
                handle_solids
            ).chain())
            .rollback_component_with_copy::<Velocity>()
            .rollback_component_with_copy::<Gravity>()
            .rollback_component_with_copy::<Solid>()
            .rollback_component_with_clone::<Collider>();
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

impl Velocity {
    pub fn calc_pos_delta(&self, delta_time: f32) -> Vec3 {
        (self.0 * delta_time).extend(0.)
    }

    pub fn add(&mut self, other: Vec2) {
        self.0 += other;
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Gravity(pub f32);

impl Gravity {
    pub fn calc_velo_delta(&self, delta_time: f32) -> Vec2 {
        Vec2::new(0., self.0 * delta_time)
    }
}

#[derive(Debug, Ordinalize, Clone, Copy)]
pub enum CollidingSide {
    Top,
    Bottom,
    Left,
    Right
}

impl CollidingSide {
    pub fn opposite(&self) -> Self {
        match self {
            CollidingSide::Top => CollidingSide::Bottom,
            CollidingSide::Bottom => CollidingSide::Top,
            CollidingSide::Left => CollidingSide::Right,
            CollidingSide::Right => CollidingSide::Left,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Collider {
    bounding_box: Vec2,
    collisions: Vec<(Entity, CollidingSide, f32)>,
    colliding_side: u8
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Solid(pub bool);

impl Collider {
    pub fn new(bounding_box: Vec2) -> Self {
        Self {
            bounding_box: bounding_box.abs(),
            collisions: Vec::new(),
            colliding_side: 0
        }
    }
    
    pub fn check_colliding_side(&self, side: CollidingSide) -> bool {
        self.colliding_side & (1u8 << side.ordinal()) > 0
    }

    pub fn add_collision(&mut self, entity: Entity, colliding_side: CollidingSide, overlap: f32) {
        self.collisions.push((entity, colliding_side, overlap));
        self.colliding_side |= 1u8 << colliding_side.ordinal();
    }

    pub fn clear_collisions(&mut self) {
        self.colliding_side = 0;
        self.collisions.clear();
    }

    pub fn colliding_with(&self, entity: &Entity) -> Option<(CollidingSide, f32)> {
        self.collisions
        .iter()
        .find(|(e, _, _)| e == entity)
        .map(|(_, side, overlap)| (*side, *overlap))
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
            velocity.0.y = velocity.0.y.max(-0.01);
            continue;
        }

        velocity.add(gravity.calc_velo_delta(delta));
    }
}

pub fn handle_solids(mut objects: Query<(Entity, &mut Transform, &Collider, &Solid)>) {
    let mut handled_collisions: HashSet<(Entity, Entity)> = HashSet::new();
    let mut iter = objects.iter_combinations_mut();

    while let Some([(e1, mut t1, c1, s1), (e2, mut t2, c2, s2)]) = iter.fetch_next()  {
        let collision = c1.colliding_with(&e2);

        if collision.is_none() {
            continue;
        }

        if handled_collisions.contains(&(e1, e2)) {
            continue;
        }

        let (side, overlap) = collision.unwrap();

        let movement = match side {
            CollidingSide::Top => Vec2::new(0., -overlap),
            CollidingSide::Bottom => Vec2::new(0., overlap),
            CollidingSide::Left => Vec2::new(overlap, 0.),
            CollidingSide::Right => Vec2::new(-overlap, 0.),
        }.extend(0.);

        match (s1.0, s2.0, c1.check_colliding_side(side.opposite()), c2.check_colliding_side(side)) {
            (true, true, false, false) => {
                t2.translation -= movement / 2.;
                t1.translation += movement / 2.;
            },
            (true, false, false, _) | (true, true, false, true) => {
                t1.translation += movement;

            },
            (false, true, _, true) | (true, true, true, false) => {
                t2.translation -= movement;

            },
            _ => (),
        }

        handled_collisions.insert((e1, e2));
        handled_collisions.insert((e2, e1));
    }
}

pub fn handle_colliders(mut objects: Query<(Entity, &Transform, &mut Collider)>) {
    for (_, _, mut collider) in &mut objects {
        collider.clear_collisions();
    }

    let mut iter = objects.iter_combinations_mut();

    while let Some([(e1, t1, mut c1), (e2, t2, mut c2)]) = iter.fetch_next() {
        let diff = t1.translation.xy() - t2.translation.xy();
        let edge_distance = diff.abs() - (c1.bounding_box + c2.bounding_box) / 2.;

        if edge_distance.max_element() > 0. {
            continue;
        }

        let collisions = if edge_distance.x > edge_distance.y {
            if diff.x < 0. {
                (CollidingSide::Right, CollidingSide::Left)
            }
            else {
                (CollidingSide::Left, CollidingSide::Right)
            }
        }
        else {
            if diff.y < 0. {
                (CollidingSide::Top, CollidingSide::Bottom)
            }
            else {
                (CollidingSide::Bottom, CollidingSide::Top)
            }
        };

        let max = edge_distance.max_element().abs();

        c1.add_collision(e2, collisions.0, max);
        c2.add_collision(e1, collisions.1, max);
    }
}
