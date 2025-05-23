use super::egui::EguiWrapper;
use super::texture::RenderAssets;
use super::App;
use macroquad::color::{Color, WHITE};
use macroquad::input::mouse_position;
use macroquad::math::{Rect, Vec2};
use macroquad::shapes::draw_rectangle;
use macroquad::text::{draw_text, measure_text};
use macroquad::texture::{draw_texture, draw_texture_ex, DrawTextureParams, Texture2D};
use macroquad::time::get_time;

const TEXT_BOX_COLOR: Color = Color::new(0.2, 0.1, 0.4, 0.6);
const TEXT_BOX_TEXT_SIZE: u16 = 16;
const TEXT_BOX_MARGIN: f32 = 12.;
const TEXT_BOX_TEXT_MAX_WIDTH: f32 = super::WINDOW_WIDTH_F - 2. * TEXT_BOX_MARGIN;

fn set_egui_style(style: &mut egui::Style) {
    style.spacing.scroll = egui::style::ScrollStyle::solid();
}

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

pub fn is_mouse_at_text_box(text_box_text: &[String]) -> bool {
    get_text_box_dimensions(text_box_text).contains(Vec2::from(mouse_position()))
}

fn get_text_box_dimensions(text: &[String]) -> Rect {
    let text_box_size = f32::max(100., 10. + text.len() as f32 * TEXT_BOX_TEXT_SIZE as f32);
    let text_box_start = super::WINDOW_HEIGHT_F - 25. - text_box_size;
    Rect::new(0., text_box_start, super::WINDOW_WIDTH_F, text_box_size)
}

pub fn draw_text_box(text: &[String], textures: &RenderAssets, click_to_proceed: bool) {
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
            &textures.left_mouse_icon,
            dimensions.right() - textures.left_mouse_icon.width() - 5.,
            dimensions.top() + 5.,
            Color::new(1., 1., 1., alpha as f32),
        );
    }
}

const FONT: egui::FontId = egui::FontId::monospace(15.0);

pub fn egui_graphic(app: &mut App, egui: &mut EguiWrapper) {
    egui.ui(|ctx| {
        ctx.style_mut(set_egui_style);

        input_panel(app, ctx);

        if let Some(tooltip) = &app.command_tooltip {
            let dimensions = tooltip.dimensions();
            egui::Area::new(egui::Id::new("tooltip"))
                .order(egui::Order::Foreground)
                .fixed_pos(egui::pos2(
                    dimensions.x / ctx.pixels_per_point(),
                    dimensions.y / ctx.pixels_per_point(),
                ))
                .show(ctx, |ui| {
                    ui.set_min_size(egui::vec2(
                        dimensions.w / ctx.pixels_per_point(),
                        dimensions.h / ctx.pixels_per_point(),
                    ))
                });
        }
    });

    egui.draw();
}

pub fn egui_text_view(app: &mut App, egui: &mut EguiWrapper) {
    egui.ui(|ctx| {
        ctx.style_mut(set_egui_style);

        input_panel(app, ctx);
        text_box_panel(app, ctx);
    });

    egui.draw();
}

fn input_panel(app: &mut App, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("input").show(ctx, |ui| egui_input_field(app, ui));
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

fn text_box_panel(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| egui_text_box(&app.render_state.text_log, ui));
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

pub(crate) fn draw_camera_arrows(side_arrow: &Texture2D, has_camera_space: [bool; 2]) {
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
