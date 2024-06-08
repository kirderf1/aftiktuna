use aftiktuna::location::GenerationState;
use aftiktuna::{game_interface, macroquad_interface};
use egui_macroquad::macroquad::window::{self, Conf};
use egui_macroquad::macroquad::{self, input};
use std::env;

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna location tester".to_string(),
        window_width: 800,
        window_height: 600,
        window_resizable: false,
        icon: Some(macroquad_interface::logo()),
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let location = args[1].to_string();
    let game = game_interface::setup_new_with(GenerationState::single(location));

    window::next_frame().await;
    let mut assets = macroquad_interface::load_assets().await;

    input::prevent_quit();
    macroquad_interface::run(game, &mut assets, true).await;
}
