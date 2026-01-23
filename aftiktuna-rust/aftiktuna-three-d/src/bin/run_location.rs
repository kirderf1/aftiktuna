use aftiktuna::game_interface;
use aftiktuna::location::GenerationState;
use aftiktuna_three_d::asset::{Assets, BuiltinFonts};
use aftiktuna_three_d::dimensions;
use aftiktuna_three_d::game::{GameAction, State};
use std::env;
use std::rc::Rc;
use three_d::FrameOutput;

const SIZE: (u32, u32) = (
    dimensions::WINDOW_WIDTH as u32,
    dimensions::WINDOW_HEIGHT as u32,
);

fn main() {
    let args: Vec<String> = env::args().collect();
    let location = &args[1];
    let game = game_interface::setup_new_with(
        GenerationState::single(location.to_owned()).expect("Unable to initialize game"),
    )
    .expect("Unable to initialize game");

    let window = three_d::Window::new(three_d::WindowSettings {
        title: format!("Aftiktuna: {location}"),
        min_size: SIZE,
        max_size: Some(SIZE),
        ..Default::default()
    })
    .unwrap();

    let mut assets = Assets::load(window.gl(), Rc::new(BuiltinFonts::init())).unwrap();
    let mut gui = three_d::GUI::new(&window.gl());

    let mut state = State::init(game, false, &mut assets);

    window.render_loop(move |frame_input| {
        let action = state.handle_game_frame(frame_input, &mut gui, &mut assets);
        match action {
            Some(GameAction::ExitGame) => FrameOutput {
                exit: true,
                ..FrameOutput::default()
            },
            None => FrameOutput::default(),
        }
    })
}
