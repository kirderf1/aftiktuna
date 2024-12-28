use aftiktuna::asset::background::BGData;
use aftiktuna::asset::{background, TextureLoader};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::game_interface::{self, Game, GameResult};
use three_d::egui;
use winit::dpi;
use winit::event_loop::EventLoop;
use winit::platform::windows::WindowBuilderExtWindows;
use winit::window::{Icon, WindowBuilder, WindowButtons};

fn main() {
    let window = init_window();

    let mut app = App::init(window.gl());

    window.render_loop(move |frame_input| app.handle_frame(frame_input));
}

fn init_window() -> three_d::Window {
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

    three_d::Window::from_winit_window(
        winit_window,
        event_loop,
        three_d::SurfaceSettings::default(),
        false,
    )
    .unwrap()
}

struct App {
    gui: three_d::GUI,
    background: BGData<three_d::Texture2DRef>,
    game: Game,
    text_box_text: Vec<String>,
    input_text: String,
}

impl App {
    fn init(context: three_d::Context) -> Self {
        let gui = three_d::GUI::new(&context);

        let background = background::load_raw_backgrounds()
            .unwrap()
            .get(&BackgroundId::blank())
            .unwrap()
            .load(&mut InPlaceLoader(context))
            .unwrap();

        Self {
                gui,
                background,
                game: game_interface::setup_new(),
                text_box_text: Vec::new(),
                input_text: String::new(),
            }
    }

    fn handle_frame(&mut self, frame_input: three_d::FrameInput) -> three_d::FrameOutput {
        let mut events = frame_input.events.clone();
        let camera = three_d::Camera::new_2d(frame_input.viewport);

        if let GameResult::Frame(frame_getter) = self.game.next_result() {
            self.text_box_text.extend(frame_getter.get().as_text());
        }

        self.gui.update(
            &mut events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_context| {
                let accept_input = input_panel(
                    &mut self.input_text,
                    self.game.ready_to_take_input(),
                    egui_context,
                );
                text_box_panel(&self.text_box_text, egui_context);
                if accept_input {
                    let result = self.game.handle_input(&self.input_text);
                    self.input_text.clear();
                    if let Err(messages) = result {
                        self.text_box_text.extend(messages);
                    }
                }
            },
        );
        frame_input
            .screen()
            .clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.))
            .render(
                &camera,
                get_render_objects_for_background(&self.background, &frame_input.context),
                &[],
            )
            .write(|| self.gui.render())
            .unwrap();
        three_d::FrameOutput::default()
    }
}

const INPUT_FONT: egui::FontId = egui::FontId::monospace(15.0);

fn input_panel(input_text: &mut String, enabled: bool, egui_context: &egui::Context) -> bool {
    egui::TopBottomPanel::bottom("input")
        .exact_height(25.)
        .show(egui_context, |ui| {
            let response = ui.add_enabled(
                enabled,
                egui::TextEdit::singleline(input_text)
                    .font(INPUT_FONT)
                    .desired_width(f32::INFINITY)
                    .lock_focus(true),
            );

            response.lost_focus()
                && ui.input(|input_state| input_state.key_pressed(egui::Key::Enter))
        })
        .inner
}

const TEXT_BOX_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(
    (0.2 * 0.6 * 255.) as u8,
    (0.1 * 0.6 * 255.) as u8,
    (0.4 * 0.6 * 255.) as u8,
    (0.6 * 255.) as u8,
);
const TEXT_BOX_MARGIN: f32 = 12.;
const TEXT_BOX_FONT: egui::FontId = egui::FontId::monospace(12.0);

fn text_box_panel<S: Into<String>>(
    lines: impl IntoIterator<Item = S>,
    egui_context: &egui::Context,
) {
    egui::TopBottomPanel::bottom("text_box")
        .frame(egui::Frame {
            inner_margin: egui::Margin::symmetric(TEXT_BOX_MARGIN, 6.),
            fill: TEXT_BOX_COLOR,
            ..Default::default()
        })
        .exact_height(100.)
        .show_separator_line(false)
        .show(egui_context, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    for line in lines {
                        ui.label(
                            egui::RichText::new(line)
                                .font(TEXT_BOX_FONT)
                                .line_height(Some(16.))
                                .color(egui::Color32::WHITE),
                        );
                    }
                });
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
