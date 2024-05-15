use crate::game_interface::{Game, GameResult};
use crate::macroquad_interface::tooltip::CommandTooltip;
use crate::serialization;
use crate::view::Frame;
use egui_macroquad::macroquad::color::Color;
use egui_macroquad::macroquad::input::{
    is_key_pressed, is_mouse_button_released, KeyCode, MouseButton,
};
use egui_macroquad::macroquad::math::Vec2;
use egui_macroquad::macroquad::miniquad::conf::Icon;
use egui_macroquad::macroquad::{color, input, text, window};
use std::mem::take;
use std::process::exit;
use std::time;
use std::time::Instant;

mod camera;
mod render;
mod store_render;
mod texture;
mod tooltip;
mod ui;

pub mod error_view {
    use egui_macroquad::macroquad::{color, text, window};

    const TEXT_SIZE: u16 = 24;

    pub async fn show(messages: Vec<String>) -> ! {
        let messages = messages
            .into_iter()
            .flat_map(split_text_line)
            .collect::<Vec<_>>();
        loop {
            window::clear_background(color::BLACK);

            let mut y = 250.;
            for message in &messages {
                super::draw_centered_text(message, y, TEXT_SIZE, color::PINK);
                y += TEXT_SIZE as f32;
            }

            window::next_frame().await;
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
        text::measure_text(line, None, TEXT_SIZE, 1.).width <= 700.
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
}

pub fn logo() -> Icon {
    Icon {
        small: *include_bytes!("../icon/icon_16x16.rgba"),
        medium: *include_bytes!("../icon/icon_32x32.rgba"),
        big: *include_bytes!("../icon/icon_64x64.rgba"),
    }
}

pub async fn run(game: Game, autosave: bool) -> ! {
    let mut app = init(game);

    window::clear_background(color::BLACK);
    draw_centered_text("Loading textures...", 300., 32, color::LIGHTGRAY);
    window::next_frame().await;

    let textures = texture::load_textures();
    let mut textures = match textures {
        Ok(textures) => textures,
        Err(error) => {
            error_view::show(vec![
                format!("Unable to load textures:"),
                format!("{error}"),
            ])
            .await
        }
    };
    let color_map = texture::load_aftik_color_data();
    let color_map = match color_map {
        Ok(color_map) => color_map,
        Err(error) => {
            error_view::show(vec![
                format!("Unable to load aftik colors:"),
                format!("{error}"),
            ])
            .await
        }
    };

    if autosave {
        input::prevent_quit();
    }
    loop {
        if autosave && input::is_quit_requested() {
            if let Err(error) = serialization::write_game_to_save_file(&app.game) {
                eprintln!("Failed to save game: {error}");
            } else {
                println!("Saved the game successfully.")
            }
            exit(0);
        }

        if is_key_pressed(KeyCode::Tab) {
            app.show_graphical = !app.show_graphical;
        }

        if app.show_graphical {
            if app.last_drag_pos.is_none() {
                tooltip::handle_click(&mut app, &mut textures);
            }
            if app.command_tooltip.is_none() {
                camera::try_drag_camera(&mut app.render_state, &mut app.last_drag_pos);
            }
        } else {
            app.last_drag_pos = None;
            app.command_tooltip = None;
        }

        app.update_frame_state();

        render::draw(&mut app, &mut textures, &color_map);

        window::next_frame().await;
    }
}

fn init(game: Game) -> App {
    App {
        input: String::new(),
        game,
        last_frame_time: None,
        render_state: render::State::new(),
        last_drag_pos: None,
        command_tooltip: None,
        show_graphical: true,
        request_input_focus: false,
        show_next_frame: true,
    }
}

pub struct App {
    input: String,
    game: Game,
    last_frame_time: Option<Instant>,
    render_state: render::State,
    last_drag_pos: Option<Vec2>,
    command_tooltip: Option<CommandTooltip>,
    show_graphical: bool,
    request_input_focus: bool,
    show_next_frame: bool,
}

const DELAY: time::Duration = time::Duration::from_secs(2);

impl App {
    fn update_frame_state(&mut self) {
        if self.show_graphical {
            self.show_next_frame |= is_key_pressed(KeyCode::Enter)
                || is_mouse_button_released(MouseButton::Left)
                    && ui::is_mouse_at_text_box(&self.render_state)
        } else {
            self.show_next_frame |= self
                .last_frame_time
                .map_or(true, |instant| instant.elapsed() >= DELAY);
        }

        if self.show_next_frame {
            if let GameResult::Frame(frame_getter) = self.game.next_result() {
                let frame = frame_getter.get();
                self.show_frame(frame);
            }
            self.show_next_frame = false;
        }
    }

    fn show_frame(&mut self, frame: Frame) {
        self.last_frame_time = Some(Instant::now());
        let ready_for_input = self.game.ready_to_take_input();
        self.render_state.show_frame(frame, ready_for_input);
        self.request_input_focus = ready_for_input;
        self.command_tooltip = None;
    }

    fn handle_input(&mut self) {
        let input = take(&mut self.input);
        if !input.is_empty() {
            self.render_state.add_to_text_log(format!("> {input}"));
            match self.game.handle_input(&input) {
                Ok(()) => {
                    self.show_next_frame = true;
                }
                Err(messages) => {
                    self.render_state.show_input_error(messages);
                    self.request_input_focus = true;
                }
            }
        }
    }
}

pub fn draw_centered_text(text: &str, y: f32, font_size: u16, color: Color) {
    let text_size = text::measure_text(text, None, font_size, 1.);
    text::draw_text(
        text,
        (800. - text_size.width) / 2.,
        y,
        font_size as f32,
        color,
    );
}
