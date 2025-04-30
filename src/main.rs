//! TODO: Option to input data as full table
//! TODO: Maybe option to change start and end years?

use std::{collections::HashSet, iter};

use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResized};
use bevy_egui::{EguiContexts, EguiPlugin};
use egui::{WidgetText, widgets::DragValue};
use egui_extras::{Column, TableBuilder};

#[derive(Resource)]
pub struct State {
    // active_years, sprite entity
    sprites: Vec<(HashSet<u32>, Entity)>,
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
    let names = [
        "Base Map", "Region 1", "Region 2", "Region 3", "Region 4", "Region 5",
    ];
    let paths = [
        "base.png",
        "region1.png",
        "region2.png",
        "region3.png",
        "region4.png",
        "region5.png",
    ];

    let start_year = 2016u32;
    let end_year = 2025u32;

    let mut sprites = vec![];

    for (idx, (name, path)) in iter::zip(names, paths).enumerate() {
        let sprite = cmds
            .spawn((
                Name::new(name),
                Sprite {
                    image: server.load(path),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, idx as f32),
                LayerMarker,
            ))
            .id();

        sprites.push((HashSet::from_iter(start_year..=end_year), sprite));
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

fn show_ui(mut ctx: EguiContexts, mut state: ResMut<State>, name_query: Query<&Name>) {
    // Reborrow trick to work around deref borrowing the whole struct
    let state = &mut *state;

    egui::Window::new("Data").show(ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Current Year: ");
            ui.add(
                DragValue::new(&mut state.current_year).range(state.start_year..=state.end_year),
            );
        });

        ui.collapsing("Coarse Data", |ui| {
            for (idx, (years, ..)) in state.sprites[1..].iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let mut is_active = !years.is_empty();

                    ui.checkbox(&mut is_active, WidgetText::default());

                    if is_active && years.is_empty() {
                        *years = HashSet::from_iter(state.start_year..=state.end_year);
                    }

                    ui.add_enabled_ui(is_active, |ui| {
                        let mut year = years.iter().min().copied().unwrap_or(state.start_year);

                        ui.label(format!("Region {} Year: ", idx + 1));
                        let res = ui.add(
                            DragValue::new(&mut year).range(state.start_year..=state.end_year),
                        );

                        if res.changed() {
                            *years = HashSet::from_iter(year..=state.end_year);
                        }
                    });
                });
            }
        });

        ui.collapsing("Fine Data", |ui| {
            TableBuilder::new(ui)
                .vscroll(false)
                .column(Column::auto())
                .columns(Column::auto(), state.sprites.len() - 1)
                .header(20.0, |mut ui| {
                    ui.col(|ui| {
                        ui.label("Year");
                    });

                    for (_years, entity) in &state.sprites[1..] {
                        ui.col(|ui| {
                            ui.label(
                                name_query
                                    .get(*entity)
                                    .map(|it| it.as_str())
                                    .unwrap_or("Unknown"),
                            );
                        });
                    }
                })
                .body(|mut ui| {
                    for year in state.start_year..=state.end_year {
                        ui.row(20.0, |mut ui| {
                            ui.col(|ui| {
                                ui.label(format!("{year}"));
                            });

                            for (years, _entity) in &mut state.sprites[1..] {
                                ui.col(|ui| {
                                    let was_shown = years.contains(&year);
                                    let mut shown = was_shown;

                                    ui.centered_and_justified(|ui| {
                                        ui.checkbox(&mut shown, WidgetText::default());
                                    });

                                    if shown != was_shown {
                                        if shown {
                                            years.insert(year);
                                        } else {
                                            years.remove(&year);
                                        }
                                    }
                                });
                            }
                        });
                    }
                });
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
    for (years, sprite) in state.sprites.iter() {
        let Ok(mut vis) = query.get_mut(*sprite) else {
            continue;
        };
        if years.contains(&state.current_year) {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
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
