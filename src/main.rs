use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use map_graph::MapGraphPlugin;

pub mod danger;
mod map_graph;
pub mod math_utils;
mod poisson;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    InGame,
}
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        //.add_plugin(EguiPlugin)
        .add_plugin(MapGraphPlugin)
        //.add_state(AppState::Menu)
        //.add_system(ui_menu.system())
        //.add_system(game_menu.system())
        .run();
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
                state.push(AppState::InGame).unwrap();
            }
        });
}
fn game_menu(mut state: ResMut<State<AppState>>, egui_context: ResMut<EguiContext>) {
    if state.current() != &AppState::InGame {
        return;
    }
    egui::SidePanel::left("panel_game")
        .default_width(200.0)
        .show(egui_context.ctx(), |ui| {
            ui.label("In game");
            if ui.button("Back").clicked() {
                state.pop().unwrap();
            }
        });
}
