use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_lyon::plugin::ShapePlugin;
use map_graph::{Coins, MapConfiguration, MapGraphPlugin};
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
            if ui.button("Start").clicked() {
                state.set(AppState::Loading);
            }
        });
}

fn input_float(ui: &mut egui::Ui, label: &str, value: &mut f32) {
    ui.label(label);
    let mut speed_init_danger = format!("{:.2}", value);
    ui.text_edit_singleline(&mut speed_init_danger);
    if let Ok(res) = speed_init_danger.parse::<f32>() {
        *value = res;
    }
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
