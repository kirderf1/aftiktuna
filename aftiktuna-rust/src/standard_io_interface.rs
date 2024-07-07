use crate::game_interface::{Game, GameResult};
use crate::serialization;
use crate::view::Frame;
use std::io::Write;
use std::{io, thread, time};

pub fn run(mut game: Game) {
    loop {
        match game.next_result() {
            GameResult::Frame(frame_getter) => {
                print_frame(&frame_getter.get());

                if game.next_result().has_frame() {
                    thread::sleep(time::Duration::from_secs(2));
                }
            }
            GameResult::Input => {
                println!();
                input_loop(&mut game);
            }
            GameResult::Stop => {
                return;
            }
        }
    }
}

fn input_loop(game: &mut Game) {
    loop {
        let input = read_input();
        if input.eq_ignore_ascii_case("save") {
            match serialization::write_game_to_save_file(game) {
                Ok(()) => println!(
                    "Successfully saved the game to {}",
                    serialization::SAVE_FILE_NAME
                ),
                Err(error) => eprintln!("Unable to save the game: {}", error),
            }
        } else if let Err(text_lines) = game.handle_input(&input) {
            for line in text_lines {
                println!("{line}");
            }
        } else {
            break;
        }
    }
}

fn read_input() -> String {
    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    String::from(input.trim())
}

fn print_frame(frame: &Frame) {
    for line in frame.as_text() {
        println!("{line}");
    }
}
