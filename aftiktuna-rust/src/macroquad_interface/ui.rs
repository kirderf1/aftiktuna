use crate::macroquad_interface::render::State as RenderState;
use crate::macroquad_interface::texture::TextureStorage;
use crate::macroquad_interface::App;
use egui_macroquad::egui;
use egui_macroquad::macroquad::color::{Color, WHITE};
use egui_macroquad::macroquad::input::mouse_position;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::shapes::draw_rectangle;
use egui_macroquad::macroquad::text::{draw_text, measure_text};
use egui_macroquad::macroquad::texture::{
    draw_texture, draw_texture_ex, DrawTextureParams, Texture2D,
};
use egui_macroquad::macroquad::time::get_time;

pub const TEXT_BOX_COLOR: Color = Color::new(0.2, 0.1, 0.4, 0.6);
pub const TEXT_BOX_TEXT_SIZE: u16 = 16;
pub const TEXT_BOX_MARGIN: f32 = 12.;
const TEXT_BOX_TEXT_MAX_WIDTH: f32 = 800. - 2. * TEXT_BOX_MARGIN;

pub fn split_text_line(line: String) -> Vec<String> {
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

pub fn is_mouse_at_text_box(state: &RenderState) -> bool {
    get_text_box_dimensions(&state.text_box_text).contains(Vec2::from(mouse_position()))
}

fn get_text_box_dimensions(text: &Vec<String>) -> Rect {
    let text_box_size = f32::max(100., 10. + text.len() as f32 * TEXT_BOX_TEXT_SIZE as f32);
    let text_box_start = 600. - 25. - text_box_size;
    Rect::new(0., text_box_start, 800., text_box_size)
}

pub fn draw_text_box(text: &Vec<String>, textures: &TextureStorage, click_to_proceed: bool) {
    if text.is_empty() {
        return;
    }
    let dimensions = get_text_box_dimensions(text);
    draw_rectangle(
        dimensions.x,
        dimensions.y,
        dimensions.w,
        dimensions.h,
        TEXT_BOX_COLOR,
    );
    for (index, text_line) in text.iter().enumerate() {
        draw_text(
            text_line,
            dimensions.x + TEXT_BOX_MARGIN,
            dimensions.y + ((index + 1) as f32 * TEXT_BOX_TEXT_SIZE as f32),
            TEXT_BOX_TEXT_SIZE as f32,
            WHITE,
        );
    }

    if click_to_proceed {
        let alpha = ((get_time() * 3.).sin() + 1.) / 2.;
        draw_texture(
            textures.left_mouse_icon,
            dimensions.right() - textures.left_mouse_icon.width() - 5.,
            dimensions.top() + 5.,
            Color::new(1., 1., 1., alpha as f32),
        );
    }
}

const FONT: egui::FontId = egui::FontId::monospace(15.0);

pub fn egui_ui(app: &mut App, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("input").show(ctx, |ui| egui_input_field(app, ui));

    if !app.show_graphical {
        egui::CentralPanel::default().show(ctx, |ui| egui_text_box(&app.render_state.text_log, ui));
    }
}

fn egui_input_field(app: &mut App, ui: &mut egui::Ui) {
    let response = ui.add_enabled(
        app.game.ready_to_take_input(),
        egui::TextEdit::singleline(&mut app.input)
            .font(FONT)
            .desired_width(f32::INFINITY)
            .lock_focus(true),
    );

    if response.lost_focus() && ui.input(|input_state| input_state.key_pressed(egui::Key::Enter)) {
        app.handle_input();
    }

    if app.request_input_focus {
        response.request_focus();
        app.request_input_focus = false;
    }
}

fn egui_text_box(text_log: &Vec<String>, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for text in text_log {
                ui.label(egui::RichText::new(text).font(FONT));
            }
        });
}

pub(crate) fn draw_camera_arrows(side_arrow: Texture2D, has_camera_space: [bool; 2]) {
    let alpha = ((get_time() * 3.).sin() + 1.) / 2.;
    let color = Color::new(1., 1., 1., alpha as f32);
    if has_camera_space[0] {
        draw_texture_ex(
            side_arrow,
            10.,
            250. - side_arrow.height() / 2.,
            color,
            DrawTextureParams {
                flip_x: true,
                ..Default::default()
            },
        )
    }
    if has_camera_space[1] {
        draw_texture_ex(
            side_arrow,
            790. - side_arrow.width(),
            250. - side_arrow.height() / 2.,
            color,
            DrawTextureParams {
                flip_x: false,
                ..Default::default()
            },
        )
    }
}
