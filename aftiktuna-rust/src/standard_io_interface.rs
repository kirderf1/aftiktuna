use crate::game_loop::{FrameCache, Game, TakeInput};
use crate::serialization;
use crate::view::Frame;
use std::io::Write;
use std::{io, thread, time};

pub fn run(mut game: Game) {
    print_frames(&mut game.frame_cache);

    loop {
        let result = game.run();
        print_frames(&mut game.frame_cache);

        match result {
            Ok(TakeInput) => {
                println!();
                input_loop(&mut game);
            }
            Err(_) => {
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
        } else if let Err(messages) = game.handle_input(&input) {
            messages.print_lines();
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

fn print_frames(frame_cache: &mut FrameCache) {
    while let Some(frame) = frame_cache.take_next_frame() {
        print_frame(&frame);

        if frame_cache.has_more_frames() {
            thread::sleep(time::Duration::from_secs(2));
        }
    }
}

fn print_frame(frame: &Frame) {
    for line in frame.as_text() {
        println!("{line}");
    }
}
