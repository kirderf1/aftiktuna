use crate::game_loop::{FrameCache, Game, TakeInput};
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

pub async fn run(game: Game, frame_cache: FrameCache) {
    let mut app = init(game);
    app.delayed_frames.cache = frame_cache;
    if let Some(frame) = app.delayed_frames.next_frame() {
        app.show_frame(frame);
    }
    let textures = texture::load_textures().await.unwrap();

    input::prevent_quit();
    loop {
        if input::is_quit_requested() {
            if let Err(error) =
                serialization::write_game_to_save_file(&app.game, &app.delayed_frames.cache)
            {
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
        delayed_frames: Default::default(),
        render_state: render::State::new(),
        show_graphical: true,
        request_input_focus: false,
    }
}

pub struct App {
    input: String,
    game: Game,
    state: GameState,
    delayed_frames: DelayedFrames,
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

#[derive(Default)]
struct DelayedFrames {
    cache: FrameCache,
    last_frame_time: Option<Instant>,
}

impl DelayedFrames {
    fn add_new_frames(&mut self, frames: Vec<Frame>) {
        self.cache.add_new_frames(frames);
        self.last_frame_time = None;
    }

    fn is_done(&self) -> bool {
        !self.cache.has_more_frames()
    }

    fn next_frame_after_elapsed_time(&mut self) -> Option<Frame> {
        if self
            .last_frame_time
            .map_or(true, |instant| instant.elapsed() >= DELAY)
        {
            self.next_frame()
        } else {
            None
        }
    }

    fn next_frame(&mut self) -> Option<Frame> {
        let frame = self.cache.take_next_frame()?;
        self.last_frame_time = Some(Instant::now());
        Some(frame)
    }
}

const DELAY: time::Duration = time::Duration::from_secs(2);

impl App {
    fn update_view_state(&mut self) {
        let frame = if !self.show_graphical {
            self.delayed_frames.next_frame_after_elapsed_time()
        } else if is_key_pressed(KeyCode::Enter)
            || is_mouse_button_released(MouseButton::Left)
                && ui::is_mouse_at_text_box(&self.render_state)
        {
            self.delayed_frames.next_frame()
        } else {
            None
        };
        if let Some(frame) = frame {
            self.show_frame(frame);
        }
    }

    fn show_frame(&mut self, frame: Frame) {
        let ready_for_input = self.delayed_frames.is_done() && self.state != GameState::Done;
        self.render_state.show_frame(frame, ready_for_input);
        self.request_input_focus = ready_for_input;
    }

    fn update_game_state(&mut self) {
        if self.state == GameState::Run {
            let (result, frames) = self.game.run();
            self.add_view_data(frames);

            match result {
                Ok(TakeInput) => {
                    self.state = GameState::Input;
                }
                Err(_) => {
                    self.state = GameState::Done;
                }
            }
        }
    }

    fn add_view_data(&mut self, frames: Vec<Frame>) {
        self.delayed_frames.add_new_frames(frames);
        if let Some(frame) = self.delayed_frames.next_frame() {
            self.show_frame(frame);
        }
    }

    fn ready_to_take_input(&self) -> bool {
        self.state == GameState::Input && self.delayed_frames.is_done()
    }

    fn handle_input(&mut self) {
        let input = take(&mut self.input);
        if !input.is_empty() {
            self.render_state.add_to_text_log(format!("> {input}"));
            if let Err(messages) = self.game.handle_input(&input) {
                self.render_state.show_input_error(messages);
                self.request_input_focus = true;
            } else {
                self.state = GameState::Run;
            }
        }
    }
}
