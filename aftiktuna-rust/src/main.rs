use aftiktuna::area::Locations;
use aftiktuna::macroquad_interface;
use macroquad::prelude::Conf;

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
    macroquad_interface::run(Locations::new(3)).await;
}
