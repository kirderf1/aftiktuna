mod app;

use aftiktuna_three_d::dimensions;
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
            dimensions::WINDOW_WIDTH,
            dimensions::WINDOW_HEIGHT,
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
