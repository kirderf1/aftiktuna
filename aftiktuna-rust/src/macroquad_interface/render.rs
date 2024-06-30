use super::camera::HorizontalDraggableCamera;
use super::texture::RenderAssets;
use super::AppWithEgui;
use crate::core::area::BackgroundId;
use crate::core::display::ModelId;
use crate::core::position::Direction;
use crate::game_loop::StopType;
use crate::macroquad_interface::{camera, store_render, texture, tooltip, ui};
use crate::view::area::{RenderData, RenderProperties};
use crate::view::{DialogueFrameData, Frame, Messages};
use macroquad::color::{BLACK, LIGHTGRAY};
use macroquad::math::Vec2;
use macroquad::{camera as mq_camera, window};

pub struct State {
    pub text_log: Vec<String>,
    pub current_frame: Frame,
    pub text_box_text: Vec<String>,
    pub camera: HorizontalDraggableCamera,
}

impl State {
    pub fn new() -> Self {
        Self {
            text_log: vec![],
            current_frame: Frame::Introduction,
            text_box_text: vec![],
            camera: HorizontalDraggableCamera::default(),
        }
    }

    pub fn show_frame(&mut self, frame: Frame, ready_for_input: bool) {
        if let Frame::AreaView { render_data, .. } = &frame {
            self.camera = HorizontalDraggableCamera::centered_on_position(
                render_data.character_coord,
                render_data.area_size,
            );
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

pub fn draw(app: &mut AppWithEgui) {
    let AppWithEgui { app, egui } = app;

    window::clear_background(BLACK);

    if app.show_graphical {
        draw_frame(
            &app.render_state.current_frame,
            &app.render_state.camera,
            app.assets,
        );

        mq_camera::set_default_camera();
        ui::draw_text_box(
            &app.render_state.text_box_text,
            app.assets,
            app.game.next_result().has_frame(),
        );

        ui::egui_graphic(app, egui);

        tooltip::draw(
            &app.render_state,
            &app.command_tooltip,
            &mut app.assets.models,
        );
    } else {
        ui::egui_text_view(app, egui);
    }
}

fn draw_frame(frame: &Frame, camera: &HorizontalDraggableCamera, assets: &mut RenderAssets) {
    match frame {
        Frame::LocationChoice(_) | Frame::Introduction => {
            mq_camera::set_default_camera();
            assets
                .lookup_background(&BackgroundId::location_choice())
                .texture
                .draw(0, &HorizontalDraggableCamera::default());
        }
        Frame::AreaView { render_data, .. } => {
            mq_camera::set_camera(&camera.make_mq_camera());
            assets
                .lookup_background(&render_data.background)
                .texture
                .draw(render_data.background_offset.unwrap_or(0), camera);

            draw_objects(render_data, assets);

            mq_camera::set_default_camera();
            ui::draw_camera_arrows(
                &assets.side_arrow,
                camera.has_space_to_drag(render_data.area_size),
            );
        }
        Frame::Dialogue { data, .. } => {
            draw_dialogue_frame(data, assets);
        }
        Frame::StoreView { view, .. } => {
            store_render::draw_store_view(assets, view);
        }
        Frame::Ending { stop_type } => {
            mq_camera::set_default_camera();
            let color = match stop_type {
                StopType::Win => LIGHTGRAY,
                StopType::Lose => BLACK,
            };
            window::clear_background(color);
        }
    }
}

fn draw_objects(render_data: &RenderData, assets: &mut RenderAssets) {
    let mut positioned_objects = camera::position_objects(&render_data.objects, &mut assets.models);
    positioned_objects.sort_by(|(pos1, _), (pos2, _)| pos1.y.total_cmp(&pos2.y));
    for (pos, data) in positioned_objects {
        texture::draw_object(&data.model_id, &data.properties, false, pos, assets);
        if data.properties.is_alive {
            if let Some(item_texture) = &data.wielded_item {
                texture::draw_object(
                    item_texture,
                    &RenderProperties {
                        direction: data.properties.direction,
                        ..RenderProperties::default()
                    },
                    true,
                    pos,
                    assets,
                );
            }
        }
    }
}

fn draw_dialogue_frame(data: &DialogueFrameData, assets: &mut RenderAssets) {
    mq_camera::set_default_camera();
    assets.lookup_background(&data.background).portrait.draw();
    let x = match data.direction {
        Direction::Left => super::WINDOW_WIDTH_F - 300.,
        Direction::Right => 300.,
    };
    let pos = Vec2::new(x, super::WINDOW_HEIGHT_F);
    texture::draw_object(
        &ModelId::portrait(),
        &RenderProperties {
            direction: data.direction,
            aftik_color: data.color.clone(),
            is_badly_hurt: data.is_badly_hurt,
            ..RenderProperties::default()
        },
        false,
        pos,
        assets,
    );
}
