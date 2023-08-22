use crate::macroquad_interface::texture::TextureStorage;
use crate::macroquad_interface::{render, texture, ui, App};
use crate::view::{Frame, ObjectRenderData, RenderData};
use egui_macroquad::macroquad::camera::Camera2D;
use egui_macroquad::macroquad::color::WHITE;
use egui_macroquad::macroquad::input::MouseButton;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::{camera, input, shapes, text};
use std::collections::HashSet;

pub struct CommandTooltip {
    pos: Vec2,
    commands: Vec<String>,
}

pub fn try_tooltip_click(app: &mut App, textures: &TextureStorage) {
    let state = &mut app.render_state;
    if !app.game.ready_to_take_input() {
        app.command_tooltip = None;
        return;
    }
    if !input::is_mouse_button_pressed(MouseButton::Left) {
        return;
    }
    let mouse_pos = Vec2::from(input::mouse_position());
    let offset_pos = mouse_pos + Vec2::new(state.camera.x, state.camera.y);
    match &app.command_tooltip {
        None => {
            if let Frame::AreaView { render_data, .. } = &state.current_frame {
                let hovered_objects = render::position_objects(&render_data.objects, textures)
                    .into_iter()
                    .filter(|(pos, data)| {
                        texture::get_rect_for_object(data, textures, *pos).contains(offset_pos)
                    })
                    .map(|(_, data)| data)
                    .collect::<Vec<_>>();
                if !hovered_objects.is_empty() {
                    app.command_tooltip = prepare_command_tooltip(offset_pos, hovered_objects);
                }
            }
        }
        Some(command_tooltip) => {
            let line =
                clicked_tooltip_line(offset_pos, command_tooltip.pos, &command_tooltip.commands)
                    .cloned();
            app.command_tooltip = None;
            if let Some(line) = line {
                app.input = line;
                app.handle_input();
            }
        }
    }
}

fn prepare_command_tooltip(pos: Vec2, objects: Vec<&ObjectRenderData>) -> Option<CommandTooltip> {
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

pub fn draw_tooltips(
    state: &render::State,
    command_tooltip: &Option<CommandTooltip>,
    textures: &TextureStorage,
) {
    if let Frame::AreaView { render_data, .. } = &state.current_frame {
        camera::set_camera(&Camera2D::from_display_rect(state.camera));
        if let Some(tooltip) = command_tooltip {
            draw_tooltip(tooltip.pos, &tooltip.commands);
        } else {
            find_and_draw_tooltip(
                render_data,
                textures,
                Vec2::new(state.camera.x, state.camera.y),
            );
        }
    }
}

fn find_and_draw_tooltip(render_data: &RenderData, textures: &TextureStorage, camera_offset: Vec2) {
    let mouse_pos = Vec2::from(input::mouse_position()) + camera_offset;
    let hovered_objects = render::position_objects(&render_data.objects, textures)
        .into_iter()
        .filter(|(pos, data)| {
            texture::get_rect_for_object(data, textures, *pos).contains(mouse_pos)
        })
        .map(|(_, data)| &data.modified_name)
        .collect::<Vec<_>>();

    if hovered_objects.is_empty() {
        return;
    }

    draw_tooltip(mouse_pos, &hovered_objects);
}

fn clicked_tooltip_line<S: AsRef<str>>(mouse_pos: Vec2, pos: Vec2, lines: &Vec<S>) -> Option<&S> {
    let size = tooltip_size(pos, lines);

    for (index, line) in lines.iter().enumerate() {
        let line_size = Rect::new(
            size.x,
            size.y + index as f32 * ui::TEXT_BOX_TEXT_SIZE as f32,
            size.w,
            ui::TEXT_BOX_TEXT_SIZE as f32,
        );
        if line_size.contains(mouse_pos) {
            return Some(line);
        }
    }
    None
}

fn draw_tooltip<S: AsRef<str>>(pos: Vec2, lines: &Vec<S>) {
    let size = tooltip_size(pos, lines);
    shapes::draw_rectangle(size.x, size.y, size.w, size.h, ui::TEXT_BOX_COLOR);

    for (index, line) in lines.iter().enumerate() {
        text::draw_text(
            line.as_ref(),
            size.x + ui::TEXT_BOX_MARGIN,
            size.y + ((index + 1) as f32 * ui::TEXT_BOX_TEXT_SIZE as f32),
            ui::TEXT_BOX_TEXT_SIZE as f32,
            WHITE,
        );
    }
}

fn tooltip_size<S: AsRef<str>>(pos: Vec2, lines: &Vec<S>) -> Rect {
    let width = lines
        .iter()
        .map(|object| text::measure_text(object.as_ref(), None, ui::TEXT_BOX_TEXT_SIZE, 1.).width)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        + 2. * ui::TEXT_BOX_MARGIN;
    let height = 8. + lines.len() as f32 * ui::TEXT_BOX_TEXT_SIZE as f32;
    Rect::new(pos.x, pos.y, width, height)
}
