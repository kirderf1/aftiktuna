use crate::game_interface::{Game, GameResult};
use crate::macroquad_interface::tooltip::CommandTooltip;
use crate::serialization;
use crate::view::Frame;
use egui_macroquad::macroquad::input;
use egui_macroquad::macroquad::input::{
    is_key_pressed, is_mouse_button_released, KeyCode, MouseButton,
};
use egui_macroquad::macroquad::math::Vec2;
use egui_macroquad::macroquad::miniquad::conf::Icon;
use egui_macroquad::macroquad::window::next_frame;
use std::mem::take;
use std::process::exit;
use std::time;
use std::time::Instant;

mod camera;
mod render;
mod store_render;
mod texture;
mod tooltip;
mod ui;

pub fn logo() -> Icon {
    Icon {
        small: *include_bytes!("../icon/icon_16x16.rgba"),
        medium: *include_bytes!("../icon/icon_32x32.rgba"),
        big: *include_bytes!("../icon/icon_64x64.rgba"),
    }
}

pub async fn run(game: Game, autosave: bool) -> ! {
    let mut app = init(game);
    let textures = texture::load_textures().await.unwrap();

    if autosave {
        input::prevent_quit();
    }
    loop {
        if autosave && input::is_quit_requested() {
            if let Err(error) = serialization::write_game_to_save_file(&app.game) {
                eprintln!("Failed to save game: {error}");
            } else {
                println!("Saved the game successfully.")
            }
            exit(0);
        }

        if is_key_pressed(KeyCode::Tab) {
            app.show_graphical = !app.show_graphical;
        }

        if app.show_graphical {
            if app.last_drag_pos.is_none() {
                tooltip::handle_click(&mut app, &textures);
            }
            if app.command_tooltip.is_none() {
                camera::try_drag_camera(&mut app.render_state, &mut app.last_drag_pos);
            }
        } else {
            app.last_drag_pos = None;
            app.command_tooltip = None;
        }

        app.update_frame_state();

        render::draw(&mut app, &textures);

        next_frame().await
    }
}

fn init(game: Game) -> App {
    App {
        input: String::new(),
        game,
        last_frame_time: None,
        render_state: render::State::new(),
        last_drag_pos: None,
        command_tooltip: None,
        show_graphical: true,
        request_input_focus: false,
        show_next_frame: true,
    }
}

pub struct App {
    input: String,
    game: Game,
    last_frame_time: Option<Instant>,
    render_state: render::State,
    last_drag_pos: Option<Vec2>,
    command_tooltip: Option<CommandTooltip>,
    show_graphical: bool,
    request_input_focus: bool,
    show_next_frame: bool,
}

const DELAY: time::Duration = time::Duration::from_secs(2);

impl App {
    fn update_frame_state(&mut self) {
        if self.show_graphical {
            self.show_next_frame |= is_key_pressed(KeyCode::Enter)
                || is_mouse_button_released(MouseButton::Left)
                    && ui::is_mouse_at_text_box(&self.render_state)
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
        self.last_frame_time = Some(Instant::now());
        let ready_for_input = self.game.ready_to_take_input();
        self.render_state.show_frame(frame, ready_for_input);
        self.request_input_focus = ready_for_input;
        self.command_tooltip = None;
    }

    fn handle_input(&mut self) {
        let input = take(&mut self.input);
        if !input.is_empty() {
            self.render_state.add_to_text_log(format!("> {input}"));
            match self.game.handle_input(&input) {
                Ok(()) => {
                    self.show_next_frame = true;
                }
                Err(messages) => {
                    self.render_state.show_input_error(messages);
                    self.request_input_focus = true;
                }
            }
        }
    }
}
