use aftiktuna::area::LocationTracker;
use aftiktuna::macroquad_interface;
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
    macroquad_interface::run(LocationTracker::new(3)).await;
}
