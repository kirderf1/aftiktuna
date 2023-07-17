use super::texture::{draw_object, BGTexture, BGTextureType, TextureStorage};
use super::App;
use crate::macroquad_interface::texture::get_rect_for_object;
use crate::macroquad_interface::ui;
use crate::position::Coord;
use crate::view::{Frame, Messages, ObjectRenderData, RenderData};
use macroquad::camera::set_camera;
use macroquad::color::{BLACK, WHITE};
use macroquad::prelude::{
    clamp, clear_background, draw_texture, is_mouse_button_down, is_mouse_button_pressed,
    mouse_position, set_default_camera, Camera2D, MouseButton, Rect, Vec2,
};
use std::collections::HashMap;

pub struct State {
    pub text_log: Vec<String>,
    view_state: ViewState,
    pub(crate) text_box_text: Vec<String>,
    camera: Rect,
    last_drag_pos: Option<Vec2>,
}

impl State {
    pub fn new() -> Self {
        Self {
            text_log: vec![],
            view_state: ViewState::LocationChoice,
            text_box_text: vec![],
            camera: default_camera_space(),
            last_drag_pos: None,
        }
    }

    pub fn show_frame(&mut self, frame: Frame, ready_for_input: bool) {
        self.text_log.extend(frame.as_text());
        if ready_for_input {
            self.text_log.push(String::default())
        }

        match frame {
            Frame::AreaView {
                render_data,
                messages,
                ..
            } => {
                self.camera = character_centered_camera(&render_data);
                self.view_state = ViewState::InGame(render_data);
                self.set_text_box_text(messages.into_text());
            }
            Frame::LocationChoice(messages) => {
                self.camera = default_camera_space();
                self.view_state = ViewState::LocationChoice;
                self.set_text_box_text(messages.into_text());
            }
            Frame::Ending(stop_type) => {
                self.set_text_box_text(stop_type.messages().into_text());
            }
        }
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

pub enum ViewState {
    LocationChoice,
    InGame(RenderData),
}

pub fn draw(app: &mut App, textures: &TextureStorage) {
    try_drag_camera(&mut app.render_state);

    clear_background(BLACK);

    if app.show_graphical {
        draw_game(&app.render_state, textures, !app.delayed_frames.is_done());
    }

    egui_macroquad::ui(|ctx| ui::egui_ui(app, ctx));

    egui_macroquad::draw();
}

fn draw_game(state: &State, textures: &TextureStorage, click_to_proceed: bool) {
    match &state.view_state {
        ViewState::LocationChoice => {
            draw_background(
                BGTextureType::LocationChoice,
                default_camera_space(),
                textures,
            );
        }
        ViewState::InGame(render_data) => {
            set_camera(&Camera2D::from_display_rect(state.camera));
            draw_background(
                render_data
                    .background
                    .map_or(BGTextureType::Blank, BGTextureType::from),
                state.camera,
                textures,
            );

            draw_objects(render_data, textures);
            find_and_draw_tooltip(
                render_data,
                textures,
                Vec2::new(state.camera.x, state.camera.y),
            );
            set_default_camera();
        }
    }

    ui::draw_text_box(&state.text_box_text, textures, click_to_proceed);
}

fn draw_background(texture_type: BGTextureType, camera_space: Rect, textures: &TextureStorage) {
    match textures.lookup_background(texture_type) {
        BGTexture::Simple(texture) => draw_texture(*texture, camera_space.x, 0., WHITE),
        BGTexture::Repeating(texture) => {
            let start_x = texture.width() * f32::floor(camera_space.x / texture.width());
            draw_texture(*texture, start_x, 0., WHITE);
            draw_texture(*texture, start_x + texture.width(), 0., WHITE);
        }
    }
}

fn draw_objects(render_data: &RenderData, textures: &TextureStorage) {
    for (pos, data) in position_objects(&render_data.objects) {
        draw_object(data, textures, pos);
    }
}

fn find_and_draw_tooltip(render_data: &RenderData, textures: &TextureStorage, camera_offset: Vec2) {
    let mouse_pos = Vec2::from(mouse_position()) + camera_offset;
    let hovered_objects = position_objects(&render_data.objects)
        .into_iter()
        .filter(|(pos, data)| get_rect_for_object(data, textures, *pos).contains(mouse_pos))
        .map(|(_, data)| &data.name)
        .collect::<Vec<_>>();

    if hovered_objects.is_empty() {
        return;
    }

    ui::draw_tooltip(mouse_pos, hovered_objects);
}

fn position_objects(objects: &Vec<ObjectRenderData>) -> Vec<(Vec2, &ObjectRenderData)> {
    let mut positioned_objects = Vec::new();
    let mut coord_counts: HashMap<Coord, i32> = HashMap::new();

    for data in objects {
        let coord = data.coord;
        let count_ref = coord_counts.entry(coord).or_insert(0);
        let count = *count_ref;
        *count_ref = count + 1;

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

fn try_drag_camera(state: &mut State) {
    match (&state.view_state, state.last_drag_pos) {
        (ViewState::InGame(render_data), Some(last_pos)) => {
            if is_mouse_button_down(MouseButton::Left) {
                let mouse_pos: Vec2 = mouse_position().into();
                let camera_delta = mouse_pos - last_pos;

                state.camera.x -= camera_delta.x;
                clamp_camera(&mut state.camera, render_data);
                state.last_drag_pos = Some(mouse_pos);
            } else {
                state.last_drag_pos = None;
            }
        }
        (ViewState::InGame(_), None) => {
            if is_mouse_button_pressed(MouseButton::Left) && !ui::is_mouse_at_text_box(state) {
                state.last_drag_pos = Some(mouse_position().into());
            }
        }
        _ => {
            state.last_drag_pos = None;
        }
    }
}

fn clamp_camera(camera: &mut Rect, render_data: &RenderData) {
    camera.x = if render_data.size <= 6 {
        (coord_to_center_x(0) + coord_to_center_x(render_data.size - 1)) / 2. - camera.w / 2.
    } else {
        clamp(
            camera.x,
            coord_to_center_x(0) - 100.,
            coord_to_center_x(render_data.size - 1) + 100. - camera.w,
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
