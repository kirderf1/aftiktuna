use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::placement;
use aftiktuna::core::position::Coord;
use aftiktuna::view::area::ObjectRenderData;
use aftiktuna::view::Frame;
use macroquad::camera as mq_camera;
use macroquad::input::MouseButton;
use macroquad::math::{Mat4, Vec2};
use macroquad::{input, math};

use super::texture::LazilyLoadedModels;
use super::{render, ui};

#[derive(Debug, Default)]
pub struct HorizontalDraggableCamera {
    pub(super) x_start: f32,
    last_drag_pos: Option<Vec2>,
    viewport: Option<(i32, i32, i32, i32)>,
}

impl HorizontalDraggableCamera {
    pub fn centered_on_position(position: Coord, area_size: Coord) -> Self {
        let mut camera = Self {
            x_start: placement::coord_to_center_x(position) - super::WINDOW_WIDTH_F / 2.,
            last_drag_pos: None,
            viewport: None,
        };
        camera.clamp(area_size);
        camera
    }

    pub fn set_default_size_viewport(&mut self, x: i32, y: i32) {
        self.viewport = Some((
            x,
            y,
            super::WINDOW_WIDTH.into(),
            super::WINDOW_HEIGHT.into(),
        ));
    }

    pub fn get_offset(&self) -> Vec2 {
        Vec2::new(self.x_start, 0.)
    }

    pub fn clamp(&mut self, area_size: Coord) {
        self.x_start = if area_size <= 6 {
            (placement::coord_to_center_x(0) + placement::coord_to_center_x(area_size - 1)) / 2.
                - super::WINDOW_WIDTH_F / 2.
        } else {
            math::clamp(
                self.x_start,
                placement::coord_to_center_x(0) - 100.,
                placement::coord_to_center_x(area_size - 1) + 100. - super::WINDOW_WIDTH_F,
            )
        };
    }

    pub fn is_dragging(&self) -> bool {
        self.last_drag_pos.is_some()
    }

    pub fn stop_dragging(&mut self) {
        self.last_drag_pos = None;
    }

    pub fn handle_drag(&mut self, area_size: Coord, can_start_dragging: bool) {
        if input::is_mouse_button_pressed(MouseButton::Left)
            && can_start_dragging
            && self.last_drag_pos.is_none()
        {
            self.last_drag_pos = Some(input::mouse_position().into());
        }

        if let Some(last_pos) = self.last_drag_pos {
            if input::is_mouse_button_down(MouseButton::Left) {
                let mouse_pos: Vec2 = input::mouse_position().into();
                let drag_delta = mouse_pos - last_pos;

                self.x_start -= drag_delta.x;
                self.clamp(area_size);
                self.last_drag_pos = Some(mouse_pos);
            } else {
                self.last_drag_pos = None;
            }
        }
    }

    pub fn has_space_to_drag(&self, area_size: Coord) -> [bool; 2] {
        if area_size <= 6 {
            [false, false]
        } else {
            [
                self.x_start > placement::coord_to_center_x(0) - 100.,
                self.x_start + super::WINDOW_WIDTH_F
                    < placement::coord_to_center_x(area_size - 1) + 100.,
            ]
        }
    }
}

impl mq_camera::Camera for HorizontalDraggableCamera {
    fn matrix(&self) -> Mat4 {
        Mat4::orthographic_rh_gl(
            self.x_start,
            self.x_start + super::WINDOW_WIDTH_F,
            super::WINDOW_HEIGHT_F,
            0.,
            1.,
            -1.,
        )
    }

    fn depth_enabled(&self) -> bool {
        false
    }

    fn render_pass(&self) -> Option<macroquad::miniquad::RenderPass> {
        None
    }

    fn viewport(&self) -> Option<(i32, i32, i32, i32)> {
        self.viewport
    }
}

pub fn position_objects(
    objects: &Vec<ObjectRenderData>,
    models: &mut LazilyLoadedModels,
) -> Vec<(Vec2, ObjectRenderData)> {
    let mut positioned_objects = Vec::new();
    let mut positioner = placement::Positioner::default();

    for data in objects {
        let (pos, _) = positioner.position_object(
            data.coord,
            models.lookup_model(&data.model_id).is_displacing(),
        );

        positioned_objects.push((to_vec2(pos), data.clone()));
    }
    positioned_objects.sort_by(|(pos1, _), (pos2, _)| pos1.y.total_cmp(&pos2.y));
    positioned_objects
}

pub fn to_vec2((x, y): (f32, f32)) -> Vec2 {
    Vec2::new(x, crate::WINDOW_HEIGHT_F - y)
}

pub(crate) fn try_drag_camera_for_state(state: &mut render::State) {
    match &state.current_frame {
        Frame::AreaView { render_data, .. } => {
            state.camera.handle_drag(
                render_data.area_size,
                !ui::is_mouse_at_text_box(&state.text_box_text),
            );
        }
        _ => {
            state.camera.stop_dragging();
        }
    }
}
