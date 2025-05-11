use aftiktuna::game_interface::{self, Game};
use aftiktuna::serialization::{self, LoadError};
use asset::Assets;
use std::env;
use std::path::Path;
use std::rc::Rc;
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
    let gl =
        three_d::WindowedContext::from_winit_window(&window, three_d::SurfaceSettings::default())
            .unwrap();

    let mut app = App {
        loaded_app: None,
        builtin_fonts: Rc::new(BuiltinFonts::init()),
        error_messages: vec![],
    };

    let mut frame_input_generator = three_d::FrameInputGenerator::from_winit_window(&window);
    event_loop.run(move |event, _, control_flow| match event {
        WinitEvent::WindowEvent { ref event, .. } => {
            frame_input_generator.handle_winit_window_event(event);
            match event {
                WindowEvent::Resized(physical_size) => {
                    gl.resize(*physical_size);
                }
                WindowEvent::CloseRequested => {
                    if let Some(loaded_app) = &mut app.loaded_app {
                        loaded_app.on_exit();
                    }
                    *control_flow = ControlFlow::Exit;
                }
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
            let action = app.handle_frame(frame_input);

            match action {
                AppAction::Continue => {
                    gl.swap_buffers().unwrap();
                    *control_flow = ControlFlow::Poll;
                    window.request_redraw();

                    if app.loaded_app.is_none() && app.error_messages.is_empty() {
                        match LoadedApp::load(&gl, app.builtin_fonts.clone()) {
                            Ok(loaded_app) => app.loaded_app = Some(loaded_app),
                            Err(error) => {
                                app.error_messages = split_screen_text_lines(
                                    &app.builtin_fonts.text_gen_size_16,
                                    vec![format!("Unable to load assets:"), format!("{error}")],
                                );
                            }
                        }
                    }
                }
                AppAction::Exit => *control_flow = ControlFlow::Exit,
            }
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

enum AppAction {
    Continue,
    Exit,
}

type ErrorMessages = Vec<String>;

struct App {
    loaded_app: Option<LoadedApp>,
    builtin_fonts: Rc<BuiltinFonts>,
    error_messages: ErrorMessages,
}

impl App {
    fn handle_frame(&mut self, mut frame_input: three_d::FrameInput) -> AppAction {
        if !self.error_messages.is_empty() {
            let clicked = check_clicked_anywhere(&mut frame_input.events);
            let pressed_enter = check_pressed_enter(&mut frame_input.events);

            let screen = frame_input.screen();
            screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));
            let mut y = 350.;
            for line in &self.error_messages {
                let text_obj = make_centered_text_obj(
                    line,
                    three_d::vec2(WINDOW_WIDTH_F / 2., y),
                    three_d::vec4(1., 0.4, 0.7, 1.),
                    &self.builtin_fonts.text_gen_size_16,
                    &frame_input.context,
                );
                screen.render(
                    default_render_camera(frame_input.viewport),
                    &[text_obj],
                    &[],
                );
                y -= 24.
            }

            if clicked || pressed_enter {
                self.error_messages.clear();
            }
            if (clicked || pressed_enter) && self.loaded_app.is_none() {
                AppAction::Exit
            } else {
                AppAction::Continue
            }
        } else if let Some(loaded_app) = &mut self.loaded_app {
            let (app_action, error_messages) = loaded_app.handle_frame(frame_input);
            self.error_messages =
                split_screen_text_lines(&self.builtin_fonts.text_gen_size_16, error_messages);
            app_action
        } else {
            let screen = frame_input.screen();
            screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));
            let text_obj = make_centered_text_obj(
                "Loading textures...",
                three_d::vec2(WINDOW_WIDTH_F / 2., 300.),
                three_d::vec4(1., 1., 1., 1.),
                &self.builtin_fonts.text_gen_size_20,
                &frame_input.context,
            );
            screen.render(
                default_render_camera(frame_input.viewport),
                &[text_obj],
                &[],
            );
            AppAction::Continue
        }
    }
}

struct BuiltinFonts {
    pub text_gen_size_16: three_d::TextGenerator<'static>,
    pub text_gen_size_20: three_d::TextGenerator<'static>,
}

impl BuiltinFonts {
    fn init() -> Self {
        Self {
            text_gen_size_16: three_d::TextGenerator::new(
                epaint_default_fonts::HACK_REGULAR,
                0,
                16.,
            )
            .expect("Unexpected error for builtin font"),
            text_gen_size_20: three_d::TextGenerator::new(
                epaint_default_fonts::HACK_REGULAR,
                0,
                20.,
            )
            .expect("Unexpected error for builtin font"),
        }
    }
}

fn make_centered_text_obj(
    text: &str,
    pos: three_d::Vec2,
    color: three_d::Vec4,
    text_gen: &three_d::TextGenerator<'static>,
    context: &three_d::Context,
) -> impl three_d::Object {
    let mut mesh = text_gen.generate(text, three_d::TextLayoutOptions::default());
    mesh.transform(three_d::Matrix4::from_translation(three_d::vec3(
        pos.x - (mesh.compute_aabb().size().x) / 2.,
        pos.y,
        0.,
    )))
    .unwrap();
    three_d::Gm::new(three_d::Mesh::new(context, &mesh), color_material(color))
}

fn make_text_obj(
    text: &str,
    pos: three_d::Vec2,
    color: three_d::Vec4,
    text_gen: &three_d::TextGenerator<'static>,
    context: &three_d::Context,
) -> impl three_d::Object {
    let mut mesh = text_gen.generate(text, three_d::TextLayoutOptions::default());
    mesh.transform(three_d::Matrix4::from_translation(three_d::vec3(
        pos.x, pos.y, 0.,
    )))
    .unwrap();
    three_d::Gm::new(three_d::Mesh::new(context, &mesh), color_material(color))
}

fn split_screen_text_lines(
    text_gen: &three_d::TextGenerator<'static>,
    lines: Vec<String>,
) -> Vec<String> {
    lines
        .into_iter()
        .flat_map(|line| {
            if text_fits_on_screen(text_gen, &line) {
                return vec![line];
            }

            let mut remaining_line: &str = &line;
            let mut vec = Vec::new();
            loop {
                let split_index = smallest_screen_text_split(text_gen, remaining_line);
                vec.push(remaining_line[..split_index].to_owned());
                remaining_line = &remaining_line[split_index..];

                if text_fits_on_screen(text_gen, remaining_line) {
                    vec.push(remaining_line.to_owned());
                    return vec;
                }
            }
        })
        .collect()
}

fn text_fits_on_screen(text_gen: &three_d::TextGenerator<'static>, line: &str) -> bool {
    text_gen
        .generate(line, three_d::TextLayoutOptions::default())
        .compute_aabb()
        .size()
        .x
        <= 700.
}

fn smallest_screen_text_split(text_gen: &three_d::TextGenerator<'static>, line: &str) -> usize {
    let mut last_space = 0;
    let mut last_index = 0;
    for (index, char) in line.char_indices() {
        if !text_fits_on_screen(text_gen, &line[..index]) {
            return if last_space != 0 {
                last_space
            } else {
                last_index
            };
        }

        if char.is_whitespace() {
            last_space = index;
        }
        last_index = index;
    }
    line.len()
}

struct LoadedApp {
    gui: three_d::GUI,
    assets: Assets,
    state: AppState,
    autosave: bool,
    close_after_ending: bool,
}

impl LoadedApp {
    fn load(
        context: &three_d::Context,
        builtin_fonts: Rc<BuiltinFonts>,
    ) -> Result<Self, asset::Error> {
        let assets = Assets::load(context.clone(), builtin_fonts)?;

        let disable_autosave = env::args().any(|arg| arg.eq("--disable-autosave"));
        let new_game = env::args().any(|arg| arg.eq("--new-game"));
        if disable_autosave {
            println!("Running without autosave");
        }
        let autosave = !disable_autosave;

        let state = if new_game {
            AppState::game(game_interface::setup_new(), autosave)
        } else {
            AppState::main_menu()
        };

        Ok(Self {
            gui: three_d::GUI::new(context),
            assets,
            state,
            autosave,
            close_after_ending: new_game,
        })
    }

    fn handle_frame(&mut self, frame_input: three_d::FrameInput) -> (AppAction, ErrorMessages) {
        match &mut self.state {
            AppState::Game(state) => {
                let game_action =
                    state.handle_game_frame(frame_input, &mut self.gui, &mut self.assets);

                if let Some(GameAction::ExitGame) = game_action {
                    if self.close_after_ending {
                        return (AppAction::Exit, vec![]);
                    } else {
                        self.state = AppState::main_menu();
                    }
                }
            }
            AppState::MainMenu { has_save_file } => {
                let menu_action = handle_menu_frame(*has_save_file, frame_input, &mut self.gui);

                match menu_action {
                    Some(MenuAction::NewGame) => {
                        self.state = AppState::game(game_interface::setup_new(), self.autosave);
                    }
                    Some(MenuAction::LoadGame) => match game_interface::load() {
                        Ok(game) => self.state = AppState::game(game, self.autosave),
                        Err(error) => {
                            let recommendation = if matches!(
                                error,
                                LoadError::UnsupportedVersion(_, _)
                            ) {
                                "Consider starting a new game or using a different version of Aftiktuna."
                            } else {
                                "Consider starting a new game."
                            };
                            return (
                                AppAction::Continue,
                                vec![
                                    format!("Unable to load save file: {error}"),
                                    recommendation.to_string(),
                                ],
                            );
                        }
                    },
                    None => {}
                }
            }
        }

        (AppAction::Continue, vec![])
    }

    fn on_exit(&self) {
        if let AppState::Game(state) = &self.state {
            state.save_game_if_enabled();
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
    fn game(game: Game, is_save_enabled: bool) -> Self {
        Self::Game(Box::new(game::State::init(game, is_save_enabled)))
    }
}

enum GameAction {
    ExitGame,
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

fn default_render_camera(viewport: three_d::Viewport) -> three_d::Camera {
    let mut render_camera = three_d::Camera::new_2d(viewport);
    render_camera.disable_tone_and_color_mapping();
    render_camera
}

fn color_material(color: three_d::Vec4) -> impl three_d::Material {
    UnalteredColorMaterial(
        three_d::ColorMaterial {
            render_states: three_d::RenderStates {
                write_mask: three_d::WriteMask::COLOR,
                blend: three_d::Blend::STANDARD_TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        },
        color,
    )
}

fn texture_material(texture: &three_d::Texture2DRef) -> impl three_d::Material + Clone {
    three_d::ColorMaterial {
        texture: Some(texture.clone()),
        render_states: three_d::RenderStates {
            write_mask: three_d::WriteMask::COLOR,
            blend: three_d::Blend::STANDARD_TRANSPARENCY,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn texture_color_material(
    texture: &three_d::Texture2DRef,
    color: three_d::Vec4,
) -> impl three_d::Material {
    UnalteredColorMaterial(
        three_d::ColorMaterial {
            texture: Some(texture.clone()),
            render_states: three_d::RenderStates {
                write_mask: three_d::WriteMask::COLOR,
                blend: three_d::Blend::STANDARD_TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        },
        color,
    )
}

pub struct UnalteredColorMaterial(pub three_d::ColorMaterial, pub three_d::Vec4);

impl three_d::Material for UnalteredColorMaterial {
    fn id(&self) -> three_d::EffectMaterialId {
        self.0.id()
    }

    fn fragment_shader_source(&self, lights: &[&dyn three_d::Light]) -> String {
        self.0.fragment_shader_source(lights)
    }

    fn use_uniforms(
        &self,
        program: &three_d::Program,
        viewer: &dyn three_d::Viewer,
        _lights: &[&dyn three_d::Light],
    ) {
        viewer.color_mapping().use_uniforms(program);
        program.use_uniform("surfaceColor", self.1);
        if let Some(ref tex) = self.0.texture {
            program.use_uniform("textureTransformation", tex.transformation);
            program.use_texture("tex", tex);
        }
    }

    fn render_states(&self) -> three_d::RenderStates {
        self.0.render_states()
    }

    fn material_type(&self) -> three_d::MaterialType {
        self.0.material_type()
    }
}

fn check_pressed_enter(events: &mut [three_d::Event]) -> bool {
    let mut pressed = false;
    for event in events {
        if let three_d::Event::KeyPress { kind, handled, .. } = event {
            if !*handled && *kind == three_d::Key::Enter {
                *handled = true;
                pressed = true;
            }
        }
    }
    pressed
}

fn check_clicked_anywhere(events: &mut [three_d::Event]) -> bool {
    let mut clicked = false;
    for event in events {
        if let three_d::Event::MousePress {
            button, handled, ..
        } = event
        {
            if !*handled && *button == three_d::MouseButton::Left {
                *handled = true;
                clicked = true;
            }
        }
    }
    clicked
}
