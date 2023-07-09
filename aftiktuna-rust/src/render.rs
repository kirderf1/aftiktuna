use crate::{App, GameState};
use aftiktuna::position::{Coord, Direction};
use aftiktuna::view::{RenderData, TextureType};
use egui_macroquad::egui;
use macroquad::color::{BLACK, WHITE};
use macroquad::math::{bool, f32, i32, Vec2};
use macroquad::prelude::{
    clear_background, draw_texture, draw_texture_ex, load_texture, DrawTextureParams, Texture2D,
};
use std::collections::HashMap;
use std::mem::take;

const FONT: egui::FontId = egui::FontId::monospace(15.0);

pub struct TextureStorage {
    background: Texture2D,
    selection_background: Texture2D,
    by_type: HashMap<TextureType, TextureData>,
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

fn texture_path(name: &str) -> String {
    format!("assets/textures/{}.png", name)
}

pub async fn load_textures() -> TextureStorage {
    let background = load_texture(&texture_path("tree_background"))
        .await
        .unwrap();
    let selection_background = load_texture(&texture_path("selection_background"))
        .await
        .unwrap();
    let unknown = load_texture(&texture_path("unknown")).await.unwrap();
    let path = load_texture(&texture_path("path")).await.unwrap();
    let aftik = load_texture(&texture_path("aftik")).await.unwrap();
    let eyesaur = load_texture(&texture_path("eyesaur")).await.unwrap();

    let mut textures = HashMap::new();

    textures.insert(TextureType::Unknown, TextureData::new_static(unknown));
    textures.insert(
        TextureType::SmallUnknown,
        TextureData {
            texture: unknown,
            dest_size: Vec2::new(100., 100.),
            directional: false,
        },
    );
    textures.insert(TextureType::Path, TextureData::new_static(path));
    textures.insert(TextureType::Aftik, TextureData::new_directional(aftik));
    textures.insert(TextureType::Goblin, TextureData::new_static(unknown));
    textures.insert(TextureType::Eyesaur, TextureData::new_directional(eyesaur));
    textures.insert(TextureType::Azureclops, TextureData::new_static(unknown));

    TextureStorage {
        background,
        selection_background,
        by_type: textures,
    }
}

fn lookup_texture(
    textures: &HashMap<TextureType, TextureData>,
    texture_type: TextureType,
) -> &TextureData {
    if let Some(data) = textures.get(&texture_type) {
        data
    } else if let TextureType::Item(_) = texture_type {
        textures.get(&TextureType::SmallUnknown).unwrap()
    } else {
        textures.get(&TextureType::Unknown).unwrap()
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

fn draw_game(app: &mut App, textures: &TextureStorage) {
    match &app.render_state {
        State::LocationChoice => {
            draw_texture(textures.selection_background, 0., 0., WHITE);
        }
        State::InGame(render_data) => {
            draw_texture(textures.background, 0., 0., WHITE);

            draw_objects(render_data, &textures.by_type);
        }
    }
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
            lookup_texture(textures, data.texture_type),
            data.direction,
            start_x + ((coord as i32) * 120 - count * 15) as f32,
            (500 + count * 10) as f32,
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

fn ui(app: &mut App, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("input").show(ctx, |ui| input_field(app, ctx, ui));

    if !app.show_graphical {
        egui::CentralPanel::default().show(ctx, |ui| text_box(app, ui));
    }
}

fn input_field(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let response = ui.add_enabled(
        app.state == GameState::Input && app.delayed_views.is_none(),
        egui::TextEdit::singleline(&mut app.input)
            .font(FONT)
            .desired_width(f32::INFINITY),
    );

    if response.lost_focus() && ui.input(|input_state| input_state.key_pressed(egui::Key::Enter)) {
        let input = take(&mut app.input);
        if !input.is_empty() {
            app.text_lines.push(format!("> {input}"));
            if let Err(messages) = app.game.handle_input(&input) {
                app.text_lines.extend(messages.into_text());
            } else {
                app.state = GameState::Run;
            }
        }
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
