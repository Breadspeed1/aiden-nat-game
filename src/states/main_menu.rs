use std::{cmp::Ordering, time::Duration};

use bevy::prelude::*;

use crate::{despawn_all_but_camera, AppState};

const MENU_BUTTONS: usize = 4;
const MENU_BUTTON_MOVE_TIME: f32 = 0.25;
const IN_POS: f32 = (30. / 196.) * 10.;
const MOVE_AMOUNT: f32 = (10. / 196.) * 10.5;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::MainMenu),
            (
                despawn_all_but_camera,
                setup
            ).chain().in_set(MainMenuSet::Setup),
        )
        .add_systems(
            Update,
            (read_keyboard, handle_button_select, handle_buttons)
                .chain()
                .in_set(MainMenuSet::Update),
        )
        .configure_sets(
            Update,
            MainMenuSet::Update.run_if(in_state(AppState::MainMenu)),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(unused)]
pub enum MainMenuSet {
    Setup,
    Update,
}

#[derive(Debug, Resource, Clone, Copy)]
struct MainMenuManager {
    selected_button: usize,
}

impl MainMenuManager {
    fn new() -> Self {
        Self { selected_button: 0 }
    }

    fn increment(&mut self) {
        match self.selected_button.cmp(&(MENU_BUTTONS - 1)) {
            Ordering::Less => self.selected_button += 1,
            _ => self.selected_button = 0,
        }
    }

    fn decrement(&mut self) {
        match self.selected_button.cmp(&0) {
            Ordering::Greater => self.selected_button -= 1,
            _ => self.selected_button = MENU_BUTTONS - 1,
        }
    }

    fn get(&self) -> usize {
        self.selected_button
    }

    fn get_state_for_selected(&self) -> Option<AppState> {
        match self.selected_button {
            //TODO: change
            0 => Some(AppState::CreateGameMenu),
            1 => Some(AppState::JoinGameMenu),
            2 => None,
            3 => None,
            _ => None
        }
    }
}

#[derive(Component, Debug)]
struct MainMenuButton {
    id: usize,
    in_xpos: f32,
    out_xpos: f32,
    focus_timer: Timer,
    selected: bool,
}

impl MainMenuButton {
    fn new(id: usize, in_xpos: f32, out_xpos: f32) -> Self {
        let mut focus_timer = Timer::from_seconds(MENU_BUTTON_MOVE_TIME, TimerMode::Once);
        focus_timer.tick(Duration::from_secs_f32(MENU_BUTTON_MOVE_TIME * 2.));

        Self {
            id,
            in_xpos,
            out_xpos,
            focus_timer,
            selected: false,
        }
    }

    fn tick(&mut self, dur: Duration) -> f32 {
        self.focus_timer.tick(dur);

        match self.selected {
            true => {
                if self.focus_timer.finished() {
                    self.out_xpos
                } else {
                    self.in_xpos
                        .lerp(self.out_xpos, self.focus_timer.fraction())
                }
            }
            false => {
                if self.focus_timer.finished() {
                    self.in_xpos
                } else {
                    self.out_xpos
                        .lerp(self.in_xpos, (self.focus_timer.fraction() * 2.).min(1.))
                }
            }
        }
    }

    fn set_selected(&mut self, val: bool) {
        if val != self.selected {
            let elapsed = self.focus_timer.elapsed();
            self.focus_timer.reset();
            if val {
                self.focus_timer.tick(self.focus_timer.duration() - elapsed);
            }
            else {
                self.focus_timer.tick(self.focus_timer.duration() - elapsed);
            }
            self.selected = val;
        }
    }
}

fn setup(mut commands: Commands, am: Res<AssetServer>) {
    commands.insert_resource(MainMenuManager::new());

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(10., 10.)),
            ..default()
        },
        texture: am.load("menu/menu_bg.png"),
        transform: Transform::from_xyz(0., 0., -1.),
        ..default()
    });

    commands.spawn((
        MainMenuButton::new(3, IN_POS, IN_POS - MOVE_AMOUNT),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10., 10.)),
                ..default()
            },
            texture: am.load("menu/menu_pf_4.png"),
            ..default()
        },
    )).with_children(|parent| {
        parent
            .spawn(
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(10., 10.)),
                        color: Color::hsl(200.0, 0.5, 0.5),
                        ..default()
                    },
                    texture: am.load("menu/menu_pf_4.png"),
                    transform: Transform::from_xyz(0., 0., -0.1),
                    ..default()
                }
            ); 
    });

    commands.spawn((
        MainMenuButton::new(2, -IN_POS, -IN_POS + MOVE_AMOUNT),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10., 10.)),
                ..default()
            },
            texture: am.load("menu/menu_pf_3.png"),
            ..default()
        },
    )).with_children(|parent| {
        parent
            .spawn(
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(10., 10.)),
                        color: Color::hsl(200.0, 0.5, 0.5),
                        ..default()
                    },
                    texture: am.load("menu/menu_pf_3.png"),
                    transform: Transform::from_xyz(0., 0., -0.1),
                    ..default()
                }
            ); 
    });

    commands.spawn((
        MainMenuButton::new(1, IN_POS, IN_POS - MOVE_AMOUNT),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10., 10.)),
                ..default()
            },
            texture: am.load("menu/menu_pf_2.png"),
            ..default()
        },
    )).with_children(|parent| {
        parent
            .spawn(
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(10., 10.)),
                        color: Color::hsl(200.0, 0.5, 0.5),
                        ..default()
                    },
                    texture: am.load("menu/menu_pf_2.png"),
                    transform: Transform::from_xyz(0., 0., -0.1),
                    ..default()
                }
            ); 
    });

    commands.spawn((
        MainMenuButton::new(0, -IN_POS, -IN_POS + MOVE_AMOUNT),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10., 10.)),
                ..default()
            },
            texture: am.load("menu/menu_pf_1.png"),
            ..default()
        },
    )).with_children(|parent| {
        parent
            .spawn(
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(10., 10.)),
                        color: Color::hsl(200.0, 0.5, 0.5),
                        ..default()
                    },
                    texture: am.load("menu/menu_pf_1.png"),
                    transform: Transform::from_xyz(0., 0., -0.1),
                    ..default()
                }
            ); 
    });
}

fn handle_button_select(manager: Res<MainMenuManager>, mut buttons: Query<&mut MainMenuButton>) {
    for mut button in &mut buttons {
        if button.id == manager.get() {
            button.set_selected(true);
        } else {
            button.set_selected(false);
        }
    }
}

fn handle_buttons(mut buttons: Query<(&mut MainMenuButton, &mut Transform, &mut Sprite)>, time: Res<Time>) {
    let elapsed = time.delta();

    for (mut button, mut transform, mut sprite) in &mut buttons {
        let xpos = button.tick(elapsed);
        transform.translation.x = xpos;
        if button.selected {
            sprite.color.set_alpha(1.);
        }
        else {
            sprite.color.set_alpha(0.3);
        }
    }
}

fn read_keyboard(mut manager: ResMut<MainMenuManager>, keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.any_pressed([KeyCode::Space, KeyCode::Enter]) {
        if let Some(state) = manager.get_state_for_selected() {
            next_state.set(state);
        }
    }
    else if keys.any_just_pressed([KeyCode::KeyW, KeyCode::KeyA]) {
        manager.decrement();
    } else if keys.any_just_pressed([KeyCode::KeyS, KeyCode::KeyD]) {
        manager.increment();
    }
}
