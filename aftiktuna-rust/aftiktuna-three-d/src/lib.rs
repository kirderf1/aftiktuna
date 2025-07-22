pub mod asset;
pub mod render;

mod camera {
    use aftiktuna::asset::placement;
    use aftiktuna::core::position::Coord;

    #[derive(Default)]
    pub struct Camera {
        pub camera_x: f32,
        pub is_dragging: bool,
    }

    impl Camera {
        pub fn set_center(&mut self, coord: Coord) {
            self.camera_x = placement::coord_to_center_x(coord) - crate::WINDOW_WIDTH_F / 2.;
        }

        pub fn handle_inputs(&mut self, events: &mut [three_d::Event]) {
            for event in events {
                match event {
                    three_d::Event::MousePress {
                        button, handled, ..
                    } => {
                        if !*handled && *button == three_d::MouseButton::Left {
                            self.is_dragging = true;
                            *handled = true;
                        }
                    }
                    three_d::Event::MouseRelease {
                        button, handled, ..
                    } => {
                        if self.is_dragging && *button == three_d::MouseButton::Left {
                            self.is_dragging = false;
                            *handled = true;
                        }
                    }
                    three_d::Event::MouseMotion { delta, handled, .. } => {
                        if !*handled && self.is_dragging {
                            self.camera_x -= delta.0;
                            *handled = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        pub fn clamp(&mut self, area_size: Coord) {
            self.camera_x = if area_size <= 6 {
                (placement::coord_to_center_x(0) + placement::coord_to_center_x(area_size - 1)) / 2.
                    - crate::WINDOW_WIDTH_F / 2.
            } else {
                self.camera_x.clamp(
                    placement::coord_to_center_x(0) - 100.,
                    placement::coord_to_center_x(area_size - 1) + 100. - crate::WINDOW_WIDTH_F,
                )
            };
        }

        pub fn has_space_to_drag(&self, area_size: Coord) -> [bool; 2] {
            if area_size <= 6 {
                [false, false]
            } else {
                [
                    self.camera_x > placement::coord_to_center_x(0) - 100.,
                    self.camera_x + crate::WINDOW_WIDTH_F
                        < placement::coord_to_center_x(area_size - 1) + 100.,
                ]
            }
        }
    }
}

pub use camera::Camera;

pub const WINDOW_WIDTH: u16 = 800;
pub const WINDOW_HEIGHT: u16 = 600;
pub const WINDOW_WIDTH_F: f32 = WINDOW_WIDTH as f32;
pub const WINDOW_HEIGHT_F: f32 = WINDOW_HEIGHT as f32;

pub struct Rect {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            left: x,
            right: x + width,
            bottom: y,
            top: y + height,
        }
    }

    pub fn combine(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            right: self.right.max(other.right),
            bottom: self.bottom.min(other.bottom),
            top: self.top.max(other.top),
        }
    }

    pub fn contains(&self, pos: three_d::Vec2) -> bool {
        self.left <= pos.x && pos.x < self.right && self.bottom <= pos.y && pos.y < self.top
    }
}

pub fn to_vec(pos: aftiktuna::Vec2, direction_mod: f32) -> three_d::Vec2 {
    three_d::vec2(direction_mod * pos.x, -pos.y)
}
