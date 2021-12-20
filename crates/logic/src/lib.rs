use bevy::{prelude::*, reflect::List};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_lyon::plugin::ShapePlugin;
use map_graph::{Coins, MapConfiguration, MapGraphPlugin, RandomDeterministic, RoomChanceWeights};
use wasm_bindgen::prelude::*;

pub mod combat;
pub mod danger;
pub mod delayed_destroy;
pub mod graphics_rooms;
pub mod map_graph;
pub mod math_utils;
mod poisson;
pub mod shapes;
pub mod text_feedback;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Menu,
    Loading,
    Game,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugins(DefaultPlugins);
        app.add_plugin(ShapePlugin);

        #[cfg(target_arch = "wasm32")]
        app.add_plugin(bevy_webgl2::WebGL2Plugin);

        app.add_plugin(EguiPlugin)
            .add_plugin(MapGraphPlugin)
            .add_state(AppState::Menu)
            .add_system(ui_menu.system())
            .add_system(game_menu.system());
    }
}

fn ui_menu(
    mut state: ResMut<State<AppState>>,
    mut map_configuration: ResMut<MapConfiguration>,
    mut chance_rooms: ResMut<RoomChanceWeights>,
    mut random: ResMut<RandomDeterministic>,
    egui_context: ResMut<EguiContext>,
) {
    if state.current() != &AppState::Menu {
        return;
    }
    egui::SidePanel::left("panel_menu")
        .default_width(200.0)
        .show(egui_context.ctx(), |ui| {
            ui.label("In Menu");
            ui.checkbox(
                &mut map_configuration.start_with_danger_zone,
                "Start with danger zone",
            );
            input_float(
                ui,
                "Danger initial speed",
                &mut map_configuration.speed_init_danger,
            );
            input_float(
                ui,
                "Danger speed gain",
                &mut map_configuration.speed_gain_danger,
            );
            let mut seed = random.seed;
            input_u64(ui, "Random Seed", &mut seed);
            if seed != random.seed {
                random.set_seed(seed);
            }
            ui.collapsing("Room chances", |ui| {
                let mut changed_weights = false;
                changed_weights = changed_weights
                    || input_usize(
                        ui,
                        "weight Danger",
                        &mut chance_rooms.weights.get_mut(0).unwrap(),
                    );
                changed_weights = changed_weights
                    || input_usize(
                        ui,
                        "weight Safe",
                        &mut chance_rooms.weights.get_mut(1).unwrap(),
                    );
                changed_weights = changed_weights
                    || input_usize(
                        ui,
                        "weight Coins",
                        &mut chance_rooms.weights.get_mut(2).unwrap(),
                    );
                changed_weights = changed_weights
                    || input_usize(
                        ui,
                        "weight Price",
                        &mut chance_rooms.weights.get_mut(3).unwrap(),
                    );
                if changed_weights {
                    chance_rooms.update_weights();
                }
            });

            /*
            pub struct RoomChanceWeights {
                pub weighted_index: WeightedIndex<usize>,
                pub definitions: [RoomDefinition; 4],
                pub rooms_to_create_on_move: u32,
            }
            pub struct RoomDefinition {
                pub type_room: RoomType,
                pub battle_chance: f64,
                pub max_rooms_create: u32,
            }*/
            if ui.button("Start").clicked() {
                state.set(AppState::Loading);
            }
        });
}
fn input_usize(ui: &mut egui::Ui, label: &str, value: &mut usize) -> bool {
    ui.label(label);
    let mut input = format!("{:}", value);
    if ui.text_edit_singleline(&mut input).changed() {
        if let Ok(res) = input.parse::<usize>() {
            *value = res;
            return true;
        }
    }
    false
}
fn input_u64(ui: &mut egui::Ui, label: &str, value: &mut u64) -> bool {
    ui.label(label);
    let mut input = format!("{:}", value);
    if ui.text_edit_singleline(&mut input).changed() {
        if let Ok(res) = input.parse::<u64>() {
            *value = res;
            return true;
        }
    }
    false
}
fn input_float(ui: &mut egui::Ui, label: &str, value: &mut f32) -> bool {
    ui.label(label);
    let mut input = format!("{:.2}", value);
    if ui.text_edit_singleline(&mut input).changed() {
        if let Ok(res) = input.parse::<f32>() {
            *value = res;
            return true;
        }
    }
    false
}
fn game_menu(
    mut state: ResMut<State<AppState>>,
    coins: Res<Coins>,
    egui_context: ResMut<EguiContext>,
) {
    if state.current() != &AppState::Game {
        return;
    }
    egui::SidePanel::left("panel_game")
        .default_width(200.0)
        .show(egui_context.ctx(), |ui| {
            ui.label("In game");
            ui.label(format!("Coins: {}", coins.amount));
            if ui.button("Back").clicked() {
                state.set(AppState::Menu);
            }
        });
}

#[wasm_bindgen]
pub fn run() {
    App::build().add_plugin(GamePlugin).run();
}
