use aftiktuna::area::LocationTracker;
use aftiktuna::{game_interface, macroquad_interface};
use egui_macroquad::macroquad;
use egui_macroquad::macroquad::window::Conf;
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
    let game = game_interface::setup_new(LocationTracker::single(location));
    macroquad_interface::run(game).await;
}
