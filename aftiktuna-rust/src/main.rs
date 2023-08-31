use aftiktuna::game_interface::Game;
use aftiktuna::serialization::LoadError;
use aftiktuna::{game_interface, macroquad_interface};
use egui_macroquad::macroquad;
use egui_macroquad::macroquad::color::{BLACK, PINK};
use egui_macroquad::macroquad::text::{draw_text, measure_text};
use egui_macroquad::macroquad::window::{clear_background, next_frame, Conf};
use std::env;

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
    let disable_autosave = env::args().any(|arg| arg.eq("--disable-autosave"));
    let new_name = env::args().any(|arg| arg.eq("--new-game"));
    if disable_autosave {
        println!("Running without autosave");
    }
    match setup_game(new_name) {
        Ok(game) => macroquad_interface::run(game, !disable_autosave).await,
        Err(error) => {
            show_error(vec![
                format!("Unable to load save file: {error}"),
                "Consider deleting the savefile or using a different version of Aftiktuna."
                    .to_owned(),
            ])
            .await
        }
    }
}

fn setup_game(new_game: bool) -> Result<Game, LoadError> {
    if new_game {
        Ok(game_interface::setup_new())
    } else {
        game_interface::new_or_load()
    }
}

const TEXT_SIZE: u16 = 24;

async fn show_error(messages: Vec<String>) -> ! {
    let messages = messages
        .into_iter()
        .flat_map(split_text_line)
        .collect::<Vec<_>>();
    loop {
        clear_background(BLACK);

        let mut y = 200.;
        for message in &messages {
            let text_size = measure_text(message, None, TEXT_SIZE, 1.);
            draw_text(
                message,
                (800. - text_size.width) / 2.,
                y,
                TEXT_SIZE as f32,
                PINK,
            );
            y += TEXT_SIZE as f32;
        }

        next_frame().await;
    }
}

fn split_text_line(line: String) -> Vec<String> {
    if fits_on_screen(&line) {
        return vec![line];
    }

    let mut remaining_line: &str = &line;
    let mut vec = Vec::new();
    loop {
        let split_index = smallest_split(remaining_line);
        vec.push(remaining_line[..split_index].to_owned());
        remaining_line = &remaining_line[split_index..];

        if fits_on_screen(remaining_line) {
            vec.push(remaining_line.to_owned());
            return vec;
        }
    }
}

fn fits_on_screen(line: &str) -> bool {
    measure_text(line, None, TEXT_SIZE, 1.).width <= 700.
}

fn smallest_split(line: &str) -> usize {
    let mut last_space = 0;
    let mut last_index = 0;
    for (index, char) in line.char_indices() {
        if !fits_on_screen(&line[..index]) {
            return if last_space != 0 {
                last_space
            } else {
                last_index
            };
        }

        if char.is_whitespace() {
            last_space = index;
        }
        last_index = index;
    }
    line.len()
}
