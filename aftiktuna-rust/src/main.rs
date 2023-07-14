use aftiktuna::area::Locations;
use aftiktuna::macroquad_interface;
use macroquad::prelude::Conf;

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna".to_string(),
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    macroquad_interface::run(Locations::new(3)).await;
}
