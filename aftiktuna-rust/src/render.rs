use crate::App;
use aftiktuna::area::BackgroundType;
use aftiktuna::item;
use aftiktuna::position::{Coord, Direction};
use aftiktuna::view::{RenderData, TextureType};
use egui_macroquad::egui;
use macroquad::camera::set_camera;
use macroquad::color::{BLACK, WHITE};
use macroquad::prelude::{
    clamp, clear_background, draw_rectangle, draw_text, draw_texture, draw_texture_ex,
    load_texture, set_default_camera, Camera2D, Color, DrawTextureParams, Rect, Texture2D, Vec2,
};
use std::collections::HashMap;

const FONT: egui::FontId = egui::FontId::monospace(15.0);

pub struct TextureStorage {
    backgrounds: HashMap<BGTextureType, BGTexture>,
    objects: HashMap<TextureType, TextureData>,
}

impl TextureStorage {
    fn lookup_background(&self, texture_type: BGTextureType) -> &BGTexture {
        self.backgrounds
            .get(&texture_type)
            .unwrap_or_else(|| self.backgrounds.get(&BGTextureType::Blank).unwrap())
    }

    fn lookup_texture(&self, texture_type: TextureType) -> &TextureData {
        if let Some(data) = self.objects.get(&texture_type) {
            data
        } else if let TextureType::Item(_) = texture_type {
            self.objects.get(&TextureType::SmallUnknown).unwrap()
        } else {
            self.objects.get(&TextureType::Unknown).unwrap()
        }
    }
}

pub enum State {
    LocationChoice,
    InGame(RenderData),
}

#[derive(Clone)]
struct TextureData {
    texture: Texture2D,
    dest_size: Vec2,
    directional: bool,
}

impl TextureData {
    fn new_static(texture: Texture2D) -> TextureData {
        TextureData {
            texture,
            dest_size: Vec2::new(texture.width(), texture.height()),
            directional: false,
        }
    }
    fn new_directional(texture: Texture2D) -> TextureData {
        TextureData {
            texture,
            dest_size: Vec2::new(texture.width(), texture.height()),
            directional: true,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
enum BGTextureType {
    LocationChoice,
    Forest,
    Blank,
}

impl From<BackgroundType> for BGTextureType {
    fn from(value: BackgroundType) -> Self {
        match value {
            BackgroundType::Forest => BGTextureType::Forest,
        }
    }
}

enum BGTexture {
    Simple(Texture2D),
    Repeating(Texture2D),
}

fn texture_path(name: &str) -> String {
    format!("assets/textures/{}.png", name)
}

pub async fn load_textures() -> TextureStorage {
    let forest_background = load_texture(&texture_path("tree_background"))
        .await
        .unwrap();
    let blank_background = load_texture(&texture_path("white_space")).await.unwrap();
    let selection_background = load_texture(&texture_path("selection_background"))
        .await
        .unwrap();
    let unknown = load_texture(&texture_path("unknown")).await.unwrap();
    let path = load_texture(&texture_path("path")).await.unwrap();
    let aftik = load_texture(&texture_path("aftik")).await.unwrap();
    let goblin = load_texture(&texture_path("goblin")).await.unwrap();
    let eyesaur = load_texture(&texture_path("eyesaur")).await.unwrap();
    let fuel_can = load_texture(&texture_path("fuel_can")).await.unwrap();
    let bat = load_texture(&texture_path("bat")).await.unwrap();

    let mut backgrounds = HashMap::new();

    backgrounds.insert(
        BGTextureType::LocationChoice,
        BGTexture::Simple(selection_background),
    );
    backgrounds.insert(
        BGTextureType::Forest,
        BGTexture::Repeating(forest_background),
    );
    backgrounds.insert(BGTextureType::Blank, BGTexture::Simple(blank_background));

    let mut objects = HashMap::new();

    objects.insert(TextureType::Unknown, TextureData::new_static(unknown));
    objects.insert(
        TextureType::SmallUnknown,
        TextureData {
            texture: unknown,
            dest_size: Vec2::new(100., 100.),
            directional: false,
        },
    );
    objects.insert(TextureType::Path, TextureData::new_static(path));
    objects.insert(TextureType::Aftik, TextureData::new_directional(aftik));
    objects.insert(TextureType::Goblin, TextureData::new_directional(goblin));
    objects.insert(TextureType::Eyesaur, TextureData::new_directional(eyesaur));
    objects.insert(
        TextureType::Item(item::Type::FuelCan),
        TextureData::new_static(fuel_can),
    );
    objects.insert(
        TextureType::Item(item::Type::Bat),
        TextureData::new_static(bat),
    );

    TextureStorage {
        backgrounds,
        objects,
    }
}

pub fn draw(app: &mut App, textures: &TextureStorage) {
    clear_background(BLACK);

    if app.show_graphical {
        draw_game(app, textures);
    }

    egui_macroquad::ui(|ctx| ui(app, ctx));

    egui_macroquad::draw();
}

fn default_camera_space() -> Rect {
    Rect::new(0., 0., 800., 600.)
}

fn draw_game(app: &mut App, textures: &TextureStorage) {
    match &app.render_state {
        State::LocationChoice => {
            draw_background(
                BGTextureType::LocationChoice,
                default_camera_space(),
                textures,
            );
        }
        State::InGame(render_data) => {
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

    draw_text_box(&app.text_box_text);
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

        draw_object(
            textures.lookup_texture(data.texture_type),
            data.direction,
            start_x + ((coord as i32) * 120 - count * 15) as f32,
            (450 + count * 10) as f32,
        );
    }
}

fn draw_object(data: &TextureData, direction: Direction, x: f32, y: f32) {
    let size = data.dest_size;
    draw_texture_ex(
        data.texture,
        x - size.x / 2.,
        y - size.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            flip_x: data.directional && direction == Direction::Left,
            ..Default::default()
        },
    );
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
const TEXT_BOX_TEXT_SIZE: f32 = 16.;

fn draw_text_box(text: &Vec<String>) {
    if text.is_empty() {
        return;
    }
    let text_box_size = f32::max(100., 10. + text.len() as f32 * TEXT_BOX_TEXT_SIZE);
    let text_box_start = 600. - 25. - text_box_size;
    draw_rectangle(0., text_box_start, 800., text_box_size, TEXT_BOX_COLOR);
    for (index, text_line) in text.iter().enumerate() {
        draw_text(
            text_line,
            8.,
            text_box_start + ((index + 1) as f32 * TEXT_BOX_TEXT_SIZE),
            TEXT_BOX_TEXT_SIZE,
            WHITE,
        );
    }
}

fn ui(app: &mut App, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("input").show(ctx, |ui| input_field(app, ctx, ui));

    if !app.show_graphical {
        egui::CentralPanel::default().show(ctx, |ui| text_box(app, ui));
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

fn text_box(app: &mut App, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for text in &app.text_lines {
                ui.label(egui::RichText::new(text).font(FONT));
            }
        });
}
