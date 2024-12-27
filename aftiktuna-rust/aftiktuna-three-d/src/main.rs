use aftiktuna::asset::background::BGData;
use aftiktuna::asset::{background, TextureLoader};
use aftiktuna::core::area::BackgroundId;
use winit::dpi;
use winit::event_loop::EventLoop;
use winit::platform::windows::WindowBuilderExtWindows;
use winit::window::{Icon, WindowBuilder, WindowButtons};

fn main() {
    let event_loop = EventLoop::new();
    let small_icon = Icon::from_rgba(
        include_bytes!("../../icon/icon_16x16.rgba").to_vec(),
        16,
        16,
    )
    .unwrap();
    let large_icon = Icon::from_rgba(
        include_bytes!("../../icon/icon_64x64.rgba").to_vec(),
        64,
        64,
    )
    .unwrap();
    let winit_window = WindowBuilder::new()
        .with_title("Aftiktuna")
        .with_window_icon(Some(small_icon))
        .with_taskbar_icon(Some(large_icon))
        .with_decorations(true)
        .with_inner_size(dpi::LogicalSize::new(800, 600))
        .with_resizable(false)
        .with_enabled_buttons(!WindowButtons::MAXIMIZE)
        .build(&event_loop)
        .unwrap();
    winit_window.focus_window();

    let window = three_d::Window::from_winit_window(
        winit_window,
        event_loop,
        three_d::SurfaceSettings::default(),
        false,
    )
    .expect("Unable to create window");

    let background = background::load_raw_backgrounds()
        .unwrap()
        .get(&BackgroundId::blank())
        .unwrap()
        .load(&mut InPlaceLoader(window.gl()))
        .unwrap();

    window.render_loop(move |frame_input| {
        let camera = three_d::Camera::new_2d(frame_input.viewport);

        frame_input
            .screen()
            .clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.))
            .render(
                &camera,
                get_render_objects_for_background(&background, &frame_input.context),
                &[],
            );
        three_d::FrameOutput::default()
    });
}

struct InPlaceLoader(three_d::Context);

impl TextureLoader<three_d::Texture2DRef, ()> for InPlaceLoader {
    fn load_texture(&mut self, name: String) -> Result<three_d::Texture2DRef, ()> {
        let path = format!("assets/texture/{name}.png");

        let texture: three_d::CpuTexture = three_d_asset::io::load_and_deserialize(path).unwrap();
        Ok(three_d::Texture2DRef::from_cpu_texture(&self.0, &texture))
    }
}

fn get_render_objects_for_background(
    background: &BGData<three_d::Texture2DRef>,
    context: &three_d::Context,
) -> Vec<impl three_d::Object> {
    background
        .primary
        .0
        .layers
        .iter()
        .map(|layer| {
            three_d::Gm::new(
                three_d::Rectangle::new(
                    context,
                    three_d::vec2(400., 300.),
                    three_d::degrees(0.),
                    800.,
                    600.,
                ),
                three_d::ColorMaterial {
                    color: three_d::Srgba::WHITE,
                    texture: Some(layer.texture.clone()),
                    ..Default::default()
                },
            )
        })
        .collect()
}
