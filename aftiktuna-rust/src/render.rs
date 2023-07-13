use crate::App;
use aftiktuna::position::Coord;
use aftiktuna::view::{Frame, Messages, RenderData};
use egui_macroquad::egui;
use macroquad::camera::set_camera;
use macroquad::color::{BLACK, WHITE};
use macroquad::prelude::{
    clamp, clear_background, draw_rectangle, draw_text, draw_texture, measure_text,
    set_default_camera, Camera2D, Color, Rect,
};
use std::collections::HashMap;
use texture::{BGTexture, BGTextureType, TextureStorage};

pub mod texture;

const FONT: egui::FontId = egui::FontId::monospace(15.0);

pub struct State {
    text_lines: Vec<String>,
    view_state: ViewState,
    text_box_text: Vec<String>,
}

impl State {
    pub fn new(messages: Messages) -> Self {
        Self {
            text_lines: messages.into_text(),
            view_state: ViewState::LocationChoice,
            text_box_text: vec![],
        }
    }

    pub fn show_frame(&mut self, frame: Frame, ready_for_input: bool) {
        self.text_lines.extend(frame.as_text());
        if ready_for_input {
            self.text_lines.push(String::default())
        }

        match frame {
            Frame::AreaView {
                render_data,
                messages,
                ..
            } => {
                self.view_state = ViewState::InGame(render_data);
                self.set_text_box_text(messages.into_text());
            }
            Frame::LocationChoice(messages) => {
                self.view_state = ViewState::LocationChoice;
                self.set_text_box_text(messages.into_text());
            }
            Frame::Ending(stop_type) => {
                self.set_text_box_text(stop_type.messages().into_text());
            }
        }
    }

    fn set_text_box_text(&mut self, text: Vec<String>) {
        self.text_box_text = text.into_iter().flat_map(split_text_line).collect();
    }

    pub fn add_to_text_log(&mut self, text: String) {
        self.text_lines.push(text);
    }

    pub fn show_input_error(&mut self, messages: Messages) {
        let text = messages.into_text();
        self.text_lines.extend(text.clone());
        self.set_text_box_text(text);
    }
}

pub enum ViewState {
    LocationChoice,
    InGame(RenderData),
}

pub fn draw(app: &mut App, textures: &TextureStorage) {
    clear_background(BLACK);

    if app.show_graphical {
        draw_game(&app.render_state, textures);
    }

    egui_macroquad::ui(|ctx| ui(app, ctx));

    egui_macroquad::draw();
}

fn default_camera_space() -> Rect {
    Rect::new(0., 0., 800., 600.)
}

fn draw_game(state: &State, textures: &TextureStorage) {
    match &state.view_state {
        ViewState::LocationChoice => {
            draw_background(
                BGTextureType::LocationChoice,
                default_camera_space(),
                textures,
            );
        }
        ViewState::InGame(render_data) => {
            let camera_space = setup_camera(render_data);
            draw_background(
                render_data
                    .background
                    .map_or(BGTextureType::Blank, BGTextureType::from),
                camera_space,
                textures,
            );

            draw_objects(render_data, textures);
            set_default_camera();
        }
    }

    draw_text_box(&state.text_box_text);
}

fn draw_background(texture_type: BGTextureType, camera_space: Rect, textures: &TextureStorage) {
    match textures.lookup_background(texture_type) {
        BGTexture::Simple(texture) => draw_texture(*texture, camera_space.x, 0., WHITE),
        BGTexture::Repeating(texture) => {
            let start_x = 800. * f32::floor(camera_space.x / 800.);
            draw_texture(*texture, start_x, 0., WHITE);
            draw_texture(*texture, start_x + 800., 0., WHITE);
        }
    }
}

fn draw_objects(render_data: &RenderData, textures: &TextureStorage) {
    let start_x = 40.;
    let mut coord_counts: HashMap<Coord, i32> = HashMap::new();

    for data in &render_data.objects {
        let coord = data.coord;
        let count_ref = coord_counts.entry(coord).or_insert(0);
        let count = *count_ref;
        *count_ref = count + 1;

        texture::draw_object(
            textures.lookup_texture(data.texture_type),
            data.direction,
            start_x + ((coord as i32) * 120 - count * 15) as f32,
            (450 + count * 10) as f32,
        );
    }
}

fn setup_camera(render_data: &RenderData) -> Rect {
    let camera_target = if render_data.size <= 6 {
        (render_data.size - 1) as f32 / 2.
    } else {
        clamp(
            render_data.character_coord as f32,
            2.5,
            render_data.size as f32 - 3.5,
        )
    };

    let camera_space = Rect::new((camera_target - 3.) * 120., 0., 800., 600.);
    set_camera(&Camera2D::from_display_rect(camera_space));
    camera_space
}

const TEXT_BOX_COLOR: Color = Color::new(0.2, 0.1, 0.4, 0.6);
const TEXT_BOX_TEXT_SIZE: u16 = 16;
const TEXT_BOX_MARGIN: f32 = 8.;
const TEXT_BOX_TEXT_MAX_WIDTH: f32 = 800. - 2. * TEXT_BOX_MARGIN;

fn split_text_line(line: String) -> Vec<String> {
    if fits_in_text_box_width(&line) {
        return vec![line];
    }

    let mut remaining_line: &str = &line;
    let mut vec = Vec::new();
    loop {
        let split_index = smallest_split(remaining_line);
        vec.push(remaining_line[..split_index].to_owned());
        remaining_line = &remaining_line[split_index..];

        if fits_in_text_box_width(remaining_line) {
            vec.push(remaining_line.to_owned());
            return vec;
        }
    }
}

fn smallest_split(line: &str) -> usize {
    let mut last_space = 0;
    let mut last_index = 0;
    for (index, char) in line.char_indices() {
        if !fits_in_text_box_width(&line[..index]) {
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

fn fits_in_text_box_width(line: &str) -> bool {
    measure_text(line, None, TEXT_BOX_TEXT_SIZE, 1.).width <= TEXT_BOX_TEXT_MAX_WIDTH
}

fn draw_text_box(text: &Vec<String>) {
    if text.is_empty() {
        return;
    }
    let text_box_size = f32::max(100., 10. + text.len() as f32 * TEXT_BOX_TEXT_SIZE as f32);
    let text_box_start = 600. - 25. - text_box_size;
    draw_rectangle(0., text_box_start, 800., text_box_size, TEXT_BOX_COLOR);
    for (index, text_line) in text.iter().enumerate() {
        draw_text(
            text_line,
            TEXT_BOX_MARGIN,
            text_box_start + ((index + 1) as f32 * TEXT_BOX_TEXT_SIZE as f32),
            TEXT_BOX_TEXT_SIZE as f32,
            WHITE,
        );
    }
}

fn ui(app: &mut App, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("input").show(ctx, |ui| input_field(app, ctx, ui));

    if !app.show_graphical {
        egui::CentralPanel::default().show(ctx, |ui| text_box(&app.render_state, ui));
    }
}

fn input_field(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let response = ui.add_enabled(
        app.ready_to_take_input(),
        egui::TextEdit::singleline(&mut app.input)
            .font(FONT)
            .desired_width(f32::INFINITY),
    );

    if response.lost_focus() && ui.input(|input_state| input_state.key_pressed(egui::Key::Enter)) {
        app.handle_input();
        ctx.memory_mut(|memory| memory.request_focus(response.id));
    }
}

fn text_box(state: &State, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for text in &state.text_lines {
                ui.label(egui::RichText::new(text).font(FONT));
            }
        });
}
