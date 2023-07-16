use crate::area::Locations;
use crate::game_loop::{Game, TakeInput};
use crate::view::Frame;
use crate::{game_loop, view};
use std::io::Write;
use std::{io, thread, time};

pub fn run(locations: Locations) {
    let mut game = game_loop::setup(locations);

    loop {
        let (result, view_buffer) = game.run();
        print_buffer(view_buffer);

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
        if let Err(messages) = game.handle_input(&input) {
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

fn print_buffer(view_buffer: view::Buffer) {
    let mut iter = view_buffer.into_frames().into_iter();
    while let Some(frame) = iter.next() {
        print_frame(frame);

        if iter.len() > 0 {
            thread::sleep(time::Duration::from_secs(2));
        }
    }
}

fn print_frame(frame: Frame) {
    for line in frame.as_text() {
        println!("{line}");
    }
}
