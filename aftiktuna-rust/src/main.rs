use aftiktuna::macroquad_interface::egui::EguiWrapper;
use aftiktuna::macroquad_interface::error_view;
use aftiktuna::macroquad_interface::texture::RenderAssets;
use aftiktuna::serialization::{self, LoadError};
use aftiktuna::{game_interface, macroquad_interface};
use macroquad::color::{self, Color};
use macroquad::input;
use macroquad::math::Vec2;
use macroquad::ui::widgets::Button;
use macroquad::ui::{self, Skin};
use macroquad::window::{self, Conf};
use std::env;
use std::path::Path;

fn config() -> Conf {
    macroquad_interface::default_conf("Aftiktuna")
}

#[macroquad::main(config)]
async fn main() {
    let disable_autosave = env::args().any(|arg| arg.eq("--disable-autosave"));
    let new_name = env::args().any(|arg| arg.eq("--new-game"));
    if disable_autosave {
        println!("Running without autosave");
    }

    window::next_frame().await;
    let mut assets = macroquad_interface::load_assets().await;
    input::prevent_quit();

    let mut egui = EguiWrapper::init();

    if new_name {
        return run_new_game(disable_autosave, &mut assets, &mut egui).await;
    }

    loop {
        let action = run_menu().await;
        match action {
            MenuAction::NewGame => {
                run_new_game(disable_autosave, &mut assets, &mut egui).await;
            }
            MenuAction::LoadGame => {
                run_load_game(disable_autosave, &mut assets, &mut egui).await;
            }
        }
    }
}

async fn run_new_game(disable_autosave: bool, assets: &mut RenderAssets, egui: &mut EguiWrapper) {
    let game = game_interface::setup_new();
    macroquad_interface::run_game(game, !disable_autosave, assets, egui).await
}

async fn run_load_game(disable_autosave: bool, assets: &mut RenderAssets, egui: &mut EguiWrapper) {
    match game_interface::load() {
        Ok(game) => macroquad_interface::run_game(game, !disable_autosave, assets, egui).await,
        Err(error) => {
            let recommendation = if matches!(error, LoadError::UnsupportedVersion(_, _)) {
                "Consider starting a new game or using a different version of Aftiktuna."
            } else {
                "Consider starting a new game."
            };
            error_view::show(vec![
                format!("Unable to load save file: {error}"),
                recommendation.to_string(),
            ])
            .await
        }
    }
}

enum MenuAction {
    NewGame,
    LoadGame,
}

async fn run_menu() -> MenuAction {
    fn button(y: f32, text: &str) -> Button {
        let size = Vec2::new(200., 60.);
        Button::new(text)
            .size(size)
            .position(Vec2::new(400. - size.x / 2., y))
    }

    let button_style = ui::root_ui()
        .style_builder()
        .color(Color::new(0.2, 0.1, 0.4, 0.6))
        .color_hovered(Color::new(0.5, 0.3, 0.6, 0.6))
        .text_color(color::WHITE)
        .text_color_hovered(color::WHITE)
        .font_size(32)
        .build();

    let skin = Skin {
        button_style,
        ..ui::root_ui().default_skin()
    };

    let has_save_file = Path::new(serialization::SAVE_FILE_NAME).exists();
    macroquad_interface::run(|| {
        window::clear_background(color::BLACK);

        macroquad_interface::draw_centered_text("AFTIKTUNA", 200., 128, color::WHITE);

        ui::root_ui().push_skin(&skin);

        let mut action = None;
        if button(350., "New Game").ui(&mut ui::root_ui()) {
            action = Some(MenuAction::NewGame);
        }

        if has_save_file && button(450., "Load Game").ui(&mut ui::root_ui()) {
            action = Some(MenuAction::LoadGame);
        }
        ui::root_ui().pop_skin();

        if let Some(action) = action {
            return Err(action);
        }
        Ok(())
    })
    .await
}
