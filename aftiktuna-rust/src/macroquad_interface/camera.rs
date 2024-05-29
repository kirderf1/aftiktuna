use crate::core::position::Coord;
use crate::view::area::{ObjectRenderData, RenderData};
use crate::view::Frame;
use egui_macroquad::macroquad::input::MouseButton;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::{input, math};
use std::collections::HashMap;

use super::texture::LazilyLoadedModels;
use super::{render, ui};

pub fn position_objects<'a>(
    objects: &'a Vec<ObjectRenderData>,
    models: &mut LazilyLoadedModels,
) -> Vec<(Vec2, &'a ObjectRenderData)> {
    let mut positioned_objects = Vec::new();
    let mut coord_counts: HashMap<Coord, i32> = HashMap::new();

    for data in objects {
        let coord = data.coord;
        let count = if models.lookup_model(&data.texture_type).is_displacing() {
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

pub fn try_drag_camera_for_state(state: &mut render::State, last_drag_pos: &mut Option<Vec2>) {
    match &state.current_frame {
        Frame::AreaView { render_data, .. } => {
            try_drag_camera(
                last_drag_pos,
                &mut state.camera,
                render_data.area_size,
                !ui::is_mouse_at_text_box(&state.text_box_text),
            );
        }
        _ => {
            *last_drag_pos = None;
        }
    }
}

pub fn try_drag_camera(
    last_drag_pos: &mut Option<Vec2>,
    camera: &mut Rect,
    area_size: Coord,
    can_start_dragging: bool,
) {
    if input::is_mouse_button_pressed(MouseButton::Left)
        && can_start_dragging
        && last_drag_pos.is_none()
    {
        *last_drag_pos = Some(input::mouse_position().into());
    }

    if let Some(last_pos) = *last_drag_pos {
        if input::is_mouse_button_down(MouseButton::Left) {
            let mouse_pos: Vec2 = input::mouse_position().into();
            let camera_delta = mouse_pos - last_pos;

            camera.x -= camera_delta.x;
            clamp_camera(camera, area_size);
            *last_drag_pos = Some(mouse_pos);
        } else {
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

fn clamp_camera(camera: &mut Rect, area_size: Coord) {
    camera.x = if area_size <= 6 {
        (coord_to_center_x(0) + coord_to_center_x(area_size - 1)) / 2. - camera.w / 2.
    } else {
        math::clamp(
            camera.x,
            coord_to_center_x(0) - 100.,
            coord_to_center_x(area_size - 1) + 100. - camera.w,
        )
    };
}

pub fn default_camera_space() -> Rect {
    Rect::new(0., 0., 800., 600.)
}

pub fn position_centered_camera(position: Coord, area_size: Coord) -> Rect {
    let mut camera_space = Rect::new(coord_to_center_x(position) - 400., 0., 800., 600.);
    clamp_camera(&mut camera_space, area_size);
    camera_space
}
