use winit::dpi;
use winit::event_loop::EventLoop;
use winit::window::{Icon, Window, WindowBuilder, WindowButtons};

mod app;
mod asset;
mod game;

pub const WINDOW_WIDTH: u16 = 800;
pub const WINDOW_HEIGHT: u16 = 600;
pub const WINDOW_WIDTH_F: f32 = WINDOW_WIDTH as f32;
pub const WINDOW_HEIGHT_F: f32 = WINDOW_HEIGHT as f32;

fn main() -> ! {
    let (window, event_loop) = init_window();

    let mut app = app::App::init(window);
    event_loop.run(move |event, _, control_flow| app.handle_event(event, control_flow));
}

fn init_window() -> (Window, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let small_icon = Icon::from_rgba(
        include_bytes!("../../icon/icon_16x16.rgba").to_vec(),
        16,
        16,
    )
    .unwrap();
    let window = WindowBuilder::new()
        .with_title("Aftiktuna")
        .with_window_icon(Some(small_icon))
        .with_decorations(true)
        .with_inner_size(dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_resizable(false)
        .with_enabled_buttons(!WindowButtons::MAXIMIZE)
        .build(&event_loop)
        .unwrap();
    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::WindowExtWindows;
        let large_icon = Icon::from_rgba(
            include_bytes!("../../icon/icon_64x64.rgba").to_vec(),
            64,
            64,
        )
        .unwrap();
        window.set_taskbar_icon(Some(large_icon));
    }
    window.focus_window();

    (window, event_loop)
}

struct BuiltinFonts {
    text_gen_size_16: three_d::TextGenerator<'static>,
    text_gen_size_20: three_d::TextGenerator<'static>,
}

impl BuiltinFonts {
    fn init() -> Self {
        Self {
            text_gen_size_16: three_d::TextGenerator::new(
                epaint_default_fonts::HACK_REGULAR,
                0,
                16.,
            )
            .expect("Unexpected error for builtin font"),
            text_gen_size_20: three_d::TextGenerator::new(
                epaint_default_fonts::HACK_REGULAR,
                0,
                20.,
            )
            .expect("Unexpected error for builtin font"),
        }
    }
}

fn make_centered_text_obj(
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
    three_d::Gm::new(three_d::Mesh::new(context, &mesh), color_material(color))
}

fn make_text_obj(
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
    three_d::Gm::new(three_d::Mesh::new(context, &mesh), color_material(color))
}

fn split_screen_text_lines(
    text_gen: &three_d::TextGenerator<'static>,
    lines: Vec<String>,
) -> Vec<String> {
    lines
        .into_iter()
        .flat_map(|line| {
            if text_fits_on_screen(text_gen, &line) {
                return vec![line];
            }

            let mut remaining_line: &str = &line;
            let mut vec = Vec::new();
            loop {
                let split_index = smallest_screen_text_split(text_gen, remaining_line);
                vec.push(remaining_line[..split_index].to_owned());
                remaining_line = &remaining_line[split_index..];

                if text_fits_on_screen(text_gen, remaining_line) {
                    vec.push(remaining_line.to_owned());
                    return vec;
                }
            }
        })
        .collect()
}

fn text_fits_on_screen(text_gen: &three_d::TextGenerator<'static>, line: &str) -> bool {
    text_gen
        .generate(line, three_d::TextLayoutOptions::default())
        .compute_aabb()
        .size()
        .x
        <= 700.
}

fn smallest_screen_text_split(text_gen: &three_d::TextGenerator<'static>, line: &str) -> usize {
    let mut last_space = 0;
    let mut last_index = 0;
    for (index, char) in line.char_indices() {
        if !text_fits_on_screen(text_gen, &line[..index]) {
            return if last_space != 0 {
                last_space
            } else {
                last_index
            };
        }

        if char.is_whitespace() {
            last_space = index;
        }
        last_index = index;
    }
    line.len()
}

struct Rect {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

impl Rect {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            left: x,
            right: x + width,
            bottom: y,
            top: y + height,
        }
    }

    fn combine(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            right: self.right.max(other.right),
            bottom: self.bottom.min(other.bottom),
            top: self.top.max(other.top),
        }
    }

    fn contains(&self, pos: three_d::Vec2) -> bool {
        self.left <= pos.x && pos.x < self.right && self.bottom <= pos.y && pos.y < self.top
    }
}

fn default_render_camera(viewport: three_d::Viewport) -> three_d::Camera {
    let mut render_camera = three_d::Camera::new_2d(viewport);
    render_camera.disable_tone_and_color_mapping();
    render_camera
}

fn color_material(color: three_d::Vec4) -> impl three_d::Material {
    UnalteredColorMaterial(
        three_d::ColorMaterial {
            render_states: three_d::RenderStates {
                write_mask: three_d::WriteMask::COLOR,
                blend: three_d::Blend::STANDARD_TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        },
        color,
    )
}

fn texture_material(texture: &three_d::Texture2DRef) -> impl three_d::Material + Clone {
    three_d::ColorMaterial {
        texture: Some(texture.clone()),
        render_states: three_d::RenderStates {
            write_mask: three_d::WriteMask::COLOR,
            blend: three_d::Blend::STANDARD_TRANSPARENCY,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn texture_color_material(
    texture: &three_d::Texture2DRef,
    color: three_d::Vec4,
) -> impl three_d::Material {
    UnalteredColorMaterial(
        three_d::ColorMaterial {
            texture: Some(texture.clone()),
            render_states: three_d::RenderStates {
                write_mask: three_d::WriteMask::COLOR,
                blend: three_d::Blend::STANDARD_TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        },
        color,
    )
}

pub struct UnalteredColorMaterial(pub three_d::ColorMaterial, pub three_d::Vec4);

impl three_d::Material for UnalteredColorMaterial {
    fn id(&self) -> three_d::EffectMaterialId {
        self.0.id()
    }

    fn fragment_shader_source(&self, lights: &[&dyn three_d::Light]) -> String {
        self.0.fragment_shader_source(lights)
    }

    fn use_uniforms(
        &self,
        program: &three_d::Program,
        viewer: &dyn three_d::Viewer,
        _lights: &[&dyn three_d::Light],
    ) {
        viewer.color_mapping().use_uniforms(program);
        program.use_uniform("surfaceColor", self.1);
        if let Some(ref tex) = self.0.texture {
            program.use_uniform("textureTransformation", tex.transformation);
            program.use_texture("tex", tex);
        }
    }

    fn render_states(&self) -> three_d::RenderStates {
        self.0.render_states()
    }

    fn material_type(&self) -> three_d::MaterialType {
        self.0.material_type()
    }
}

fn check_pressed_enter(events: &mut [three_d::Event]) -> bool {
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

fn check_clicked_anywhere(events: &mut [three_d::Event]) -> bool {
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
