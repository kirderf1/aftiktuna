use aftiktuna::game_interface::{self, Game};
use aftiktuna::serialization;
use asset::Assets;
use std::path::Path;
use three_d::egui;
use winit::dpi;
use winit::event::{Event as WinitEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::windows::WindowBuilderExtWindows;
use winit::window::{Icon, Window, WindowBuilder, WindowButtons};

mod asset;
mod game;

pub const WINDOW_WIDTH: u16 = 800;
pub const WINDOW_HEIGHT: u16 = 600;
pub const WINDOW_WIDTH_F: f32 = WINDOW_WIDTH as f32;
pub const WINDOW_HEIGHT_F: f32 = WINDOW_HEIGHT as f32;

fn main() -> ! {
    let (window, event_loop) = init_window();
    let gl = three_d::WindowedContext::from_winit_window(
        &window,
        three_d::SurfaceSettings {
            multisamples: 0,
            ..Default::default()
        },
    )
    .unwrap();

    let mut app = App::init((*gl).clone());

    let mut frame_input_generator = three_d::FrameInputGenerator::from_winit_window(&window);
    event_loop.run(move |event, _, control_flow| match event {
        WinitEvent::WindowEvent { ref event, .. } => {
            frame_input_generator.handle_winit_window_event(event);
            match event {
                WindowEvent::Resized(physical_size) => {
                    gl.resize(*physical_size);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    gl.resize(**new_inner_size);
                }
                _ => (),
            }
        }
        WinitEvent::MainEventsCleared => {
            window.request_redraw();
        }
        WinitEvent::RedrawRequested(_) => {
            let frame_input = frame_input_generator.generate(&gl);
            app.handle_frame(frame_input);

            gl.swap_buffers().unwrap();
            *control_flow = ControlFlow::Poll;
            window.request_redraw();
        }
        _ => (),
    });
}

fn init_window() -> (Window, EventLoop<()>) {
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
    let window = WindowBuilder::new()
        .with_title("Aftiktuna")
        .with_window_icon(Some(small_icon))
        .with_taskbar_icon(Some(large_icon))
        .with_decorations(true)
        .with_inner_size(dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_resizable(false)
        .with_enabled_buttons(!WindowButtons::MAXIMIZE)
        .build(&event_loop)
        .unwrap();
    window.focus_window();

    (window, event_loop)
}

struct App {
    gui: three_d::GUI,
    assets: Assets,
    state: AppState,
}

impl App {
    fn init(context: three_d::Context) -> Self {
        Self {
            gui: three_d::GUI::new(&context),
            assets: Assets::load(context),
            state: AppState::main_menu(),
        }
    }

    fn handle_frame(&mut self, frame_input: three_d::FrameInput) {
        match &mut self.state {
            AppState::Game(state) => {
                state.handle_game_frame(frame_input, &mut self.gui, &mut self.assets);
            }
            AppState::MainMenu { has_save_file } => {
                let menu_action = handle_menu_frame(*has_save_file, frame_input, &mut self.gui);
                match menu_action {
                    Some(MenuAction::NewGame) => {
                        self.state = AppState::game(game_interface::setup_new())
                    }
                    Some(MenuAction::LoadGame) => {
                        self.state = AppState::game(game_interface::load().unwrap())
                    }
                    None => {}
                }
            }
        }
    }
}

enum AppState {
    MainMenu { has_save_file: bool },
    Game(Box<game::State>),
}

impl AppState {
    fn main_menu() -> Self {
        let has_save_file = Path::new(serialization::SAVE_FILE_NAME).exists();
        Self::MainMenu { has_save_file }
    }
    fn game(game: Game) -> Self {
        Self::Game(Box::new(game::State::init(game)))
    }
}

enum MenuAction {
    NewGame,
    LoadGame,
}

fn handle_menu_frame(
    has_save_file: bool,
    mut frame_input: three_d::FrameInput,
    gui: &mut three_d::GUI,
) -> Option<MenuAction> {
    let mut menu_action = None;
    gui.update(
        &mut frame_input.events,
        frame_input.accumulated_time,
        frame_input.viewport,
        frame_input.device_pixel_ratio,
        |egui_context| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(egui_context, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.add_space(116.);
                        const TITLE_FONT: egui::FontId = egui::FontId::monospace(90.);
                        ui.label(
                            egui::RichText::new("AFTIKTUNA")
                                .font(TITLE_FONT)
                                .color(egui::Color32::WHITE),
                        );

                        ui.style_mut().spacing.button_padding = egui::vec2(46., 18.);
                        const BUTTON_FONT: egui::FontId = egui::FontId::proportional(22.);
                        const BUTTON_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(
                            (0.2 * 0.6 * 255.) as u8,
                            (0.1 * 0.6 * 255.) as u8,
                            (0.4 * 0.6 * 255.) as u8,
                            (0.6 * 255.) as u8,
                        );

                        ui.add_space(124.);
                        let pressed_new_game = ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("New Game")
                                        .font(BUTTON_FONT)
                                        .color(egui::Color32::WHITE),
                                )
                                .fill(BUTTON_COLOR),
                            )
                            .clicked();
                        if pressed_new_game {
                            menu_action = Some(MenuAction::NewGame);
                        }
                        if has_save_file {
                            ui.add_space(38.);
                            let pressed_load_game = ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Load Game")
                                            .font(BUTTON_FONT)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .fill(BUTTON_COLOR),
                                )
                                .clicked();
                            if pressed_load_game {
                                menu_action = Some(MenuAction::LoadGame);
                            }
                        }
                    });
                });
        },
    );

    let screen = frame_input.screen();
    screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));
    screen.write(|| gui.render()).unwrap();

    menu_action
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
