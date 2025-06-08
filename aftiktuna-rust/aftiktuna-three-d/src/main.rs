mod app;
mod game;

use aftiktuna::asset::color::{self, AftikColorData};
use aftiktuna::core::display::AftikColorId;
use aftiktuna_three_d::asset::{self, BackgroundMap, LazilyLoadedModels};
use aftiktuna_three_d::render;
use std::collections::HashMap;
use std::rc::Rc;
use winit::dpi;
use winit::event_loop::EventLoop;
use winit::window::{Icon, Window, WindowBuilder, WindowButtons};

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
        .with_inner_size(dpi::LogicalSize::new(
            aftiktuna_three_d::WINDOW_WIDTH,
            aftiktuna_three_d::WINDOW_HEIGHT,
        ))
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

struct Assets {
    backgrounds: BackgroundMap,
    models: LazilyLoadedModels,
    aftik_colors: HashMap<AftikColorId, AftikColorData>,
    left_mouse_icon: three_d::Texture2DRef,
    side_arrow_texture: three_d::Texture2DRef,
    builtin_fonts: Rc<BuiltinFonts>,
}

impl Assets {
    fn load(
        context: three_d::Context,
        builtin_fonts: Rc<BuiltinFonts>,
    ) -> Result<Self, asset::Error> {
        let left_mouse_icon = asset::load_texture("left_mouse", &context)?;
        let side_arrow_texture = asset::load_texture("side_arrow", &context)?;
        Ok(Self {
            backgrounds: BackgroundMap::load(context.clone())?,
            models: LazilyLoadedModels::new(context)?,
            aftik_colors: color::load_aftik_color_data()?,
            left_mouse_icon,
            side_arrow_texture,
            builtin_fonts,
        })
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
    three_d::Gm::new(
        three_d::Mesh::new(context, &mesh),
        render::color_material(color),
    )
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
    three_d::Gm::new(
        three_d::Mesh::new(context, &mesh),
        render::color_material(color),
    )
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
