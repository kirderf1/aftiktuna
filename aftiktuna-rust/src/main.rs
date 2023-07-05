use aftiktuna::area::Locations;
use aftiktuna::game_loop;
use aftiktuna::game_loop::{Game, TakeInput};
use aftiktuna::position::Coord;
use aftiktuna::view;
use aftiktuna::view::{Messages, RenderData, TextureType};
use egui_macroquad::egui;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::mem::take;
use std::time;
use std::time::Instant;

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna".to_string(),
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let mut app = init();
    let background = load_texture("assets/tree_background.png").await.unwrap();
    let textures = setup_object_textures().await;

    loop {
        app.update_view_state();

        app.update_game_state();

        clear_background(BLACK);

        if is_key_down(KeyCode::Tab) {
            let width = screen_width();
            let height = screen_height();
            draw_texture_ex(
                background,
                0.,
                0.,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new((800. - width) / 2., 600. - height, width, height)),
                    dest_size: Some(Vec2::new(width, height)),
                    ..Default::default()
                },
            );

            if let Some(render_data) = &app.render_data {
                draw_objects(render_data, &textures);
            }
        } else {
            egui_macroquad::ui(|ctx| app.ui(ctx));

            egui_macroquad::draw();
        }

        next_frame().await
    }
}

struct TextureData {
    texture: Texture2D,
    dest_size: Vec2,
}

async fn setup_object_textures() -> HashMap<TextureType, TextureData> {
    let unknown = load_texture("assets/unknown.png").await.unwrap();
    let aftik = load_texture("assets/aftik.png").await.unwrap();

    let mut textures = HashMap::new();

    textures.insert(
        TextureType::Unknown,
        TextureData {
            texture: unknown,
            dest_size: Vec2::new(120., 200.),
        },
    );
    textures.insert(
        TextureType::SmallUnknown,
        TextureData {
            texture: unknown,
            dest_size: Vec2::new(100., 100.),
        },
    );
    textures.insert(
        TextureType::Aftik,
        TextureData {
            texture: aftik,
            dest_size: Vec2::new(100., 140.),
        },
    );

    textures
}

fn draw_objects(render_data: &RenderData, textures: &HashMap<TextureType, TextureData>) {
    let size = render_data.size;
    let start_x = (800. - (size - 1) as f32 * 120.) / 2.;
    let mut coord_counts: HashMap<Coord, i32> = HashMap::new();

    for data in &render_data.objects {
        let coord = data.coord;
        let count_ref = coord_counts.entry(coord).or_insert(0);
        let count = *count_ref;
        *count_ref = count + 1;

        draw_object(
            textures.get(&data.texture_type).unwrap(),
            start_x + ((coord as i32) * 120 - count * 15) as f32,
            (500 + count * 10) as f32,
        );
    }
}

fn draw_object(data: &TextureData, x: f32, y: f32) {
    let size = data.dest_size;
    draw_texture_ex(
        data.texture,
        x - size.x / 2.,
        y - size.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            ..Default::default()
        },
    );
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
    }
}

struct App {
    text_lines: Vec<String>,
    input: String,
    game: Game,
    state: State,
    delayed_views: Option<(Instant, DelayedViews)>,
    render_data: Option<RenderData>,
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

const FONT: egui::FontId = egui::FontId::monospace(15.0);
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

    fn input_field(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let response = ui.add_enabled(
            self.state == State::Input && self.delayed_views.is_none(),
            egui::TextEdit::singleline(&mut self.input)
                .font(FONT)
                .desired_width(f32::INFINITY),
        );

        if response.lost_focus()
            && ui.input(|input_state| input_state.key_pressed(egui::Key::Enter))
        {
            let input = take(&mut self.input);
            if !input.is_empty() {
                self.text_lines.push(format!("> {input}"));
                if let Err(messages) = self.game.handle_input(&input) {
                    self.text_lines.extend(messages.into_text());
                } else {
                    self.state = State::Run;
                }
            }
            ctx.memory_mut(|memory| memory.request_focus(response.id));
        }
    }

    fn text_box(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
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
        self.delayed_views =
            views.next_and_write(&mut self.text_lines, |data| self.render_data = Some(data));
    }

    fn ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("input").show(ctx, |ui| self.input_field(ctx, ui));

        egui::CentralPanel::default().show(ctx, |ui| self.text_box(ui));
    }
}
