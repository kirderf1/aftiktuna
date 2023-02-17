use crate::area::Locations;
use crate::game_loop::TakeInput;
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
                view_buffer.print();
                let input = read_input();
                if let Err(messages) = game.handle_input(&input) {
                    messages.print_lines();
                }
            }
            Err(stop_type) => {
                view_buffer.print();
                thread::sleep(time::Duration::from_secs(2));
                println!();
                stop_type.messages().print_lines();
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
