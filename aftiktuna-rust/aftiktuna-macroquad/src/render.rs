use super::camera::HorizontalDraggableCamera;
use super::texture::{self, background, RenderAssets};
use super::{store_render, tooltip, ui, AppWithEgui};
use crate::texture::LazilyLoadedModels;
use aftiktuna::asset::placement;
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::ModelId;
use aftiktuna::core::position::Direction;
use aftiktuna::view::area::{ObjectRenderData, RenderProperties};
use aftiktuna::view::{DialogueFrameData, Frame};
use aftiktuna::StopType;
use macroquad::color::{BLACK, LIGHTGRAY};
use macroquad::math::Vec2;
use macroquad::{camera as mq_camera, window};

pub(crate) struct State {
    pub text_log: Vec<String>,
    pub current_frame: Frame,
    pub cached_objects: Vec<(Vec2, ObjectRenderData)>,
    pub text_box_text: Vec<String>,
    pub camera: HorizontalDraggableCamera,
}

impl State {
    pub fn new() -> Self {
        Self {
            text_log: Vec::new(),
            current_frame: Frame::Introduction,
            cached_objects: Vec::default(),
            text_box_text: Vec::default(),
            camera: HorizontalDraggableCamera::default(),
        }
    }

    pub fn show_frame(
        &mut self,
        frame: Frame,
        ready_for_input: bool,
        models: &mut LazilyLoadedModels,
    ) {
        if let Frame::AreaView { render_data, .. } = &frame {
            self.camera = HorizontalDraggableCamera::centered_on_position(
                render_data.character_coord,
                render_data.area_size,
            );
            self.cached_objects = placement::position_objects(&render_data.objects, models)
                .into_iter()
                .map(|(pos, data)| (crate::to_vec2(pos), data))
                .collect();
        } else {
            self.cached_objects = Vec::default();
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

    pub fn show_input_text_lines(&mut self, text_lines: Vec<String>) {
        self.text_log.extend(text_lines.clone());
        self.set_text_box_text(text_lines);
    }
}

pub fn draw(app: &mut AppWithEgui) {
    let AppWithEgui { app, egui } = app;

    window::clear_background(BLACK);

    if app.show_graphical {
        draw_frame(
            &app.render_state.current_frame,
            &app.render_state.cached_objects,
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

fn draw_frame(
    frame: &Frame,
    cached_objects: &Vec<(Vec2, ObjectRenderData)>,
    camera: &HorizontalDraggableCamera,
    assets: &mut RenderAssets,
) {
    match frame {
        Frame::LocationChoice(_) | Frame::Introduction | Frame::Error(_) => {
            mq_camera::set_default_camera();
            background::draw_primary(
                &assets
                    .lookup_background(&BackgroundId::location_choice())
                    .primary,
                0,
                &HorizontalDraggableCamera::default(),
            );
        }
        Frame::AreaView { render_data, .. } => {
            mq_camera::set_camera(camera);
            background::draw_primary(
                &assets.lookup_background(&render_data.background).primary,
                render_data.background_offset,
                camera,
            );

            draw_objects(cached_objects, assets);

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

fn draw_objects(objects: &Vec<(Vec2, ObjectRenderData)>, assets: &mut RenderAssets) {
    for (pos, data) in objects {
        texture::draw_object(&data.model_id, &data.properties, false, *pos, assets);
        if data.properties.is_alive {
            if let Some(item_texture) = &data.wielded_item {
                texture::draw_object(
                    item_texture,
                    &RenderProperties {
                        direction: data.properties.direction,
                        ..RenderProperties::default()
                    },
                    true,
                    *pos,
                    assets,
                );
            }
        }
    }
}

fn draw_dialogue_frame(data: &DialogueFrameData, assets: &mut RenderAssets) {
    mq_camera::set_default_camera();
    background::draw_portrait(&assets.lookup_background(&data.background).portrait);
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
