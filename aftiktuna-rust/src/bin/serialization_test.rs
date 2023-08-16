use aftiktuna::{area, serialization};
use hecs::World;
use std::io::Cursor;

fn main() {
    let mut world = World::new();
    area::load_data("location/village")
        .unwrap()
        .build(&mut world)
        .unwrap();
    let mut cursor = Cursor::new(Vec::<u8>::new());
    serialization::serialize_world(&world, &mut cursor);
    cursor.set_position(0);
    let parsed_world = serialization::deserialize_world(&mut cursor);
    assert_eq!(world.len(), parsed_world.len());
    println!("It works!");
}
