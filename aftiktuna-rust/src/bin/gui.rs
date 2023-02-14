use aftiktuna::area::Locations;
use aftiktuna::game_loop;
use aftiktuna::game_loop::{Game, TakeInput};
use aftiktuna::view;
use eframe::egui;
use eframe::egui::ScrollArea;
use std::mem::take;

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
    }
}

struct App {
    text_lines: Vec<String>,
    input: String,
    game: Game,
    state: State,
}

#[derive(Eq, PartialEq)]
enum State {
    Input,
    Run,
    Done,
}

impl App {
    fn add_view_data(&mut self, view_buffer: view::Buffer) {
        self.text_lines.extend(
            view_buffer
                .into_data()
                .into_iter()
                .flat_map(view::Data::into_text)
                .collect::<Vec<_>>(),
        );
    }
}

const FONT: egui::FontId = egui::FontId::monospace(15.0);

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state == State::Run {
            let mut view_buffer = view::Buffer::default();
            match self.game.run(&mut view_buffer) {
                Ok(TakeInput) => {
                    self.add_view_data(view_buffer);
                    self.state = State::Input
                },
                Err(stop_type) => {
                    self.add_view_data(view_buffer);
                    self.text_lines.extend(stop_type.messages().into_text());
                    self.state = State::Done;
                }
            }
        }

        egui::TopBottomPanel::bottom("input").show(ctx, |ui| {
            let response = ui.add_enabled(
                self.state == State::Input,
                egui::TextEdit::singleline(&mut self.input)
                    .font(FONT)
                    .desired_width(f32::INFINITY),
            );

            if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                let input = take(&mut self.input);
                if !input.is_empty() {
                    self.text_lines.push(format!("> {}", input));
                    if let Err(messages) = self.game.handle_input(&input) {
                        self.text_lines.extend(messages.into_text());
                    } else {
                        self.state = State::Run;
                    }
                }
                ctx.memory().request_focus(response.id);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for text in &self.text_lines {
                        ui.label(egui::RichText::new(text).font(FONT));
                    }
                })
        });
    }
}
