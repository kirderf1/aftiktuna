use wasm_bindgen::prelude::*;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    use winit::platform::web::WindowBuilderExtWebSys;

    let event_loop = EventLoop::new();
    let window = {
        let canvas = web_sys::window()
            .expect("Web window not found")
            .document()
            .expect("Web document not found")
            .get_element_by_id("game_canvas")
            .expect("Game canvas not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Game canvas is not of the correct type");

        winit::window::WindowBuilder::new()
            .with_title("Aftiktuna")
            .with_canvas(Some(canvas))
            .with_inner_size(LogicalSize::new(800, 600))
            .with_prevent_default(true)
            .build(&event_loop)
            .expect("Failed to build winit window")
    };

    let mut app = crate::app::App::init(window);
    event_loop.run(move |event, _, control_flow| app.handle_event(event, control_flow));
}
