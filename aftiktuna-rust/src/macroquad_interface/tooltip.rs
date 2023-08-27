use crate::macroquad_interface::texture::TextureStorage;
use crate::macroquad_interface::{render, store_render, texture, App};
use crate::view::{Frame, ObjectRenderData, RenderData};
use egui_macroquad::macroquad::camera::Camera2D;
use egui_macroquad::macroquad::color::{Color, WHITE};
use egui_macroquad::macroquad::input::MouseButton;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::{camera, input, shapes, text};
use std::collections::HashSet;

pub struct CommandTooltip {
    pos: Vec2,
    commands: Vec<String>,
}

pub fn handle_click(app: &mut App, textures: &TextureStorage) {
    let state = &mut app.render_state;
    if !app.game.ready_to_take_input() {
        app.command_tooltip = None;
        return;
    }
    if !input::is_mouse_button_pressed(MouseButton::Left) {
        return;
    }
    let mouse_pos = Vec2::from(input::mouse_position());
    match &app.command_tooltip {
        None => {
            if let Frame::AreaView { render_data, .. } = &state.current_frame {
                let offset_pos = mouse_pos + Vec2::new(state.camera.x, state.camera.y);
                let hovered_objects = render::position_objects(&render_data.objects, textures)
                    .into_iter()
                    .filter(|(pos, data)| {
                        texture::get_rect_for_object(data, textures, *pos).contains(offset_pos)
                    })
                    .map(|(_, data)| data)
                    .collect::<Vec<_>>();
                if !hovered_objects.is_empty() {
                    app.command_tooltip = prepare_command_data(mouse_pos, hovered_objects);
                }
            } else if let Frame::StoreView { view, .. } = &state.current_frame {
                if let Some(priced_item) = store_render::find_stock_at(mouse_pos, view) {
                    app.command_tooltip = Some(CommandTooltip {
                        pos: mouse_pos,
                        commands: priced_item.command_suggestions(),
                    });
                }
            }
        }
        Some(command_tooltip) => {
            let line_index =
                line_index_at(mouse_pos, command_tooltip.pos, &command_tooltip.commands);
            if let Some(line_index) = line_index {
                app.input = command_tooltip.commands[line_index].clone();
                app.handle_input();
            }
            app.command_tooltip = None;
        }
    }
}

fn prepare_command_data(pos: Vec2, objects: Vec<&ObjectRenderData>) -> Option<CommandTooltip> {
    let mut commands = objects
        .into_iter()
        .flat_map(|object| {
            object
                .interactions
                .iter()
                .flat_map(|interaction| interaction.commands(&object.name))
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    commands.sort();
    if commands.is_empty() {
        None
    } else {
        Some(CommandTooltip { pos, commands })
    }
}

pub fn draw(
    state: &render::State,
    command_tooltip: &Option<CommandTooltip>,
    textures: &TextureStorage,
) {
    let mouse_pos = Vec2::from(input::mouse_position());
    if let Some(tooltip) = command_tooltip {
        camera::set_default_camera();
        draw_lines(
            tooltip.pos,
            &tooltip.commands,
            line_index_at(mouse_pos, tooltip.pos, &tooltip.commands),
        );
    } else if let Frame::AreaView { render_data, .. } = &state.current_frame {
        camera::set_camera(&Camera2D::from_display_rect(state.camera));
        let camera_offset = Vec2::new(state.camera.x, state.camera.y);
        let mouse_pos = mouse_pos + camera_offset;
        let names = get_hovered_object_names(render_data, textures, mouse_pos);
        draw_lines(mouse_pos, &names, None);
    }
}

fn get_hovered_object_names<'a>(
    render_data: &'a RenderData,
    textures: &TextureStorage,
    mouse_pos: Vec2,
) -> Vec<&'a String> {
    render::position_objects(&render_data.objects, textures)
        .into_iter()
        .filter(|(pos, data)| {
            texture::get_rect_for_object(data, textures, *pos).contains(mouse_pos)
        })
        .map(|(_, data)| &data.modified_name)
        .collect::<Vec<_>>()
}

const TEXT_BOX_COLOR: Color = Color::new(0.2, 0.1, 0.4, 0.6);
const TEXT_BOX_HIGHLIGHT_COLOR: Color = Color::new(0.5, 0.3, 0.6, 0.6);
const TEXT_BOX_TEXT_SIZE: u16 = 16;
const TEXT_BOX_MARGIN: f32 = 10.;

fn line_index_at<S: AsRef<str>>(mouse_pos: Vec2, pos: Vec2, lines: &Vec<S>) -> Option<usize> {
    let size = tooltip_size(pos, lines);

    for index in 0..lines.len() {
        let line_size = Rect::new(
            size.x,
            size.y + index as f32 * TEXT_BOX_TEXT_SIZE as f32,
            size.w,
            TEXT_BOX_TEXT_SIZE as f32,
        );
        if line_size.contains(mouse_pos) {
            return Some(index);
        }
    }
    None
}

fn draw_lines<S: AsRef<str>>(pos: Vec2, lines: &Vec<S>, highlighted_index: Option<usize>) {
    if lines.is_empty() {
        return;
    }

    let size = tooltip_size(pos, lines);
    if let Some(line_index) = highlighted_index {
        let highlight_start = line_index as f32 * TEXT_BOX_TEXT_SIZE as f32;
        let highlight_end = (line_index + 1) as f32 * TEXT_BOX_TEXT_SIZE as f32;
        shapes::draw_rectangle(size.x, size.y, size.w, highlight_start, TEXT_BOX_COLOR);
        shapes::draw_rectangle(
            size.x,
            size.y + highlight_start,
            size.w,
            highlight_end - highlight_start,
            TEXT_BOX_HIGHLIGHT_COLOR,
        );
        shapes::draw_rectangle(
            size.x,
            size.y + highlight_end,
            size.w,
            size.h - highlight_end,
            TEXT_BOX_COLOR,
        );
    } else {
        shapes::draw_rectangle(size.x, size.y, size.w, size.h, TEXT_BOX_COLOR);
    }

    for (index, line) in lines.iter().enumerate() {
        text::draw_text(
            line.as_ref(),
            size.x + TEXT_BOX_MARGIN,
            size.y - 4.0 + ((index + 1) as f32 * TEXT_BOX_TEXT_SIZE as f32),
            TEXT_BOX_TEXT_SIZE as f32,
            WHITE,
        );
    }
}

fn tooltip_size<S: AsRef<str>>(pos: Vec2, lines: &Vec<S>) -> Rect {
    let width = lines
        .iter()
        .map(|object| text::measure_text(object.as_ref(), None, TEXT_BOX_TEXT_SIZE, 1.).width)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        + 2. * TEXT_BOX_MARGIN;
    let height = lines.len() as f32 * TEXT_BOX_TEXT_SIZE as f32;
    Rect::new(pos.x, pos.y, width, height)
}
