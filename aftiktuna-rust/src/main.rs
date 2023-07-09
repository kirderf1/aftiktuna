use aftiktuna::area::Locations;
use aftiktuna::game_loop;
use aftiktuna::game_loop::{Game, TakeInput};
use aftiktuna::view;
use aftiktuna::view::{Messages, RenderData};
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
    let background = load_texture(&render::texture_path("tree_background"))
        .await
        .unwrap();
    let textures = render::setup_object_textures().await;

    loop {
        app.update_view_state();

        app.update_game_state();

        if is_key_pressed(KeyCode::Tab) {
            app.show_graphical = !app.show_graphical;
        }

        render::draw(&mut app, background, &textures);

        next_frame().await
    }
}

fn init() -> App {
    let (messages, game) = game_loop::setup(Locations::new(3));
    App {
        text_lines: messages.into_text(),
        input: String::new(),
        game,
        state: State::Run,
        delayed_views: None,
        render_data: None,
        show_graphical: false,
    }
}

pub struct App {
    text_lines: Vec<String>,
    input: String,
    game: Game,
    state: State,
    delayed_views: Option<(Instant, DelayedViews)>,
    render_data: Option<RenderData>,
    show_graphical: bool,
}

#[derive(Eq, PartialEq)]
enum State {
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
        data_consumer: impl FnOnce(RenderData),
    ) -> Option<(Instant, Self)> {
        if let Some(view_data) = self.remaining_views.pop() {
            text_lines.extend(view_data.as_text());
            if let view::Data::Full { render_data, .. } = view_data {
                data_consumer(render_data);
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
                    .next_and_write(&mut self.text_lines, |data| self.render_data = Some(data))
            });
        }
    }

    fn update_game_state(&mut self) {
        if self.state == State::Run {
            let mut view_buffer = view::Buffer::default();
            match self.game.run(&mut view_buffer) {
                Ok(TakeInput) => {
                    self.add_view_data(view_buffer, None);
                    self.state = State::Input;
                }
                Err(stop_type) => {
                    self.add_view_data(view_buffer, Some(stop_type.messages()));
                    self.state = State::Done;
                }
            }
        }
    }

    fn add_view_data(&mut self, view_buffer: view::Buffer, extra_messages: Option<Messages>) {
        let views = DelayedViews::new(view_buffer, extra_messages);
        self.delayed_views =
            views.next_and_write(&mut self.text_lines, |data| self.render_data = Some(data));
    }
}
