use super::texture::{draw_object, BGTextureType, TextureStorage};
use super::App;
use crate::core::position::Coord;
use crate::core::StopType;
use crate::macroquad_interface::texture::draw_background;
use crate::macroquad_interface::{tooltip, ui};
use crate::view::{Frame, Messages, ObjectRenderData, RenderData};
use egui_macroquad::macroquad::camera::{set_camera, set_default_camera, Camera2D};
use egui_macroquad::macroquad::color::{BLACK, LIGHTGRAY, WHITE};
use egui_macroquad::macroquad::input::{
    is_mouse_button_down, is_mouse_button_pressed, mouse_position, MouseButton,
};
use egui_macroquad::macroquad::math::{clamp, Rect, Vec2};
use egui_macroquad::macroquad::text::draw_text;
use egui_macroquad::macroquad::window::clear_background;
use std::collections::HashMap;

pub struct State {
    pub text_log: Vec<String>,
    pub current_frame: Frame,
    pub text_box_text: Vec<String>,
    pub camera: Rect,
}

impl State {
    pub fn new() -> Self {
        Self {
            text_log: vec![],
            current_frame: Frame::Introduction,
            text_box_text: vec![],
            camera: default_camera_space(),
        }
    }

    pub fn show_frame(&mut self, frame: Frame, ready_for_input: bool) {
        if let Frame::AreaView { render_data, .. } = &frame {
            self.camera = character_centered_camera(render_data);
        }

        self.text_log.extend(frame.as_text());
        if ready_for_input {
            self.text_log.push(String::default())
        }
        self.set_text_box_text(frame.get_messages());

        self.current_frame = frame;
    }

    fn set_text_box_text(&mut self, text: Vec<String>) {
        self.text_box_text = text.into_iter().flat_map(ui::split_text_line).collect();
    }

    pub fn add_to_text_log(&mut self, text: String) {
        self.text_log.push(text);
    }

    pub fn show_input_error(&mut self, messages: Messages) {
        let text = messages.into_text();
        self.text_log.extend(text.clone());
        self.set_text_box_text(text);
    }
}

pub fn draw(app: &mut App, textures: &TextureStorage) {
    clear_background(BLACK);

    if app.show_graphical {
        draw_frame(
            &app.render_state.current_frame,
            app.render_state.camera,
            textures,
        );

        ui::draw_text_box(
            &app.render_state.text_box_text,
            textures,
            app.game.next_result().has_frame(),
        );

        tooltip::draw_tooltips(&app.render_state, &app.command_tooltip, textures);
    }

    egui_macroquad::ui(|ctx| ui::egui_ui(app, ctx));

    egui_macroquad::draw();
}

fn draw_frame(frame: &Frame, camera: Rect, textures: &TextureStorage) {
    match frame {
        Frame::LocationChoice(_) | Frame::Introduction => {
            draw_background(
                BGTextureType::LocationChoice,
                0,
                default_camera_space(),
                textures,
            );
        }
        Frame::AreaView { render_data, .. } => {
            set_camera(&Camera2D::from_display_rect(camera));
            draw_background(
                render_data
                    .background
                    .map_or(BGTextureType::Blank, BGTextureType::from),
                render_data.background_offset.unwrap_or(0),
                camera,
                textures,
            );

            draw_objects(render_data, textures);

            set_default_camera();

            ui::draw_camera_arrows(textures.side_arrow, has_camera_space(camera, render_data));
        }
        Frame::StoreView { view, .. } => {
            const TEXT_SIZE: f32 = 32.;
            for (index, text_line) in view.messages().into_text().iter().enumerate() {
                draw_text(
                    text_line,
                    200.,
                    100. + ((index + 1) as f32 * TEXT_SIZE),
                    TEXT_SIZE,
                    WHITE,
                );
            }
        }
        Frame::Ending { stop_type } => {
            let color = match stop_type {
                StopType::Win => LIGHTGRAY,
                StopType::Lose => BLACK,
            };
            clear_background(color);
        }
    }
}

fn draw_objects(render_data: &RenderData, textures: &TextureStorage) {
    for (pos, data) in position_objects(&render_data.objects, textures) {
        draw_object(
            data.texture_type,
            data.direction,
            data.aftik_color,
            false,
            textures,
            pos,
        );
        if let Some(item_texture) = data.wielded_item {
            draw_object(item_texture, data.direction, None, true, textures, pos);
        }
    }
}

pub fn position_objects<'a>(
    objects: &'a Vec<ObjectRenderData>,
    textures: &TextureStorage,
) -> Vec<(Vec2, &'a ObjectRenderData)> {
    let mut positioned_objects = Vec::new();
    let mut coord_counts: HashMap<Coord, i32> = HashMap::new();

    for data in objects {
        let coord = data.coord;
        let count = if textures.lookup_texture(data.texture_type).is_displacing() {
            let count_ref = coord_counts.entry(coord).or_insert(0);
            let count = *count_ref;
            *count_ref = count + 1;
            count
        } else {
            0
        };

        positioned_objects.push((
            Vec2::new(
                coord_to_center_x(coord) - count as f32 * 15.,
                (450 + count * 10) as f32,
            ),
            data,
        ));
    }
    positioned_objects
}

// Coordinates are mapped like this so that when the left edge of the window is 0,
// coord 3 will be placed in the middle of the window.
fn coord_to_center_x(coord: Coord) -> f32 {
    40. + 120. * coord as f32
}

pub fn try_drag_camera(state: &mut State, last_drag_pos: &mut Option<Vec2>) {
    match (&state.current_frame, *last_drag_pos) {
        (Frame::AreaView { render_data, .. }, Some(last_pos)) => {
            if is_mouse_button_down(MouseButton::Left) {
                let mouse_pos: Vec2 = mouse_position().into();
                let camera_delta = mouse_pos - last_pos;

                state.camera.x -= camera_delta.x;
                clamp_camera(&mut state.camera, render_data);
                *last_drag_pos = Some(mouse_pos);
            } else {
                *last_drag_pos = None;
            }
        }
        (Frame::AreaView { .. }, None) => {
            if is_mouse_button_pressed(MouseButton::Left) && !ui::is_mouse_at_text_box(state) {
                *last_drag_pos = Some(mouse_position().into());
            }
        }
        _ => {
            *last_drag_pos = None;
        }
    }
}

fn has_camera_space(camera: Rect, render_data: &RenderData) -> [bool; 2] {
    if render_data.area_size <= 6 {
        [false, false]
    } else {
        [
            camera.left() > coord_to_center_x(0) - 100.,
            camera.right() < coord_to_center_x(render_data.area_size - 1) + 100.,
        ]
    }
}

fn clamp_camera(camera: &mut Rect, render_data: &RenderData) {
    camera.x = if render_data.area_size <= 6 {
        (coord_to_center_x(0) + coord_to_center_x(render_data.area_size - 1)) / 2. - camera.w / 2.
    } else {
        clamp(
            camera.x,
            coord_to_center_x(0) - 100.,
            coord_to_center_x(render_data.area_size - 1) + 100. - camera.w,
        )
    };
}

fn default_camera_space() -> Rect {
    Rect::new(0., 0., 800., 600.)
}

fn character_centered_camera(render_data: &RenderData) -> Rect {
    let mut camera_space = Rect::new(
        coord_to_center_x(render_data.character_coord) - 400.,
        0.,
        800.,
        600.,
    );
    clamp_camera(&mut camera_space, render_data);
    camera_space
}
