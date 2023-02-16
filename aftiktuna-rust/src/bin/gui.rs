use aftiktuna::area::Locations;
use aftiktuna::game_loop;
use aftiktuna::game_loop::{Game, TakeInput};
use aftiktuna::view;
use aftiktuna::view::Messages;
use eframe::egui;
use eframe::egui::ScrollArea;
use std::mem::take;
use std::time;
use std::time::Instant;

fn main() {
    let options = eframe::NativeOptions {
        resizable: false,
        ..Default::default()
    };
    eframe::run_native("Aftiktuna", options, Box::new(|_cc| Box::new(init())));
}

fn init() -> App {
    let (messages, game) = game_loop::setup(Locations::new(3));
    App {
        text_lines: messages.into_text(),
        input: String::new(),
        game,
        state: State::Run,
        delayed_views: None,
    }
}

struct App {
    text_lines: Vec<String>,
    input: String,
    game: Game,
    state: State,
    delayed_views: Option<(Instant, DelayedViews)>,
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

    fn next_and_write(mut self, text_lines: &mut Vec<String>) -> Option<(Instant, Self)> {
        if let Some(view_data) = self.remaining_views.pop() {
            text_lines.extend(view_data.into_text());
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

const FONT: egui::FontId = egui::FontId::monospace(15.0);
const DELAY: time::Duration = time::Duration::from_secs(2);

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_view_state();

        self.update_game_state();

        egui::TopBottomPanel::bottom("input").show(ctx, |ui| self.input_field(ctx, ui));

        egui::CentralPanel::default().show(ctx, |ui| self.text_box(ui));
    }
}

impl App {
    fn update_view_state(&mut self) {
        if self
            .delayed_views
            .as_ref()
            .map_or(false, |(instant, _)| instant.elapsed() >= DELAY)
        {
            self.delayed_views = take(&mut self.delayed_views).and_then(|(_, delayed_views)| {
                delayed_views.next_and_write(&mut self.text_lines)
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

    fn input_field(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let response = ui.add_enabled(
            self.state == State::Input && self.delayed_views.is_none(),
            egui::TextEdit::singleline(&mut self.input)
                .font(FONT)
                .desired_width(f32::INFINITY),
        );

        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
            let input = take(&mut self.input);
            if !input.is_empty() {
                self.text_lines.push(format!("> {input}"));
                if let Err(messages) = self.game.handle_input(&input) {
                    self.text_lines.extend(messages.into_text());
                } else {
                    self.state = State::Run;
                }
            }
            ctx.memory().request_focus(response.id);
        }
    }

    fn text_box(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for text in &self.text_lines {
                    ui.label(egui::RichText::new(text).font(FONT));
                }
            });
    }

    fn add_view_data(&mut self, view_buffer: view::Buffer, extra_messages: Option<Messages>) {
        let views = DelayedViews::new(view_buffer, extra_messages);
        self.delayed_views = views.next_and_write(&mut self.text_lines);
    }
}
