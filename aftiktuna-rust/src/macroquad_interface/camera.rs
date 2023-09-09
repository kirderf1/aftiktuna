use crate::core::position::Coord;
use crate::macroquad_interface::texture::TextureStorage;
use crate::macroquad_interface::{render, ui};
use crate::view::{Frame, ObjectRenderData, RenderData};
use egui_macroquad::macroquad::input::MouseButton;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::{input, math};
use std::collections::HashMap;

pub fn position_objects<'a>(
    objects: &'a Vec<ObjectRenderData>,
    textures: &mut TextureStorage,
) -> Vec<(Vec2, &'a ObjectRenderData)> {
    let mut positioned_objects = Vec::new();
    let mut coord_counts: HashMap<Coord, i32> = HashMap::new();

    for data in objects {
        let coord = data.coord;
        let count = if textures.lookup_texture(&data.texture_type).is_displacing() {
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

pub fn try_drag_camera(state: &mut render::State, last_drag_pos: &mut Option<Vec2>) {
    match (&state.current_frame, *last_drag_pos) {
        (Frame::AreaView { render_data, .. }, Some(last_pos)) => {
            if input::is_mouse_button_down(MouseButton::Left) {
                let mouse_pos: Vec2 = input::mouse_position().into();
                let camera_delta = mouse_pos - last_pos;

                state.camera.x -= camera_delta.x;
                clamp_camera(&mut state.camera, render_data);
                *last_drag_pos = Some(mouse_pos);
            } else {
                *last_drag_pos = None;
            }
        }
        (Frame::AreaView { .. }, None) => {
            if input::is_mouse_button_pressed(MouseButton::Left) && !ui::is_mouse_at_text_box(state)
            {
                *last_drag_pos = Some(input::mouse_position().into());
            }
        }
        _ => {
            *last_drag_pos = None;
        }
    }
}

pub fn has_camera_space(camera: Rect, render_data: &RenderData) -> [bool; 2] {
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
        math::clamp(
            camera.x,
            coord_to_center_x(0) - 100.,
            coord_to_center_x(render_data.area_size - 1) + 100. - camera.w,
        )
    };
}

pub fn default_camera_space() -> Rect {
    Rect::new(0., 0., 800., 600.)
}

pub fn character_centered_camera(render_data: &RenderData) -> Rect {
    let mut camera_space = Rect::new(
        coord_to_center_x(render_data.character_coord) - 400.,
        0.,
        800.,
        600.,
    );
    clamp_camera(&mut camera_space, render_data);
    camera_space
}
