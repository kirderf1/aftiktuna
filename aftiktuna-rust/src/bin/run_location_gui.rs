use aftiktuna::area::Locations;
use std::env;
use macroquad::prelude::Conf;
use aftiktuna::macroquad_interface;

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna location tester".to_string(),
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let location = args[1].to_string();
    macroquad_interface::run(Locations::single(location)).await;
}
