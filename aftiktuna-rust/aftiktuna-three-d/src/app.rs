use aftiktuna::serialization::LoadError;
use aftiktuna::{game_interface, serialization};
use aftiktuna_three_d::asset::{self, Assets, BuiltinFonts};
use aftiktuna_three_d::game::{self, GameAction};
use aftiktuna_three_d::{dimensions, render};
use std::env;
use std::path::Path;
use std::rc::Rc;
use three_d::egui;
use winit::event::{Event as WinitEvent, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window;

pub struct App {
    loaded_app: Option<LoadedApp>,
    builtin_fonts: Rc<BuiltinFonts>,
    error_messages: ErrorMessages,
    gl: three_d::WindowedContext,
    frame_input_generator: three_d::FrameInputGenerator,
    window: Window,
}

impl App {
    pub fn init(window: Window) -> Self {
        let gl = three_d::WindowedContext::from_winit_window(
            &window,
            three_d::SurfaceSettings::default(),
        )
        .unwrap();

        Self {
            loaded_app: None,
            builtin_fonts: Rc::new(BuiltinFonts::init()),
            error_messages: vec![],
            gl,
            frame_input_generator: three_d::FrameInputGenerator::from_winit_window(&window),
            window,
        }
    }

    pub fn handle_event(&mut self, event: WinitEvent<'_, ()>, control_flow: &mut ControlFlow) {
        match event {
            WinitEvent::WindowEvent { ref event, .. } => {
                self.frame_input_generator.handle_winit_window_event(event);
                match event {
                    WindowEvent::Resized(physical_size) => {
                        self.gl.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => {
                        if let Some(loaded_app) = &mut self.loaded_app {
                            loaded_app.on_exit();
                        }
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.gl.resize(**new_inner_size);
                    }
                    _ => (),
                }
            }
            WinitEvent::MainEventsCleared => {
                self.window.request_redraw();
            }
            WinitEvent::RedrawRequested(_) => {
                let frame_input = self.frame_input_generator.generate(&self.gl);
                let action = self.handle_frame(frame_input);

                match action {
                    AppAction::Continue => {
                        self.gl.swap_buffers().unwrap();
                        *control_flow = ControlFlow::Poll;
                        self.window.request_redraw();

                        if self.loaded_app.is_none() && self.error_messages.is_empty() {
                            match LoadedApp::load(&self.gl, self.builtin_fonts.clone()) {
                                Ok(loaded_app) => self.loaded_app = Some(loaded_app),
                                Err(error) => {
                                    self.error_messages = crate::split_screen_text_lines(
                                        &self.builtin_fonts.text_gen_size_16,
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
        }
    }

    fn handle_frame(&mut self, mut frame_input: three_d::FrameInput) -> AppAction {
        if !self.error_messages.is_empty() {
            let clicked = aftiktuna_three_d::check_clicked_anywhere(&mut frame_input.events);
            let pressed_enter = aftiktuna_three_d::check_pressed_enter(&mut frame_input.events);

            let screen = frame_input.screen();
            screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));
            let mut y = 350.;
            for line in &self.error_messages {
                let text_obj = aftiktuna_three_d::make_centered_text_obj(
                    line,
                    three_d::vec2(dimensions::WINDOW_WIDTH_F / 2., y),
                    three_d::vec4(1., 0.4, 0.7, 1.),
                    &self.builtin_fonts.text_gen_size_16,
                    &frame_input.context,
                );
                screen.render(
                    render::default_render_camera(frame_input.viewport),
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
            self.error_messages = crate::split_screen_text_lines(
                &self.builtin_fonts.text_gen_size_16,
                error_messages,
            );
            app_action
        } else {
            let screen = frame_input.screen();
            screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));
            let text_obj = aftiktuna_three_d::make_centered_text_obj(
                "Loading textures...",
                three_d::vec2(dimensions::WINDOW_WIDTH_F / 2., 300.),
                three_d::vec4(1., 1., 1., 1.),
                &self.builtin_fonts.text_gen_size_20,
                &frame_input.context,
            );
            screen.render(
                render::default_render_camera(frame_input.viewport),
                &[text_obj],
                &[],
            );
            AppAction::Continue
        }
    }
}

enum AppAction {
    Continue,
    Exit,
}

type ErrorMessages = Vec<String>;

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
        let mut assets = Assets::load(context.clone(), builtin_fonts)?;

        let disable_autosave = env::args().any(|arg| arg.eq("--disable-autosave"));
        let new_game = env::args().any(|arg| arg.eq("--new-game"));
        if disable_autosave {
            println!("Running without autosave");
        }
        let autosave = !disable_autosave;

        let state = if new_game {
            let game = game_interface::setup_new()?;
            AppState::game(game, autosave, &mut assets)
        } else {
            AppState::main_menu()
        };

        let gui = three_d::GUI::new(context);
        gui.context().set_theme(egui::Theme::Dark);
        Ok(Self {
            gui,
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
                    Some(MenuAction::NewGame) => match game_interface::setup_new() {
                        Ok(game) => {
                            self.state = AppState::game(game, self.autosave, &mut self.assets)
                        }
                        Err(error) => {
                            return (
                                AppAction::Continue,
                                vec![format!("Unable to load assets:"), format!("{error}")],
                            );
                        }
                    },
                    Some(MenuAction::LoadGame) => match game_interface::load() {
                        Ok(game) => {
                            self.state = AppState::game(game, self.autosave, &mut self.assets)
                        }
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
    fn game(game: game_interface::Game, is_save_enabled: bool, assets: &mut Assets) -> Self {
        Self::Game(Box::new(game::State::init(game, is_save_enabled, assets)))
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
