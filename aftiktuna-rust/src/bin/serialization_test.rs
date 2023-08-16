use aftiktuna::area::LocationTracker;
use aftiktuna::{game_loop, serialization};
use std::io::Cursor;

fn main() {
    let mut game = game_loop::setup(LocationTracker::single("location/village".to_string()));
    let _ = game.run();
    let mut cursor = Cursor::new(Vec::<u8>::new());
    serialization::serialize_game(&game, &mut cursor).unwrap();
    cursor.set_position(0);
    let parsed_game = serialization::deserialize_game(&mut cursor).unwrap();
    assert_eq!(game.world.len(), parsed_game.world.len());
    println!("It works!");
}
