//! TODO: Option to input data as full table
//! TODO: Maybe option to change start and end years?

use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResized};
use bevy_egui::{EguiContexts, EguiPlugin};
use egui::widgets::DragValue;

#[derive(Resource)]
pub struct State {
    // transition_year, sprite entity, is_active
    sprites: Vec<(u32, Entity, bool)>,
    start_year: u32,
    end_year: u32,
    current_year: u32,
}

#[derive(Component)]
pub struct LayerMarker;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (show_ui, advance_year, set_visibility, adjust_scale),
        )
        .run();
}

fn setup(mut cmds: Commands, server: Res<AssetServer>) {
    let paths = [
        "base.png",
        "region1.png",
        "region2.png",
        "region3.png",
        "region4.png",
        "region5.png",
    ];

    let start_year = 2016;
    let end_year = 2025;

    let mut sprites = vec![];

    for (idx, path) in paths.into_iter().enumerate() {
        let sprite = cmds
            .spawn((
                Name::new(path),
                Sprite {
                    image: server.load(path),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, idx as f32),
                LayerMarker,
            ))
            .id();

        sprites.push((start_year, sprite, true));
    }

    cmds.insert_resource(State {
        sprites,
        start_year,
        current_year: start_year,
        end_year,
    });

    cmds.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 100.0),
    ));
}

fn show_ui(mut ctx: EguiContexts, mut state: ResMut<State>) {
    // Reborrow trick to work around deref borrowing the whole struct
    let state = &mut *state;

    egui::Window::new("Data").show(ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Current Year: ");
            ui.add(
                DragValue::new(&mut state.current_year).range(state.start_year..=state.end_year),
            );
        });

        ui.collapsing("Data", |ui| {
            for (idx, (year, _, is_active)) in state.sprites[1..].iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(is_active, "");
                    ui.add_enabled_ui(*is_active, |ui| {
                        ui.label(format!("Region {} Year: ", idx + 1));
                        ui.add(DragValue::new(year).range(state.start_year..=state.end_year));
                    });
                });
            }
        });
    });
}

fn advance_year(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<State>) {
    if keys.just_pressed(KeyCode::Space) {
        state.current_year += 1;
        if state.current_year > state.end_year {
            state.current_year = state.end_year;
        }
    }
    if keys.just_pressed(KeyCode::Backspace) {
        state.current_year -= 1;
        if state.current_year < state.start_year {
            state.current_year = state.start_year;
        }
    }
}

fn set_visibility(mut query: Query<&mut Visibility, With<LayerMarker>>, state: Res<State>) {
    if state.is_changed() {
        for (year, sprite, is_active) in state.sprites.iter() {
            let Ok(mut vis) = query.get_mut(*sprite) else {
                continue;
            };
            if *year <= state.current_year && *is_active {
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn adjust_scale(
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut Sprite, With<LayerMarker>>,
) {
    for event in resize_events.read() {
        for mut sprite in query.iter_mut() {
            let size = event.width.min(event.height);
            sprite.custom_size = Some(Vec2::splat(size));
        }
    }
}
