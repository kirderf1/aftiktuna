use crate::game_loop::{Game, TakeInput};
use crate::serialization;
use crate::view::Frame;
use egui_macroquad::macroquad::input;
use egui_macroquad::macroquad::input::{
    is_key_pressed, is_mouse_button_released, KeyCode, MouseButton,
};
use egui_macroquad::macroquad::miniquad::conf::Icon;
use egui_macroquad::macroquad::window::next_frame;
use std::mem::take;
use std::process::exit;
use std::time;
use std::time::Instant;

mod render;
mod texture;
mod ui;

pub fn logo() -> Icon {
    Icon {
        small: *include_bytes!("../icon/icon_16x16.rgba"),
        medium: *include_bytes!("../icon/icon_32x32.rgba"),
        big: *include_bytes!("../icon/icon_64x64.rgba"),
    }
}

pub async fn run(game: Game) {
    let mut app = init(game);
    if let Some(frame) = app.game.frame_cache.take_next_frame() {
        app.show_frame(frame);
    }
    let textures = texture::load_textures().await.unwrap();

    input::prevent_quit();
    loop {
        if input::is_quit_requested() {
            if let Err(error) = serialization::write_game_to_save_file(&app.game) {
                eprintln!("Failed to save game: {error}");
            } else {
                println!("Saved the game successfully.")
            }
            exit(0);
        }

        app.update_view_state();

        app.update_game_state();

        if is_key_pressed(KeyCode::Tab) {
            app.show_graphical = !app.show_graphical;
        }

        render::draw(&mut app, &textures);

        next_frame().await
    }
}

fn init(game: Game) -> App {
    App {
        input: String::new(),
        game,
        state: GameState::Run,
        last_frame_time: None,
        render_state: render::State::new(),
        show_graphical: true,
        request_input_focus: false,
    }
}

pub struct App {
    input: String,
    game: Game,
    state: GameState,
    last_frame_time: Option<Instant>,
    render_state: render::State,
    show_graphical: bool,
    request_input_focus: bool,
}

#[derive(Eq, PartialEq)]
enum GameState {
    Input,
    Run,
    Done,
}

const DELAY: time::Duration = time::Duration::from_secs(2);

impl App {
    fn update_view_state(&mut self) {
        let frame = if !self.show_graphical {
            if self
                .last_frame_time
                .map_or(true, |instant| instant.elapsed() >= DELAY)
            {
                self.game.frame_cache.take_next_frame()
            } else {
                None
            }
        } else if is_key_pressed(KeyCode::Enter)
            || is_mouse_button_released(MouseButton::Left)
                && ui::is_mouse_at_text_box(&self.render_state)
        {
            self.game.frame_cache.take_next_frame()
        } else {
            None
        };
        if let Some(frame) = frame {
            self.show_frame(frame);
        }
    }

    fn show_frame(&mut self, frame: Frame) {
        self.last_frame_time = Some(Instant::now());
        let ready_for_input =
            !self.game.frame_cache.has_more_frames() && self.state != GameState::Done;
        self.render_state.show_frame(frame, ready_for_input);
        self.request_input_focus = ready_for_input;
    }

    fn update_game_state(&mut self) {
        if self.state == GameState::Run && !self.game.frame_cache.has_more_frames() {
            match self.game.run() {
                Ok(TakeInput) => {
                    self.state = GameState::Input;
                }
                Err(_) => {
                    self.state = GameState::Done;
                }
            }
            if let Some(frame) = self.game.frame_cache.take_next_frame() {
                self.show_frame(frame);
            }
        }
    }

    fn ready_to_take_input(&self) -> bool {
        self.state == GameState::Input && !self.game.frame_cache.has_more_frames()
    }

    fn handle_input(&mut self) {
        let input = take(&mut self.input);
        if !input.is_empty() {
            self.render_state.add_to_text_log(format!("> {input}"));
            if let Err(messages) = self.game.handle_input(&input) {
                self.render_state.show_input_error(messages);
                self.request_input_focus = true;
            } else {
                self.last_frame_time = None;
                self.state = GameState::Run;
            }
        }
    }
}
