use aftiktuna::command_suggestion::{self, Suggestion};
use aftiktuna::game_interface::{self, Game, GameResult};
use aftiktuna::view::area::RenderData;
use aftiktuna::view::Frame;
use asset::Assets;
use winit::dpi;
use winit::event_loop::EventLoop;
use winit::platform::windows::WindowBuilderExtWindows;
use winit::window::{Icon, WindowBuilder, WindowButtons};

mod asset;
mod render;
mod ui;

mod placement {
    use aftiktuna::{core::position::Coord, view::area::ObjectRenderData};
    use std::collections::HashMap;

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
    }

    pub fn position_objects<'a>(
        objects: &'a Vec<ObjectRenderData>,
        models: &mut crate::asset::LazilyLoadedModels,
    ) -> Vec<(three_d::Vec2, &'a ObjectRenderData)> {
        let mut positioned_objects = Vec::new();
        let mut positioner = Positioner::new();

        for data in objects {
            let pos = positioner.position_object(
                data.coord,
                models.lookup_model(&data.model_id).is_displacing(),
            );

            positioned_objects.push((pos, data));
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

pub const WINDOW_WIDTH: u16 = 800;
pub const WINDOW_HEIGHT: u16 = 600;
pub const WINDOW_WIDTH_F: f32 = WINDOW_WIDTH as f32;
pub const WINDOW_HEIGHT_F: f32 = WINDOW_HEIGHT as f32;

fn main() {
    let window = init_window();

    let mut app = App::init(window.gl());

    window.render_loop(move |frame_input| app.handle_frame(frame_input));
}

fn init_window() -> three_d::Window {
    let event_loop = EventLoop::new();
    let small_icon = Icon::from_rgba(
        include_bytes!("../../icon/icon_16x16.rgba").to_vec(),
        16,
        16,
    )
    .unwrap();
    let large_icon = Icon::from_rgba(
        include_bytes!("../../icon/icon_64x64.rgba").to_vec(),
        64,
        64,
    )
    .unwrap();
    let winit_window = WindowBuilder::new()
        .with_title("Aftiktuna")
        .with_window_icon(Some(small_icon))
        .with_taskbar_icon(Some(large_icon))
        .with_decorations(true)
        .with_inner_size(dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_resizable(false)
        .with_enabled_buttons(!WindowButtons::MAXIMIZE)
        .build(&event_loop)
        .unwrap();
    winit_window.focus_window();

    three_d::Window::from_winit_window(
        winit_window,
        event_loop,
        three_d::SurfaceSettings::default(),
        false,
    )
    .unwrap()
}

struct App {
    gui: three_d::GUI,
    assets: Assets,
    game: Game,
    state: State,
}

impl App {
    fn init(context: three_d::Context) -> Self {
        let gui = three_d::GUI::new(&context);

        let mut app = Self {
            gui,
            assets: Assets::load(context),
            game: game_interface::setup_new(),
            state: State {
                frame: Frame::Introduction,
                text_box_text: Vec::new(),
                input_text: String::new(),
                request_input_focus: false,
                camera: placement::Camera::default(),
                mouse_pos: three_d::vec2(0., 0.),
                command_tooltip: None,
            },
        };
        app.try_get_next_frame();
        app
    }

    fn handle_frame(&mut self, mut frame_input: three_d::FrameInput) -> three_d::FrameOutput {
        for event in &frame_input.events {
            if let three_d::Event::MouseMotion { position, .. } = event {
                self.state.mouse_pos = three_d::vec2(position.x, position.y);
            }
        }

        let mut ui_result = ui::update_ui(self, &mut frame_input);

        if ui_result.clicked_text_box {
            self.try_get_next_frame();
        }
        if let Some(chosen_suggestion) = ui_result.clicked_suggestion {
            match chosen_suggestion {
                Suggestion::Simple(command) => {
                    self.state.input_text = command;
                    ui_result.triggered_input = true;
                }
                Suggestion::Recursive(_, commands) => {
                    let pos = self.state.command_tooltip.as_ref().unwrap().pos;
                    self.state.command_tooltip = Some(CommandTooltip { pos, commands });
                }
            }
        }
        if ui_result.triggered_input {
            let result = self.game.handle_input(&self.state.input_text);
            self.state.input_text.clear();
            self.state.command_tooltip = None;

            match result {
                Ok(()) => self.try_get_next_frame(),
                Err(messages) => {
                    self.state.text_box_text = messages;
                    self.state.request_input_focus = true;
                }
            }
        }

        handle_command_suggestion_input(
            &mut frame_input.events,
            &mut self.state,
            &mut self.assets.models,
        );

        if let Frame::AreaView { render_data, .. } = &self.state.frame {
            self.state.camera.handle_inputs(&mut frame_input.events);
            self.state.camera.clamp(render_data.area_size);
        }

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        render::render_frame(
            &self.state.frame,
            &self.state.camera,
            &screen,
            &frame_input,
            &mut self.assets,
        );

        screen.write(|| self.gui.render()).unwrap();
        if self.game.next_result().has_frame() {
            ui::draw_frame_click_icon(&self.assets.left_mouse_icon, screen, &frame_input);
        }
        three_d::FrameOutput::default()
    }

    fn try_get_next_frame(&mut self) {
        if let GameResult::Frame(frame_getter) = self.game.next_result() {
            self.state.frame = frame_getter.get();
            if let Frame::AreaView { render_data, .. } = &self.state.frame {
                self.state.camera.camera_x =
                    placement::coord_to_center_x(render_data.character_coord) - WINDOW_WIDTH_F / 2.;
                self.state.camera.clamp(render_data.area_size);
            }
            self.state.text_box_text = self.state.frame.get_messages();
            self.state.request_input_focus = self.game.ready_to_take_input();
        }
    }
}

struct State {
    frame: Frame,
    text_box_text: Vec<String>,
    input_text: String,
    request_input_focus: bool,
    camera: placement::Camera,
    mouse_pos: three_d::Vec2,
    command_tooltip: Option<CommandTooltip>,
}

fn get_hovered_object_names<'a>(
    render_data: &'a RenderData,
    mouse_pos: three_d::Vec2,
    models: &mut asset::LazilyLoadedModels,
) -> Vec<&'a String> {
    placement::position_objects(&render_data.objects, models)
        .into_iter()
        .filter(|(pos, data)| models.get_rect_for_object(data, *pos).contains(mouse_pos))
        .filter_map(|(_, data)| data.name_data.as_ref())
        .map(|name_data| &name_data.modified_name)
        .collect::<Vec<_>>()
}

struct Rect {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

impl Rect {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            left: x,
            right: x + width,
            bottom: y,
            top: y + height,
        }
    }

    fn combine(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            right: self.right.max(other.right),
            bottom: self.bottom.min(other.bottom),
            top: self.top.max(other.top),
        }
    }

    fn contains(&self, pos: three_d::Vec2) -> bool {
        self.left <= pos.x && pos.x < self.right && self.bottom <= pos.y && pos.y < self.top
    }
}

struct CommandTooltip {
    pos: three_d::Vec2,
    commands: Vec<Suggestion>,
}

fn handle_command_suggestion_input(
    events: &mut [three_d::Event],
    state: &mut State,
    models: &mut asset::LazilyLoadedModels,
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
    models: &mut asset::LazilyLoadedModels,
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
    models: &mut asset::LazilyLoadedModels,
) -> Vec<Suggestion> {
    match &state.frame {
        Frame::AreaView { render_data, .. } => {
            let mouse_pos = screen_mouse_pos + three_d::vec2(state.camera.camera_x, 0.);
            placement::position_objects(&render_data.objects, models)
                .into_iter()
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

fn default_render_camera(viewport: three_d::Viewport) -> three_d::Camera {
    let mut render_camera = three_d::Camera::new_2d(viewport);
    render_camera.disable_tone_and_color_mapping();
    render_camera
}
