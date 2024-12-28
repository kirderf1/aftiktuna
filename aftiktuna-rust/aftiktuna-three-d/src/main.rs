use aftiktuna::asset::background::BGData;
use aftiktuna::asset::{background, TextureLoader};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::game_interface::{self, Game, GameResult};
use aftiktuna::view::Frame;
use std::collections::HashMap;
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
    backgrounds: HashMap<BackgroundId, BGData<three_d::Texture2DRef>>,
    game: Game,
    frame: Frame,
    text_box_text: Vec<String>,
    input_text: String,
}

impl App {
    fn init(context: three_d::Context) -> Self {
        let gui = three_d::GUI::new(&context);

        let mut texture_loader = CachedLoader(HashMap::new(), context);
        let background_data = background::load_raw_backgrounds().unwrap();
        let backgrounds = background_data
            .into_iter()
            .map(|(id, data)| (id, data.load(&mut texture_loader).unwrap()))
            .collect();

        Self {
            gui,
            backgrounds,
            game: game_interface::setup_new(),
            frame: Frame::Introduction,
            text_box_text: Vec::new(),
            input_text: String::new(),
        }
    }

    fn handle_frame(&mut self, frame_input: three_d::FrameInput) -> three_d::FrameOutput {
        let mut events = frame_input.events.clone();
        let screen = frame_input.screen();

        if let GameResult::Frame(frame_getter) = self.game.next_result() {
            self.frame = frame_getter.get();
            self.text_box_text.extend(self.frame.as_text());
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
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        self.render_frame(&screen, frame_input.viewport, &frame_input.context);

        screen.write(|| self.gui.render()).unwrap();
        three_d::FrameOutput::default()
    }

    fn render_frame(
        &self,
        screen: &three_d::RenderTarget<'_>,
        viewport: three_d::Viewport,
        context: &three_d::Context,
    ) {
        let camera = three_d::Camera::new_2d(viewport);
        match &self.frame {
            Frame::Introduction | Frame::LocationChoice(_) | Frame::Error(_) => {
                let background_objects = get_render_objects_for_background(
                    get_background_or_default(&BackgroundId::location_choice(), &self.backgrounds),
                    context,
                );
                screen.render(&camera, background_objects, &[]);
            }
            Frame::AreaView { render_data, .. } => {
                let background_objects = get_render_objects_for_background(
                    get_background_or_default(&render_data.background, &self.backgrounds),
                    context,
                );
                screen.render(&camera, background_objects, &[]);
            }
            Frame::Dialogue { data, .. } => {
                let background_object = get_render_object_for_secondary_background(
                    get_background_or_default(&data.background, &self.backgrounds),
                    context,
                );
                screen.render(&camera, [background_object], &[]);
            }
            Frame::StoreView { view, .. } => {
                let background_object = get_render_object_for_secondary_background(
                    get_background_or_default(&view.background, &self.backgrounds),
                    context,
                );
                screen.render(&camera, [background_object], &[]);
            }
            Frame::Ending { stop_type } => {
                let color = match stop_type {
                    aftiktuna::StopType::Win => three_d::Srgba::new_opaque(199, 199, 199),
                    aftiktuna::StopType::Lose => three_d::Srgba::BLACK,
                };
                let background_object = three_d::Gm::new(
                    three_d::Rectangle::new(
                        context,
                        three_d::vec2(400., 300.),
                        three_d::degrees(0.),
                        800.,
                        600.,
                    ),
                    three_d::ColorMaterial {
                        color,
                        ..Default::default()
                    },
                );
                screen.render(&camera, [background_object], &[]);
            }
        }
    }
}

fn get_background_or_default<'a>(
    id: &BackgroundId,
    backgrounds: &'a HashMap<BackgroundId, BGData<three_d::Texture2DRef>>,
) -> &'a BGData<three_d::Texture2DRef> {
    backgrounds
        .get(id)
        .or_else(|| backgrounds.get(&BackgroundId::blank()))
        .expect("Missing blank texture")
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

struct CachedLoader(HashMap<String, three_d::Texture2DRef>, three_d::Context);

impl TextureLoader<three_d::Texture2DRef, three_d_asset::Error> for CachedLoader {
    fn load_texture(
        &mut self,
        name: String,
    ) -> Result<three_d::Texture2DRef, three_d_asset::Error> {
        if let Some(texture) = self.0.get(&name) {
            return Ok(texture.clone());
        }

        let path = format!("assets/texture/{name}.png");

        let texture: three_d::CpuTexture = three_d_asset::io::load_and_deserialize(path)?;
        let texture = three_d::Texture2DRef::from_cpu_texture(&self.1, &texture);
        self.0.insert(name, texture.clone());
        Ok(texture)
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
                    texture: Some(layer.texture.clone()),
                    render_states: three_d::RenderStates {
                        depth_test: three_d::DepthTest::Always,
                        blend: three_d::Blend::STANDARD_TRANSPARENCY,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
        })
        .collect()
}

fn get_render_object_for_secondary_background(
    background: &BGData<three_d::Texture2DRef>,
    context: &three_d::Context,
) -> impl three_d::Object {
    let material = match &background.portrait {
        &background::PortraitBGData::Color(color) => three_d::ColorMaterial {
            color: color.into(),
            ..Default::default()
        },
        background::PortraitBGData::Texture(texture) => three_d::ColorMaterial {
            texture: Some(texture.clone()),
            ..Default::default()
        },
    };
    three_d::Gm::new(
        three_d::Rectangle::new(
            context,
            three_d::vec2(400., 300.),
            three_d::degrees(0.),
            800.,
            600.,
        ),
        material,
    )
}
