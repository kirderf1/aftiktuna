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
    frame: Frame,
    text_box_text: Vec<String>,
    input_text: String,
    request_input_focus: bool,
    camera: Camera,
    mouse_pos: three_d::Vec2,
}

impl App {
    fn init(context: three_d::Context) -> Self {
        let gui = three_d::GUI::new(&context);

        let mut app = Self {
            gui,
            assets: Assets::load(context),
            game: game_interface::setup_new(),
            frame: Frame::Introduction,
            text_box_text: Vec::new(),
            input_text: String::new(),
            request_input_focus: false,
            camera: Camera::default(),
            mouse_pos: three_d::vec2(0., 0.),
        };
        app.try_get_next_frame();
        app
    }

    fn handle_frame(&mut self, mut frame_input: three_d::FrameInput) -> three_d::FrameOutput {
        let ui_result = ui::update_ui(self, &mut frame_input);

        if ui_result.clicked_text_box {
            self.try_get_next_frame();
        }
        if ui_result.triggered_input {
            let result = self.game.handle_input(&self.input_text);
            self.input_text.clear();

            match result {
                Ok(()) => self.try_get_next_frame(),
                Err(messages) => {
                    self.text_box_text = messages;
                    self.request_input_focus = true;
                }
            }
        }

        self.camera.handle_inputs(&mut frame_input.events);

        for event in &frame_input.events {
            if let three_d::Event::MouseMotion { position, .. } = event {
                self.mouse_pos = three_d::vec2(position.x, position.y);
            }
        }

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        render::render_frame(
            &self.frame,
            &self.camera,
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
            self.frame = frame_getter.get();
            if let Frame::AreaView { render_data, .. } = &self.frame {
                self.camera.camera_x =
                    render::coord_to_center_x(render_data.character_coord) - WINDOW_WIDTH_F / 2.;
            }
            self.text_box_text = self.frame.get_messages();
            self.request_input_focus = self.game.ready_to_take_input();
        }
    }
}

fn get_hovered_object_names<'a>(
    render_data: &'a RenderData,
    mouse_pos: three_d::Vec2,
    models: &mut asset::LazilyLoadedModels,
) -> Vec<&'a String> {
    render::position_objects(&render_data.objects, models)
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

#[derive(Default)]
struct Camera {
    camera_x: f32,
    is_dragging: bool,
}

impl Camera {
    fn get_render_camera(&self, viewport: three_d::Viewport) -> three_d::Camera {
        let mut render_camera = three_d::Camera::new_orthographic(
            viewport,
            three_d::vec3(
                self.camera_x + viewport.width as f32 * 0.5,
                viewport.height as f32 * 0.5,
                1.0,
            ),
            three_d::vec3(
                self.camera_x + viewport.width as f32 * 0.5,
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

    fn handle_inputs(&mut self, events: &mut [three_d::Event]) {
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
}

fn default_render_camera(viewport: three_d::Viewport) -> three_d::Camera {
    let mut render_camera = three_d::Camera::new_2d(viewport);
    render_camera.disable_tone_and_color_mapping();
    render_camera
}
