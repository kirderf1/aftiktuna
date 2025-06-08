mod frame_render;
mod ui;

use crate::Assets;
use aftiktuna::asset::placement;
use aftiktuna::command_suggestion::{self, Suggestion};
use aftiktuna::game_interface::{Game, GameResult};
use aftiktuna::serialization;
use aftiktuna::view::area::ObjectRenderData;
use aftiktuna::view::{Frame, FullStatus};
use aftiktuna_three_d::asset::LazilyLoadedModels;
use aftiktuna_three_d::Camera;
use std::fs;

pub enum GameAction {
    ExitGame,
}

pub struct State {
    is_save_enabled: bool,
    game: Game,
    frame: Frame,
    cached_objects: Vec<(three_d::Vec2, ObjectRenderData)>,
    text_box_text: Vec<String>,
    displayed_status: Option<FullStatus>,
    input_text: String,
    request_input_focus: bool,
    camera: Camera,
    mouse_pos: three_d::Vec2,
    command_tooltip: Option<CommandTooltip>,
}

impl State {
    pub fn init(game: Game, is_save_enabled: bool, assets: &mut Assets) -> Self {
        let mut state = Self {
            is_save_enabled,
            game,
            frame: Frame::Introduction,
            cached_objects: Vec::new(),
            text_box_text: Vec::new(),
            displayed_status: None,
            input_text: String::new(),
            request_input_focus: false,
            camera: Camera::default(),
            mouse_pos: three_d::vec2(0., 0.),
            command_tooltip: None,
        };
        state.try_get_next_frame(&mut assets.models);
        state
    }

    pub fn handle_game_frame(
        &mut self,
        mut frame_input: three_d::FrameInput,
        gui: &mut three_d::GUI,
        assets: &mut Assets,
    ) -> Option<GameAction> {
        let mut action = None;

        for event in &frame_input.events {
            if let three_d::Event::MouseMotion { position, .. } = event {
                self.mouse_pos = three_d::vec2(position.x, position.y);
            }
        }

        let mut ui_result = ui::update_ui(gui, &mut frame_input, self, assets);

        if ui_result.closed_status_window {
            self.displayed_status = None;
            self.request_input_focus = true;
        }

        let pressed_enter = crate::check_pressed_enter(&mut frame_input.events);

        if matches!(self.game.next_result(), GameResult::Stop) {
            let clicked = ui_result.clicked_text_box
                || crate::check_clicked_anywhere(&mut frame_input.events);
            if clicked || pressed_enter {
                self.save_game_if_enabled();
                action = Some(GameAction::ExitGame);
            }
        }

        if ui_result.clicked_text_box || pressed_enter {
            self.try_get_next_frame(&mut assets.models);
        }
        if let Some(chosen_suggestion) = ui_result.clicked_suggestion {
            match chosen_suggestion {
                Suggestion::Simple(command) => {
                    self.input_text = command;
                    ui_result.triggered_input = true;
                }
                Suggestion::Recursive(_, commands) => {
                    let pos = self.command_tooltip.as_ref().unwrap().pos;
                    self.command_tooltip = Some(CommandTooltip { pos, commands });
                }
            }
        }
        if ui_result.triggered_input {
            if self.input_text.eq_ignore_ascii_case("exit game") {
                self.save_game_if_enabled();
                action = Some(GameAction::ExitGame);
            } else {
                let result = self.game.handle_input(&self.input_text);

                match result {
                    Ok(()) => self.try_get_next_frame(&mut assets.models),
                    Err(command_info) => match command_info {
                        aftiktuna::CommandInfo::Message(items) => {
                            self.text_box_text = items;
                            self.request_input_focus = true;
                        }
                        aftiktuna::CommandInfo::Status(full_status) => {
                            self.text_box_text = vec![];
                            self.displayed_status = Some(full_status);
                        }
                    },
                }
            }

            self.input_text.clear();
            self.command_tooltip = None;
        }

        if self.game.ready_to_take_input() && self.displayed_status.is_none() {
            handle_command_suggestion_input(&mut frame_input.events, self, &mut assets.models);
        } else {
            self.command_tooltip = None;
        }

        if self.displayed_status.is_none() {
            if let Frame::AreaView { render_data, .. } = &self.frame {
                self.camera.handle_inputs(&mut frame_input.events);
                self.camera.clamp(render_data.area_size);
            }
        }

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        frame_render::render_frame(
            &self.frame,
            &self.cached_objects,
            &self.camera,
            &screen,
            &frame_input,
            assets,
        );

        screen.write(|| gui.render()).unwrap();
        if self.game.next_result().has_frame() {
            ui::draw_frame_click_icon(&assets.left_mouse_icon, screen, &frame_input);
        }

        action
    }

    pub fn save_game_if_enabled(&self) {
        if !matches!(self.frame, Frame::Ending { .. }) && self.is_save_enabled {
            if let Err(error) = serialization::write_game_to_save_file(&self.game) {
                eprintln!("Failed to save game: {error}");
            } else {
                println!("Saved the game successfully.")
            }
        }
    }

    fn try_get_next_frame(&mut self, models: &mut LazilyLoadedModels) {
        if let GameResult::Frame(frame_getter) = self.game.next_result() {
            self.frame = frame_getter.get();
            if let Frame::AreaView { render_data, .. } = &self.frame {
                self.camera.set_center(render_data.character_coord);
                self.camera.clamp(render_data.area_size);
                self.cached_objects = placement::position_objects(&render_data.objects, models)
                    .into_iter()
                    .map(|(pos, data)| (pos.into(), data))
                    .collect();
            } else {
                self.cached_objects = Vec::new();
            }
            if matches!(self.frame, Frame::Ending { .. }) && self.is_save_enabled {
                let _ = fs::remove_file(serialization::SAVE_FILE_NAME);
            }
            self.text_box_text = self.frame.get_messages();
            self.request_input_focus = self.game.ready_to_take_input();
        }
    }
}

fn get_hovered_object_names<'a>(
    objects: &'a [(three_d::Vec2, ObjectRenderData)],
    mouse_pos: three_d::Vec2,
    models: &mut LazilyLoadedModels,
) -> Vec<&'a String> {
    objects
        .iter()
        .filter(|(pos, data)| models.get_rect_for_object(data, *pos).contains(mouse_pos))
        .filter_map(|(_, data)| data.name_data.as_ref())
        .map(|name_data| &name_data.modified_name)
        .collect::<Vec<_>>()
}

struct CommandTooltip {
    pos: three_d::Vec2,
    commands: Vec<Suggestion>,
}

fn handle_command_suggestion_input(
    events: &mut [three_d::Event],
    state: &mut State,
    models: &mut LazilyLoadedModels,
) {
    for event in events {
        if let three_d::Event::MousePress {
            button,
            position,
            handled,
            ..
        } = event
        {
            if !*handled && *button == three_d::MouseButton::Left {
                *handled = handle_command_suggestion_click(
                    three_d::vec2(position.x, position.y),
                    state,
                    models,
                )
            }
        }
    }
}

fn handle_command_suggestion_click(
    screen_mouse_pos: three_d::Vec2,
    state: &mut State,
    models: &mut LazilyLoadedModels,
) -> bool {
    if state.command_tooltip.is_some() {
        state.command_tooltip = None;
        false
    } else {
        let commands = find_command_suggestions(screen_mouse_pos, state, models);
        if commands.is_empty() {
            false
        } else {
            state.command_tooltip = Some(CommandTooltip {
                pos: screen_mouse_pos + three_d::vec2(state.camera.camera_x, 0.),
                commands: command_suggestion::sorted_without_duplicates(commands),
            });
            true
        }
    }
}

fn find_command_suggestions(
    screen_mouse_pos: three_d::Vec2,
    state: &State,
    models: &mut LazilyLoadedModels,
) -> Vec<Suggestion> {
    match &state.frame {
        Frame::AreaView { render_data, .. } => {
            let mouse_pos = screen_mouse_pos + three_d::vec2(state.camera.camera_x, 0.);
            state
                .cached_objects
                .iter()
                .filter(|(pos, data)| models.get_rect_for_object(data, *pos).contains(mouse_pos))
                .filter_map(|(_, data)| data.name_data.as_ref().zip(Some(&data.interactions)))
                .flat_map(|(name_data, interactions)| {
                    interactions.iter().flat_map(|interaction| {
                        interaction.commands(&name_data.name, &render_data.inventory)
                    })
                })
                .collect::<Vec<_>>()
        }
        Frame::StoreView { view, .. } => command_suggestion::for_store(
            frame_render::find_stock_at(screen_mouse_pos, view),
            &view.sellable_items,
        ),
        Frame::LocationChoice(choice) => command_suggestion::for_location_choice(choice),
        _ => vec![],
    }
}

fn get_render_camera(camera: &Camera, viewport: three_d::Viewport) -> three_d::Camera {
    let mut render_camera = three_d::Camera::new_orthographic(
        viewport,
        three_d::vec3(
            camera.camera_x + viewport.width as f32 * 0.5,
            viewport.height as f32 * 0.5,
            1.0,
        ),
        three_d::vec3(
            camera.camera_x + viewport.width as f32 * 0.5,
            viewport.height as f32 * 0.5,
            0.0,
        ),
        three_d::vec3(0.0, 1.0, 0.0),
        viewport.height as f32,
        0.0,
        10.0,
    );
    render_camera.disable_tone_and_color_mapping();
    render_camera
}
