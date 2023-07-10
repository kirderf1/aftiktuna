use aftiktuna::area::Locations;
use aftiktuna::game_loop;
use aftiktuna::game_loop::{Game, TakeInput};
use aftiktuna::view;
use aftiktuna::view::Messages;
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
        text_lines: messages.into_text(),
        input: String::new(),
        game,
        state: GameState::Run,
        delayed_views: None,
        render_state: render::State::LocationChoice,
        show_graphical: false,
    }
}

pub struct App {
    text_lines: Vec<String>,
    input: String,
    game: Game,
    state: GameState,
    delayed_views: Option<(Instant, DelayedViews)>,
    render_state: render::State,
    show_graphical: bool,
}

#[derive(Eq, PartialEq)]
enum GameState {
    Input,
    Run,
    Done,
}

struct DelayedViews {
    remaining_views: Vec<view::Data>,
    extra_messages: Option<Messages>,
}

impl DelayedViews {
    fn new(view_buffer: view::Buffer, extra_messages: Option<Messages>) -> Self {
        let mut views = view_buffer.into_data();
        views.reverse();
        Self {
            remaining_views: views,
            extra_messages,
        }
    }

    fn next_and_write(
        mut self,
        text_lines: &mut Vec<String>,
        state_consumer: impl FnOnce(render::State),
    ) -> Option<(Instant, Self)> {
        if let Some(view_data) = self.remaining_views.pop() {
            text_lines.extend(view_data.as_text());
            match view_data {
                view::Data::Full { render_data, .. } => {
                    state_consumer(render::State::InGame(render_data))
                }
                view::Data::Simple {
                    is_at_selection, ..
                } => {
                    if is_at_selection {
                        state_consumer(render::State::LocationChoice);
                    }
                }
            }

            if self.remaining_views.is_empty() && self.extra_messages.is_none() {
                None
            } else {
                Some((Instant::now(), self))
            }
        } else {
            if let Some(messages) = self.extra_messages {
                text_lines.push(String::default());
                text_lines.extend(messages.into_text());
            }
            None
        }
    }
}

const DELAY: time::Duration = time::Duration::from_secs(2);

impl App {
    fn update_view_state(&mut self) {
        if self
            .delayed_views
            .as_ref()
            .map_or(false, |(instant, _)| instant.elapsed() >= DELAY)
        {
            self.delayed_views = take(&mut self.delayed_views).and_then(|(_, delayed_views)| {
                delayed_views
                    .next_and_write(&mut self.text_lines, |state| self.render_state = state)
            });
        }
    }

    fn update_game_state(&mut self) {
        if self.state == GameState::Run {
            let mut view_buffer = view::Buffer::default();
            match self.game.run(&mut view_buffer) {
                Ok(TakeInput) => {
                    self.add_view_data(view_buffer, None);
                    self.state = GameState::Input;
                }
                Err(stop_type) => {
                    self.add_view_data(view_buffer, Some(stop_type.messages()));
                    self.state = GameState::Done;
                }
            }
        }
    }

    fn add_view_data(&mut self, view_buffer: view::Buffer, extra_messages: Option<Messages>) {
        let views = DelayedViews::new(view_buffer, extra_messages);
        self.delayed_views =
            views.next_and_write(&mut self.text_lines, |state| self.render_state = state);
    }

    fn ready_to_take_input(&self) -> bool {
        self.state == GameState::Input && self.delayed_views.is_none()
    }

    fn handle_input(&mut self) {
        let input = take(&mut self.input);
        if !input.is_empty() {
            self.text_lines.push(format!("> {input}"));
            if let Err(messages) = self.game.handle_input(&input) {
                self.text_lines.extend(messages.into_text());
            } else {
                self.state = GameState::Run;
            }
        }
    }
}
