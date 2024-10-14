use aftiktuna::location::GenerationState;
use aftiktuna::macroquad_interface::egui::EguiWrapper;
use aftiktuna::{game_interface, macroquad_interface};
use macroquad::input;
use macroquad::window::{self, Conf};
use std::env;

fn config() -> Conf {
    macroquad_interface::default_conf("Aftiktuna location tester")
}

#[macroquad::main(config)]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let location = args[1].to_string();
    let game = game_interface::setup_new_with(
        GenerationState::single(location).expect("Unable to initialize game"),
    );

    window::next_frame().await;
    let mut assets = macroquad_interface::load_assets().await;

    input::prevent_quit();
    macroquad_interface::run_game(game, true, &mut assets, &mut EguiWrapper::init()).await;
}
