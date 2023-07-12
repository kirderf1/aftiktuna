use aftiktuna::area::Locations;
use aftiktuna::game_loop;
use aftiktuna::game_loop::{Game, TakeInput};
use aftiktuna::view;
use aftiktuna::view::Frame;
use macroquad::prelude::*;
use std::mem::take;
use std::time;
use std::time::Instant;

mod render;

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna".to_string(),
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let mut app = init();
    let textures = render::load_textures().await;

    loop {
        app.update_view_state();

        app.update_game_state();

        if is_key_pressed(KeyCode::Tab) {
            app.show_graphical = !app.show_graphical;
        }

        render::draw(&mut app, &textures);

        next_frame().await
    }
}

fn init() -> App {
    let (messages, game) = game_loop::setup(Locations::new(3));
    App {
        input: String::new(),
        game,
        state: GameState::Run,
        delayed_frames: Default::default(),
        render_state: render::State::new(messages),
        show_graphical: true,
    }
}

pub struct App {
    input: String,
    game: Game,
    state: GameState,
    delayed_frames: DelayedFrames,
    render_state: render::State,
    show_graphical: bool,
}

#[derive(Eq, PartialEq)]
enum GameState {
    Input,
    Run,
    Done,
}

#[derive(Default)]
struct DelayedFrames {
    remaining_frames: Vec<Frame>,
    last_frame: Option<Instant>,
}

impl DelayedFrames {
    fn new(view_buffer: view::Buffer) -> Self {
        let mut frames = view_buffer.into_frames();
        frames.reverse();
        Self {
            remaining_frames: frames,
            last_frame: None,
        }
    }

    fn is_done(&self) -> bool {
        self.remaining_frames.is_empty()
    }

    fn next_frame(&mut self) -> Option<Frame> {
        if self
            .last_frame
            .map_or(true, |instant| instant.elapsed() >= DELAY)
        {
            let frame = self.remaining_frames.pop()?;
            self.last_frame = Some(Instant::now());
            Some(frame)
        } else {
            None
        }
    }
}

const DELAY: time::Duration = time::Duration::from_secs(2);

impl App {
    fn update_view_state(&mut self) {
        if let Some(frame) = self.delayed_frames.next_frame() {
            self.render_state.show_frame(
                frame,
                self.delayed_frames.is_done() && self.state != GameState::Done,
            );
        }
    }

    fn update_game_state(&mut self) {
        if self.state == GameState::Run {
            let (result, view_buffer) = self.game.run();
            self.add_view_data(view_buffer);

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

    fn add_view_data(&mut self, view_buffer: view::Buffer) {
        self.delayed_frames = DelayedFrames::new(view_buffer);
        self.update_view_state();
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
            } else {
                self.state = GameState::Run;
            }
        }
    }
}
