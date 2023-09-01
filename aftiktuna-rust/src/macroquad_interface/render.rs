use super::texture::TextureStorage;
use super::App;
use crate::area::BackgroundType;
use crate::core::StopType;
use crate::macroquad_interface::texture::draw_background;
use crate::macroquad_interface::{camera, store_render, texture, tooltip, ui};
use crate::view::{Frame, Messages, RenderData};
use egui_macroquad::macroquad::camera::{set_camera, set_default_camera, Camera2D};
use egui_macroquad::macroquad::color::{BLACK, LIGHTGRAY};
use egui_macroquad::macroquad::math::Rect;
use egui_macroquad::macroquad::window::clear_background;

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
            camera: camera::default_camera_space(),
        }
    }

    pub fn show_frame(&mut self, frame: Frame, ready_for_input: bool) {
        if let Frame::AreaView { render_data, .. } = &frame {
            self.camera = camera::character_centered_camera(render_data);
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

        set_default_camera();
        ui::draw_text_box(
            &app.render_state.text_box_text,
            textures,
            app.game.next_result().has_frame(),
        );
        tooltip::draw(&app.render_state, &app.command_tooltip, textures);

        ui::egui_only_input(app);
    } else {
        ui::egui_full(app);
    }
}

fn draw_frame(frame: &Frame, camera: Rect, textures: &TextureStorage) {
    match frame {
        Frame::LocationChoice(_) | Frame::Introduction => {
            set_default_camera();
            draw_background(
                BackgroundType::LocationChoice,
                0,
                camera::default_camera_space(),
                textures,
            );
        }
        Frame::AreaView { render_data, .. } => {
            set_camera(&Camera2D::from_display_rect(camera));
            draw_background(
                render_data.background.unwrap_or_default(),
                render_data.background_offset.unwrap_or(0),
                camera,
                textures,
            );

            draw_objects(render_data, textures);

            set_default_camera();
            ui::draw_camera_arrows(
                textures.side_arrow,
                camera::has_camera_space(camera, render_data),
            );
        }
        Frame::StoreView { view, .. } => {
            store_render::draw_store_view(textures, view);
        }
        Frame::Ending { stop_type } => {
            set_default_camera();
            let color = match stop_type {
                StopType::Win => LIGHTGRAY,
                StopType::Lose => BLACK,
            };
            clear_background(color);
        }
    }
}

fn draw_objects(render_data: &RenderData, textures: &TextureStorage) {
    for (pos, data) in camera::position_objects(&render_data.objects, textures) {
        texture::draw_object(
            textures.lookup_texture(data.texture_type),
            data.direction,
            data.aftik_color,
            false,
            pos,
        );
        if let Some(item_texture) = data.wielded_item {
            texture::draw_object(
                textures.lookup_texture(item_texture),
                data.direction,
                None,
                true,
                pos,
            );
        }
    }
}
