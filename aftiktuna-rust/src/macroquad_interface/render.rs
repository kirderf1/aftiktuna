use std::collections::HashMap;

use super::texture::{AftikColorData, TextureStorage};
use super::App;
use crate::core::area::BackgroundType;
use crate::core::position::Direction;
use crate::core::StopType;
use crate::macroquad_interface::texture::{draw_background, BGData, TextureData};
use crate::macroquad_interface::{camera, store_render, texture, tooltip, ui};
use crate::view::area::{AftikColor, RenderData, RenderProperties};
use crate::view::{Frame, Messages};
use egui_macroquad::macroquad::camera::{set_camera, set_default_camera, Camera2D};
use egui_macroquad::macroquad::color::{BLACK, LIGHTGRAY};
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::window;

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

pub fn draw(
    app: &mut App,
    textures: &mut TextureStorage,
    color_map: &HashMap<AftikColor, AftikColorData>,
) {
    window::clear_background(BLACK);

    if app.show_graphical {
        draw_frame(
            &app.render_state.current_frame,
            app.render_state.camera,
            textures,
            color_map,
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

fn draw_frame(
    frame: &Frame,
    camera: Rect,
    textures: &mut TextureStorage,
    color_map: &HashMap<AftikColor, AftikColorData>,
) {
    match frame {
        Frame::LocationChoice(_) | Frame::Introduction => {
            set_default_camera();
            draw_background(
                &BackgroundType::location_choice(),
                0,
                camera::default_camera_space(),
                textures,
            );
        }
        Frame::AreaView { render_data, .. } => {
            set_camera(&Camera2D::from_display_rect(camera));
            draw_background(
                &render_data.background,
                render_data.background_offset.unwrap_or(0),
                camera,
                textures,
            );

            draw_objects(render_data, textures, color_map);

            set_default_camera();
            ui::draw_camera_arrows(
                textures.side_arrow,
                camera::has_camera_space(camera, render_data),
            );
        }
        Frame::Dialogue {
            background,
            color,
            direction,
            ..
        } => {
            draw_dialogue_frame(
                textures.lookup_background(background),
                &textures.portrait,
                *color,
                *direction,
                color_map,
            );
        }
        Frame::StoreView { view, .. } => {
            store_render::draw_store_view(textures, color_map, view);
        }
        Frame::Ending { stop_type } => {
            set_default_camera();
            let color = match stop_type {
                StopType::Win => LIGHTGRAY,
                StopType::Lose => BLACK,
            };
            window::clear_background(color);
        }
    }
}

fn draw_objects(
    render_data: &RenderData,
    textures: &mut TextureStorage,
    color_map: &HashMap<AftikColor, AftikColorData>,
) {
    for (pos, data) in camera::position_objects(&render_data.objects, textures) {
        texture::draw_object(
            textures.object_textures.lookup_texture(&data.texture_type),
            &data.properties,
            false,
            pos,
            color_map,
        );
        if data.properties.is_alive {
            if let Some(item_texture) = &data.wielded_item {
                texture::draw_object(
                    textures.object_textures.lookup_texture(item_texture),
                    &RenderProperties {
                        direction: data.properties.direction,
                        ..RenderProperties::default()
                    },
                    true,
                    pos,
                    color_map,
                );
            }
        }
    }
}

fn draw_dialogue_frame(
    background: &BGData,
    portrait: &TextureData,
    aftik_color: Option<AftikColor>,
    direction: Direction,
    color_map: &HashMap<AftikColor, AftikColorData>,
) {
    set_default_camera();
    texture::draw_background_portrait(background);
    let pos = match direction {
        Direction::Left => Vec2::new(500., 600.),
        Direction::Right => Vec2::new(300., 600.),
    };
    texture::draw_object(
        portrait,
        &RenderProperties {
            direction,
            aftik_color,
            ..RenderProperties::default()
        },
        false,
        pos,
        color_map,
    );
}
