pub mod macroquad_interface;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value.eq(&Default::default())
}
