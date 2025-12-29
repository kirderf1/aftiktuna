pub mod asset;
pub mod game;
pub mod render;

mod camera {
    use crate::dimensions;
    use aftiktuna::asset::placement;
    use aftiktuna::core::position::Coord;

    #[derive(Default)]
    pub struct Camera {
        pub camera_x: f32,
        pub is_dragging: bool,
    }

    impl Camera {
        pub fn set_center(&mut self, coord: Coord) {
            self.camera_x = placement::coord_to_center_x(coord) - dimensions::WINDOW_WIDTH_F / 2.;
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
                    - dimensions::WINDOW_WIDTH_F / 2.
            } else {
                self.camera_x.clamp(
                    placement::coord_to_center_x(0) - 100.,
                    placement::coord_to_center_x(area_size - 1) + 100. - dimensions::WINDOW_WIDTH_F,
                )
            };
        }

        pub fn has_space_to_drag(&self, area_size: Coord) -> [bool; 2] {
            if area_size <= 6 {
                [false, false]
            } else {
                [
                    self.camera_x > placement::coord_to_center_x(0) - 100.,
                    self.camera_x + dimensions::WINDOW_WIDTH_F
                        < placement::coord_to_center_x(area_size - 1) + 100.,
                ]
            }
        }
    }
}

pub mod dimensions {
    pub const WINDOW_WIDTH: u16 = 800;
    pub const WINDOW_HEIGHT: u16 = 600;
    pub const WINDOW_WIDTH_F: f32 = WINDOW_WIDTH as f32;
    pub const WINDOW_HEIGHT_F: f32 = WINDOW_HEIGHT as f32;
}

pub use camera::Camera;

const TRANSPARENCY_BLEND: three_d::Blend = three_d::Blend::TRANSPARENCY;

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
    three_d::vec2(direction_mod * pos.x, pos.y)
}

pub fn make_centered_text_obj(
    text: &str,
    pos: three_d::Vec2,
    color: three_d::Vec4,
    text_gen: &three_d::TextGenerator<'static>,
    context: &three_d::Context,
) -> impl three_d::Object {
    let mut mesh = text_gen.generate(text, three_d::TextLayoutOptions::default());
    mesh.transform(three_d::Matrix4::from_translation(three_d::vec3(
        pos.x - (mesh.compute_aabb().size().x) / 2.,
        pos.y,
        0.,
    )))
    .unwrap();
    three_d::Gm::new(
        three_d::Mesh::new(context, &mesh),
        render::color_material(color),
    )
}

pub fn make_text_obj(
    text: &str,
    pos: three_d::Vec2,
    color: three_d::Vec4,
    text_gen: &three_d::TextGenerator<'static>,
    context: &three_d::Context,
) -> impl three_d::Object {
    let mut mesh = text_gen.generate(text, three_d::TextLayoutOptions::default());
    mesh.transform(three_d::Matrix4::from_translation(three_d::vec3(
        pos.x, pos.y, 0.,
    )))
    .unwrap();
    three_d::Gm::new(
        three_d::Mesh::new(context, &mesh),
        render::color_material(color),
    )
}

pub fn check_pressed_enter(events: &mut [three_d::Event]) -> bool {
    let mut pressed = false;
    for event in events {
        if let three_d::Event::KeyPress { kind, handled, .. } = event {
            if !*handled && *kind == three_d::Key::Enter {
                *handled = true;
                pressed = true;
            }
        }
    }
    pressed
}

pub fn check_clicked_anywhere(events: &mut [three_d::Event]) -> bool {
    let mut clicked = false;
    for event in events {
        if let three_d::Event::MousePress {
            button, handled, ..
        } = event
        {
            if !*handled && *button == three_d::MouseButton::Left {
                *handled = true;
                clicked = true;
            }
        }
    }
    clicked
}
