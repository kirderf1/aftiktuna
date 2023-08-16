use aftiktuna::area::LocationTracker;
use aftiktuna::{game_loop, macroquad_interface};
use egui_macroquad::macroquad;
use egui_macroquad::macroquad::window::Conf;

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna".to_string(),
        window_width: 800,
        window_height: 600,
        window_resizable: false,
        icon: Some(macroquad_interface::logo()),
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let game = game_loop::setup(LocationTracker::new(3));
    macroquad_interface::run(game).await;
}
