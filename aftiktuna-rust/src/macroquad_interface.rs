use crate::game_interface::{Game, GameResult};
use crate::macroquad_interface::tooltip::CommandTooltip;
use crate::serialization;
use crate::view::Frame;
use egui::EguiWrapper;
use macroquad::color::{self, Color};
use macroquad::input::{self, KeyCode, MouseButton};
use macroquad::miniquad::conf::Icon;
use macroquad::text;
use macroquad::window::{self, Conf};
use std::fs;
use std::mem;
use std::process;
use std::time::{self, Instant};

pub mod camera;
mod render;
mod store_render;
pub mod texture;
mod tooltip;
mod ui;

pub mod error_view {
    use macroquad::{color, input, text, window};

    const TEXT_SIZE: u16 = 24;

    pub async fn show(messages: Vec<String>) {
        let messages = messages
            .into_iter()
            .flat_map(split_text_line)
            .collect::<Vec<_>>();
        super::run(|| {
            if input::is_key_pressed(input::KeyCode::Enter)
                || input::is_mouse_button_pressed(input::MouseButton::Left)
            {
                return Err(());
            }

            window::clear_background(color::BLACK);

            let mut y = 250.;
            for message in &messages {
                super::draw_centered_text(message, y, TEXT_SIZE, color::PINK);
                y += TEXT_SIZE as f32;
            }
            Ok(())
        })
        .await
    }

    fn split_text_line(line: String) -> Vec<String> {
        if fits_on_screen(&line) {
            return vec![line];
        }

        let mut remaining_line: &str = &line;
        let mut vec = Vec::new();
        loop {
            let split_index = smallest_split(remaining_line);
            vec.push(remaining_line[..split_index].to_owned());
            remaining_line = &remaining_line[split_index..];

            if fits_on_screen(remaining_line) {
                vec.push(remaining_line.to_owned());
                return vec;
            }
        }
    }

    fn fits_on_screen(line: &str) -> bool {
        text::measure_text(line, None, TEXT_SIZE, 1.).width <= 700.
    }

    fn smallest_split(line: &str) -> usize {
        let mut last_space = 0;
        let mut last_index = 0;
        for (index, char) in line.char_indices() {
            if !fits_on_screen(&line[..index]) {
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
}

pub const WINDOW_WIDTH: u16 = 800;
pub const WINDOW_HEIGHT: u16 = 600;
pub const WINDOW_WIDTH_F: f32 = WINDOW_WIDTH as f32;
pub const WINDOW_HEIGHT_F: f32 = WINDOW_HEIGHT as f32;

pub fn default_conf(title: impl Into<String>) -> Conf {
    Conf {
        window_title: title.into(),
        window_width: WINDOW_WIDTH.into(),
        window_height: WINDOW_HEIGHT.into(),
        window_resizable: false,
        icon: Some(logo()),
        ..Default::default()
    }
}

pub fn logo() -> Icon {
    Icon {
        small: *include_bytes!("../icon/icon_16x16.rgba"),
        medium: *include_bytes!("../icon/icon_32x32.rgba"),
        big: *include_bytes!("../icon/icon_64x64.rgba"),
    }
}

pub async fn load_assets() -> texture::RenderAssets {
    window::clear_background(color::BLACK);
    draw_centered_text("Loading textures...", 300., 32, color::LIGHTGRAY);
    window::next_frame().await;

    match texture::load_assets() {
        Ok(assets) => assets,
        Err(error) => {
            error_view::show(vec![format!("Unable to load assets:"), format!("{error}")]).await;
            process::exit(0)
        }
    }
}

pub trait Interface<T> {
    fn on_frame(&mut self) -> Result<(), T>;

    fn on_quit(&mut self) {}
}

impl<T, F: FnMut() -> Result<(), T>> Interface<T> for F {
    fn on_frame(&mut self) -> Result<(), T> {
        self()
    }
}

pub async fn run<T>(mut interface: impl Interface<T>) -> T {
    loop {
        if input::is_quit_requested() {
            interface.on_quit();
            process::exit(0);
        }

        let result = interface.on_frame();

        window::next_frame().await;

        if let Err(value) = result {
            return value;
        }
    }
}

pub async fn run_game(
    game: Game,
    autosave: bool,
    assets: &mut texture::RenderAssets,
    egui: &mut EguiWrapper,
) {
    let app = App {
        input: String::new(),
        game,
        assets,
        last_frame_time: None,
        render_state: render::State::new(),
        command_tooltip: None,
        show_graphical: true,
        request_input_focus: false,
        given_exit_command: false,
        show_next_frame: true,
        autosave,
    };
    run(AppWithEgui { app, egui }).await
}

struct AppWithEgui<'a> {
    app: App<'a>,
    egui: &'a mut EguiWrapper,
}

struct App<'a> {
    input: String,
    game: Game,
    assets: &'a mut texture::RenderAssets,
    last_frame_time: Option<Instant>,
    render_state: render::State,
    command_tooltip: Option<CommandTooltip>,
    show_graphical: bool,
    request_input_focus: bool,
    given_exit_command: bool,
    show_next_frame: bool,
    autosave: bool,
}

impl Interface<()> for AppWithEgui<'_> {
    fn on_frame(&mut self) -> Result<(), ()> {
        let app = &mut self.app;
        if matches!(app.render_state.current_frame, Frame::Ending { .. })
            && (input::is_key_pressed(KeyCode::Enter)
                || input::is_mouse_button_pressed(MouseButton::Left))
        {
            return Err(());
        }

        if input::is_key_pressed(KeyCode::Tab) {
            app.show_graphical = !app.show_graphical;
        }

        if app.show_graphical {
            if !app.render_state.camera.is_dragging() {
                tooltip::handle_click(app);
            }
            if app.command_tooltip.is_none() {
                camera::try_drag_camera_for_state(&mut app.render_state);
            }
        } else {
            app.render_state.camera.stop_dragging();
            app.command_tooltip = None;
        }

        app.update_frame_state();

        render::draw(self);

        if self.app.given_exit_command {
            self.app.save_game_if_relevant();
            Err(())
        } else {
            Ok(())
        }
    }

    fn on_quit(&mut self) {
        self.app.save_game_if_relevant();
    }
}

const DELAY: time::Duration = time::Duration::from_secs(2);

impl App<'_> {
    fn update_frame_state(&mut self) {
        if self.show_graphical {
            self.show_next_frame |= input::is_key_pressed(KeyCode::Enter)
                || input::is_mouse_button_pressed(MouseButton::Left)
                    && ui::is_mouse_at_text_box(&self.render_state.text_box_text)
        } else {
            self.show_next_frame |= self
                .last_frame_time
                .map_or(true, |instant| instant.elapsed() >= DELAY);
        }

        if self.show_next_frame {
            if let GameResult::Frame(frame_getter) = self.game.next_result() {
                let frame = frame_getter.get();
                self.show_frame(frame);
            }
            self.show_next_frame = false;
        }
    }

    fn show_frame(&mut self, frame: Frame) {
        if matches!(frame, Frame::Ending { .. }) {
            let _ = fs::remove_file(serialization::SAVE_FILE_NAME);
        }
        self.last_frame_time = Some(Instant::now());
        let ready_for_input = self.game.ready_to_take_input();
        self.render_state.show_frame(frame, ready_for_input);
        self.request_input_focus = ready_for_input;
        self.command_tooltip = None;
    }

    fn handle_input(&mut self) {
        let input = mem::take(&mut self.input);
        if input.is_empty() {
            return;
        }
        if input.eq_ignore_ascii_case("exit game") {
            self.given_exit_command = true;
            return;
        }

        self.render_state.add_to_text_log(format!("> {input}"));
        match self.game.handle_input(&input) {
            Ok(()) => {
                self.show_next_frame = true;
            }
            Err(text_lines) => {
                self.render_state.show_input_text_lines(text_lines);
                self.request_input_focus = true;
            }
        }
    }

    fn save_game_if_relevant(&self) {
        if self.autosave && !matches!(self.render_state.current_frame, Frame::Ending { .. }) {
            if let Err(error) = serialization::write_game_to_save_file(&self.game) {
                eprintln!("Failed to save game: {error}");
            } else {
                println!("Saved the game successfully.")
            }
        }
    }
}

pub fn draw_centered_text(text: &str, y: f32, font_size: u16, color: Color) {
    let text_size = text::measure_text(text, None, font_size, 1.);
    text::draw_text(
        text,
        (WINDOW_WIDTH_F - text_size.width) / 2.,
        y,
        font_size as f32,
        color,
    );
}

pub mod egui {
    use macroquad::input;
    use macroquad::miniquad;
    use macroquad::window::get_internal_gl;

    pub struct EguiWrapper {
        egui_mq: egui_miniquad::EguiMq,
        input_subscriber: usize,
    }

    impl EguiWrapper {
        pub fn init() -> Self {
            let gl = unsafe { get_internal_gl() };
            let egui_mq = egui_miniquad::EguiMq::new(gl.quad_context);
            Self {
                egui_mq,
                input_subscriber: input::utils::register_input_subscriber(),
            }
        }

        pub fn ui(&mut self, f: impl FnOnce(&egui::Context)) {
            input::utils::repeat_all_miniquad_input(
                &mut EventTransfer(&mut self.egui_mq),
                self.input_subscriber,
            );
            let gl = unsafe { get_internal_gl() };
            self.egui_mq.run(gl.quad_context, |_, ctx| f(ctx));
        }

        pub fn draw(&mut self) {
            let mut gl = unsafe { get_internal_gl() };
            gl.flush();
            self.egui_mq.draw(gl.quad_context);
        }
    }

    struct EventTransfer<'a>(&'a mut egui_miniquad::EguiMq);

    impl<'a> miniquad::EventHandler for EventTransfer<'a> {
        fn mouse_motion_event(&mut self, x: f32, y: f32) {
            self.0.mouse_motion_event(x, y)
        }

        fn mouse_wheel_event(&mut self, x: f32, y: f32) {
            self.0.mouse_wheel_event(x / 50., y / 50.)
        }

        fn mouse_button_down_event(&mut self, button: miniquad::MouseButton, x: f32, y: f32) {
            self.0.mouse_button_down_event(button, x, y)
        }

        fn mouse_button_up_event(&mut self, button: miniquad::MouseButton, x: f32, y: f32) {
            self.0.mouse_button_up_event(button, x, y)
        }

        fn char_event(&mut self, character: char, _keymods: miniquad::KeyMods, _repeat: bool) {
            self.0.char_event(character)
        }

        fn key_down_event(
            &mut self,
            keycode: miniquad::KeyCode,
            keymods: miniquad::KeyMods,
            _repeat: bool,
        ) {
            self.0.key_down_event(keycode, keymods)
        }

        fn key_up_event(&mut self, keycode: miniquad::KeyCode, keymods: miniquad::KeyMods) {
            self.0.key_up_event(keycode, keymods)
        }

        fn update(&mut self) {}

        fn draw(&mut self) {}
    }
}
