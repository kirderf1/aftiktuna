use aftiktuna::area::LocationTracker;
use aftiktuna::macroquad_interface;
use macroquad::prelude::Conf;
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
    macroquad_interface::run(LocationTracker::single(location)).await;
}
