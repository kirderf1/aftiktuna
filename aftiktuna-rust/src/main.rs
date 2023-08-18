use aftiktuna::{game_loop, macroquad_interface};
use egui_macroquad::macroquad;
use egui_macroquad::macroquad::color::{BLACK, PINK};
use egui_macroquad::macroquad::text::{draw_text, measure_text};
use egui_macroquad::macroquad::window::{clear_background, next_frame, Conf};

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
    match game_loop::new_or_load() {
        Ok((game, frame_cache)) => macroquad_interface::run(game, frame_cache).await,
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

const TEXT_SIZE: u16 = 24;

async fn show_error(messages: Vec<String>) {
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
