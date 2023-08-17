use crate::game_loop::{Game, TakeInput};
use crate::serialization;
use crate::view::Frame;
use std::io::Write;
use std::{io, thread, time};

pub fn run(mut game: Game, frames: Vec<Frame>) {
    let mut last_frame = Frame::Introduction;
    print_frames(frames, &mut last_frame);

    loop {
        let (result, view_buffer) = game.run();
        print_frames(view_buffer.into_frames(), &mut last_frame);

        match result {
            Ok(TakeInput) => {
                println!();
                input_loop(&mut game, &last_frame);
            }
            Err(_) => {
                return;
            }
        }
    }
}

fn input_loop(game: &mut Game, last_frame: &Frame) {
    loop {
        let input = read_input();
        if input.eq_ignore_ascii_case("save") {
            match serialization::write_game_to_save_file(game, vec![last_frame]) {
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

fn print_frames(frames: Vec<Frame>, last_frame: &mut Frame) {
    let mut iter = frames.into_iter();
    while let Some(frame) = iter.next() {
        print_frame(&frame);

        if iter.len() > 0 {
            thread::sleep(time::Duration::from_secs(2));
        } else {
            *last_frame = frame;
        }
    }
}

fn print_frame(frame: &Frame) {
    for line in frame.as_text() {
        println!("{line}");
    }
}
