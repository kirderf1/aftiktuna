use crate::area::Locations;
use crate::game_loop::TakeInput;
use crate::view::Frame;
use crate::{game_loop, view};
use std::io::Write;
use std::{io, thread, time};

pub fn run(locations: Locations) {
    let (messages, mut game) = game_loop::setup(locations);
    messages.print_lines();

    loop {
        let mut view_buffer = view::Buffer::default();
        match game.run(&mut view_buffer) {
            Ok(TakeInput) => {
                print_buffer(view_buffer);
                let input = read_input();
                if let Err(messages) = game.handle_input(&input) {
                    messages.print_lines();
                }
            }
            Err(stop_type) => {
                view_buffer.push_frame(Frame::Ending(stop_type));
                print_buffer(view_buffer);
                return;
            }
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
