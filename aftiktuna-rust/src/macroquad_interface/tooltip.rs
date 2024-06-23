use crate::command::suggestion;
use crate::command::suggestion::Suggestion;
use crate::view::area::RenderData;
use crate::view::Frame;
use egui_macroquad::macroquad::color::{Color, WHITE};
use egui_macroquad::macroquad::input::MouseButton;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::{input, shapes, text};

use super::texture::LazilyLoadedModels;
use super::{camera, render, store_render, App};

trait TextRepresentable {
    fn as_text(&self) -> &str;
}

impl TextRepresentable for String {
    fn as_text(&self) -> &str {
        self
    }
}

impl TextRepresentable for Suggestion {
    fn as_text(&self) -> &str {
        self.text()
    }
}

impl<T: TextRepresentable> TextRepresentable for &T {
    fn as_text(&self) -> &str {
        (*self).as_text()
    }
}

pub struct CommandTooltip {
    pos: Vec2,
    commands: Vec<Suggestion>,
}

impl CommandTooltip {
    pub fn dimensions(&self) -> Rect {
        tooltip_dimensions(self.pos, &self.commands)
    }
}

pub fn handle_click(app: &mut App) {
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
            let commands = find_raw_command_suggestions(mouse_pos, state, &mut app.assets.models);
            if !commands.is_empty() {
                app.command_tooltip = Some(CommandTooltip {
                    pos: mouse_pos,
                    commands: suggestion::sorted_without_duplicates(commands),
                });
            }
        }
        Some(command_tooltip) => {
            let line_index =
                line_index_at(mouse_pos, command_tooltip.pos, &command_tooltip.commands);
            if let Some(line_index) = line_index {
                match &command_tooltip.commands[line_index] {
                    Suggestion::Simple(command) => {
                        app.input.clone_from(command);
                        app.handle_input();
                        app.command_tooltip = None;
                    }
                    Suggestion::Recursive(_, suggestions) => {
                        app.command_tooltip = Some(CommandTooltip {
                            pos: command_tooltip.pos,
                            commands: suggestions.clone(),
                        });
                    }
                }
            } else {
                app.command_tooltip = None;
            }
        }
    }
}

fn find_raw_command_suggestions(
    mouse_pos: Vec2,
    state: &render::State,
    models: &mut LazilyLoadedModels,
) -> Vec<Suggestion> {
    match &state.current_frame {
        Frame::AreaView { render_data, .. } => {
            let offset_pos = mouse_pos + Vec2::new(state.camera.x, state.camera.y);

            camera::position_objects(&render_data.objects, models)
                .into_iter()
                .filter(|(pos, data)| models.get_rect_for_object(data, *pos).contains(offset_pos))
                .filter_map(|(_, data)| data.name_data.as_ref().zip(Some(&data.interactions)))
                .flat_map(|(name_data, interactions)| {
                    interactions.iter().flat_map(|interaction| {
                        interaction.commands(&name_data.name, &render_data.inventory)
                    })
                })
                .collect::<Vec<_>>()
        }
        Frame::StoreView { view, .. } => suggestion::for_store(
            store_render::find_stock_at(mouse_pos, view),
            &view.sellable_items,
        ),
        Frame::LocationChoice(choice) => suggestion::for_location_choice(choice),
        _ => vec![],
    }
}

pub fn draw(
    state: &render::State,
    command_tooltip: &Option<CommandTooltip>,
    models: &mut LazilyLoadedModels,
) {
    let mouse_pos = Vec2::from(input::mouse_position());
    if let Some(tooltip) = command_tooltip {
        draw_lines(
            tooltip.pos,
            &tooltip.commands,
            line_index_at(mouse_pos, tooltip.pos, &tooltip.commands),
        );
    } else if let Frame::AreaView { render_data, .. } = &state.current_frame {
        let camera_offset = Vec2::new(state.camera.x, state.camera.y);
        let names = get_hovered_object_names(render_data, models, mouse_pos + camera_offset);
        draw_lines(mouse_pos, &names, None);
    }
}

fn get_hovered_object_names<'a>(
    render_data: &'a RenderData,
    models: &mut LazilyLoadedModels,
    mouse_pos: Vec2,
) -> Vec<&'a String> {
    camera::position_objects(&render_data.objects, models)
        .into_iter()
        .filter(|(pos, data)| models.get_rect_for_object(data, *pos).contains(mouse_pos))
        .filter_map(|(_, data)| data.name_data.as_ref())
        .map(|name_data| &name_data.modified_name)
        .collect::<Vec<_>>()
}

const TEXT_BOX_COLOR: Color = Color::new(0.2, 0.1, 0.4, 0.6);
const TEXT_BOX_HIGHLIGHT_COLOR: Color = Color::new(0.5, 0.3, 0.6, 0.6);
const TEXT_BOX_TEXT_SIZE: u16 = 16;
const TEXT_BOX_MARGIN: f32 = 10.;

fn line_index_at<S: TextRepresentable>(mouse_pos: Vec2, pos: Vec2, lines: &[S]) -> Option<usize> {
    let size = tooltip_dimensions(pos, lines);

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

fn draw_lines<S: TextRepresentable>(pos: Vec2, lines: &[S], highlighted_index: Option<usize>) {
    if lines.is_empty() {
        return;
    }

    let size = tooltip_dimensions(pos, lines);
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
            line.as_text(),
            size.x + TEXT_BOX_MARGIN,
            size.y - 4.0 + ((index + 1) as f32 * TEXT_BOX_TEXT_SIZE as f32),
            TEXT_BOX_TEXT_SIZE as f32,
            WHITE,
        );
    }
}

fn tooltip_dimensions<S: TextRepresentable>(pos: Vec2, lines: &[S]) -> Rect {
    let width = lines
        .iter()
        .map(|object| text::measure_text(object.as_text(), None, TEXT_BOX_TEXT_SIZE, 1.).width)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        + 2. * TEXT_BOX_MARGIN;
    let height = lines.len() as f32 * TEXT_BOX_TEXT_SIZE as f32;
    Rect::new(f32::min(pos.x, 800. - width), pos.y, width, height)
}
