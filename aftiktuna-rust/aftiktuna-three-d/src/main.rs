use aftiktuna_three_d::dimensions;
use winit::dpi;
use winit::event_loop::EventLoop;
use winit::window::{Icon, Window, WindowBuilder, WindowButtons};

fn main() -> ! {
    let (window, event_loop) = init_window();

    let mut app = aftiktuna_three_d::app::App::init(window);
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
