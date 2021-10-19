use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use map_graph::{Coins, MapGraphPlugin};
use wasm_bindgen::prelude::*;

pub mod danger;
pub mod graphics_rooms;
pub mod map_graph;
pub mod math_utils;
mod poisson;
use graphics_rooms::*;

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

        #[cfg(target_arch = "wasm32")]
        app.add_plugin(bevy_webgl2::WebGL2Plugin);

        app.add_plugin(EguiPlugin)
            .add_plugin(MapGraphPlugin)
            .add_state(AppState::Menu)
            .add_system(ui_menu.system())
            .add_system(game_menu.system());
    }
}

fn ui_menu(mut state: ResMut<State<AppState>>, egui_context: ResMut<EguiContext>) {
    if state.current() != &AppState::Menu {
        return;
    }
    egui::SidePanel::left("panel_menu")
        .default_width(200.0)
        .show(egui_context.ctx(), |ui| {
            ui.label("In Menu");
            if ui.button("Start").clicked() {
                state.set(AppState::Loading);
            }
        });
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
