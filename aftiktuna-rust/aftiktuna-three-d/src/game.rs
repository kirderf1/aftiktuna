use crate::asset::{Assets, LazilyLoadedModels};
use aftiktuna::command_suggestion::{self, Suggestion};
use aftiktuna::game_interface::{Game, GameResult};
use aftiktuna::serialization;
use aftiktuna::view::area::ObjectRenderData;
use aftiktuna::view::{Frame, FullStatus};
use std::fs;

mod render;
mod ui;

mod placement {
    use aftiktuna::core::display::OrderWeight;
    use aftiktuna::core::position::Coord;
    use aftiktuna::view::area::ObjectRenderData;
    use std::collections::HashMap;
    use std::mem;

    // Coordinates are mapped like this so that when the left edge of the window is 0,
    // coord 3 will be placed in the middle of the window.
    pub fn coord_to_center_x(coord: Coord) -> f32 {
        40. + 120. * coord as f32
    }

    #[derive(Default)]
    pub struct Camera {
        pub camera_x: f32,
        pub is_dragging: bool,
    }

    impl Camera {
        pub fn handle_inputs(&mut self, events: &mut [three_d::Event]) {
            for event in events {
                match event {
                    three_d::Event::MousePress {
                        button, handled, ..
                    } => {
                        if !*handled && *button == three_d::MouseButton::Left {
                            self.is_dragging = true;
                            *handled = true;
                        }
                    }
                    three_d::Event::MouseRelease {
                        button, handled, ..
                    } => {
                        if self.is_dragging && *button == three_d::MouseButton::Left {
                            self.is_dragging = false;
                            *handled = true;
                        }
                    }
                    three_d::Event::MouseMotion { delta, handled, .. } => {
                        if !*handled && self.is_dragging {
                            self.camera_x -= delta.0;
                            *handled = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        pub fn clamp(&mut self, area_size: Coord) {
            self.camera_x = if area_size <= 6 {
                (coord_to_center_x(0) + coord_to_center_x(area_size - 1)) / 2.
                    - crate::WINDOW_WIDTH_F / 2.
            } else {
                self.camera_x.clamp(
                    coord_to_center_x(0) - 100.,
                    coord_to_center_x(area_size - 1) + 100. - crate::WINDOW_WIDTH_F,
                )
            };
        }

        pub fn has_space_to_drag(&self, area_size: Coord) -> [bool; 2] {
            if area_size <= 6 {
                [false, false]
            } else {
                [
                    self.camera_x > coord_to_center_x(0) - 100.,
                    self.camera_x + crate::WINDOW_WIDTH_F < coord_to_center_x(area_size - 1) + 100.,
                ]
            }
        }
    }

    pub fn position_objects(
        objects: &Vec<ObjectRenderData>,
        models: &mut crate::asset::LazilyLoadedModels,
    ) -> Vec<(three_d::Vec2, ObjectRenderData)> {
        let mut positioned_objects = Vec::new();
        let mut positioner = Positioner::new();
        let mut groups_cache: Vec<Vec<ObjectRenderData>> =
            vec![Vec::new(); objects.iter().map(|obj| obj.coord).max().unwrap_or(0) + 1];
        let mut last_weight = OrderWeight::Background;

        for data in objects {
            if data.weight != last_weight {
                for object_group in &mut groups_cache {
                    positioned_objects
                        .extend(positioner.position_group(mem::take(object_group), models));
                }
                last_weight = data.weight;
            }

            let object_group = &mut groups_cache[data.coord];
            if object_group
                .first()
                .is_some_and(|cached_object| cached_object.model_id != data.model_id)
            {
                positioned_objects
                    .extend(positioner.position_group(mem::take(object_group), models));
            }

            object_group.push(data.clone());
        }

        for object_group in groups_cache {
            positioned_objects.extend(positioner.position_group(object_group, models));
        }

        positioned_objects
    }

    fn position_from_coord(coord: Coord, count: i32) -> three_d::Vec2 {
        three_d::vec2(
            coord_to_center_x(coord) - count as f32 * 15.,
            (190 - count * 15) as f32,
        )
    }

    #[derive(Default)]
    struct Positioner {
        coord_counts: HashMap<Coord, i32>,
    }

    impl Positioner {
        pub fn new() -> Self {
            Self::default()
        }

        fn position_group(
            &mut self,
            object_group: Vec<ObjectRenderData>,
            models: &mut crate::asset::LazilyLoadedModels,
        ) -> Vec<(three_d::Vec2, ObjectRenderData)> {
            if let Some((coord, model)) = object_group
                .first()
                .map(|object| (object.coord, models.lookup_model(&object.model_id)))
            {
                model
                    .group_placement
                    .position(object_group.len() as u16)
                    .into_iter()
                    .flat_map(|offsets| {
                        let base_pos = self.position_object(coord, model.is_displacing());
                        offsets.into_iter().map(move |offset| {
                            base_pos + three_d::vec2(offset.0.into(), offset.1.into())
                        })
                    })
                    .zip(object_group)
                    .collect()
            } else {
                Vec::default()
            }
        }

        fn position_object(&mut self, coord: Coord, is_displacing: bool) -> three_d::Vec2 {
            if is_displacing {
                let count_ref = self.coord_counts.entry(coord).or_insert(0);
                let count = *count_ref;
                *count_ref = count + 1;
                position_from_coord(coord, count)
            } else {
                position_from_coord(coord, 0)
            }
        }
    }
}

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
    camera: placement::Camera,
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
            camera: placement::Camera::default(),
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

        render::render_frame(
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
                self.camera.camera_x = placement::coord_to_center_x(render_data.character_coord)
                    - crate::WINDOW_WIDTH_F / 2.;
                self.camera.clamp(render_data.area_size);
                self.cached_objects = placement::position_objects(&render_data.objects, models);
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
            render::find_stock_at(screen_mouse_pos, view),
            &view.sellable_items,
        ),
        Frame::LocationChoice(choice) => command_suggestion::for_location_choice(choice),
        _ => vec![],
    }
}

fn get_render_camera(camera: &placement::Camera, viewport: three_d::Viewport) -> three_d::Camera {
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
