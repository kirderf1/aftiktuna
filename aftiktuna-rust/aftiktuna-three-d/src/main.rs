use aftiktuna::asset::model::{self, Model};
use aftiktuna::asset::TextureLoader;
use aftiktuna::core::display::ModelId;
use aftiktuna::game_interface::{self, Game, GameResult};
use aftiktuna::view::Frame;
use background::BackgroundMap;
use std::collections::HashMap;
use three_d::egui;
use winit::dpi;
use winit::event_loop::EventLoop;
use winit::platform::windows::WindowBuilderExtWindows;
use winit::window::{Icon, WindowBuilder, WindowButtons};

mod background;
mod render;

pub const WINDOW_WIDTH: u16 = 800;
pub const WINDOW_HEIGHT: u16 = 600;
pub const WINDOW_WIDTH_F: f32 = WINDOW_WIDTH as f32;
pub const WINDOW_HEIGHT_F: f32 = WINDOW_HEIGHT as f32;

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
        .with_inner_size(dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
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
    assets: Assets,
    game: Game,
    frame: Frame,
    text_box_text: Vec<String>,
    input_text: String,
    request_input_focus: bool,
    camera: Camera,
}

impl App {
    fn init(context: three_d::Context) -> Self {
        let gui = three_d::GUI::new(&context);

        let mut app = Self {
            gui,
            assets: Assets::load(context),
            game: game_interface::setup_new(),
            frame: Frame::Introduction,
            text_box_text: Vec::new(),
            input_text: String::new(),
            request_input_focus: false,
            camera: Camera::default(),
        };
        app.try_get_next_frame();
        app
    }

    fn handle_frame(&mut self, mut frame_input: three_d::FrameInput) -> three_d::FrameOutput {
        let mut clicked_text_box = false;
        let mut accept_input = false;
        self.gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_context| {
                accept_input = input_panel(
                    &mut self.input_text,
                    self.game.ready_to_take_input(),
                    std::mem::take(&mut self.request_input_focus),
                    egui_context,
                );
                clicked_text_box = text_box_panel(&self.text_box_text, egui_context);
            },
        );

        if clicked_text_box {
            self.try_get_next_frame();
        }
        if accept_input {
            let result = self.game.handle_input(&self.input_text);
            self.input_text.clear();

            match result {
                Ok(()) => self.try_get_next_frame(),
                Err(messages) => {
                    self.text_box_text.extend(messages);
                    self.request_input_focus = true;
                }
            }
        }

        self.camera.handle_inputs(&mut frame_input.events);

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        render::render_frame(
            &self.frame,
            &self.camera,
            &screen,
            &frame_input,
            &mut self.assets,
        );

        screen.write(|| self.gui.render()).unwrap();
        if self.game.next_result().has_frame() {
            draw_frame_click_icon(&self.assets.left_mouse_icon, screen, &frame_input);
        }
        three_d::FrameOutput::default()
    }

    fn try_get_next_frame(&mut self) {
        if let GameResult::Frame(frame_getter) = self.game.next_result() {
            self.frame = frame_getter.get();
            self.text_box_text = self.frame.as_text();
            self.request_input_focus = self.game.ready_to_take_input();
        }
    }
}

struct Assets {
    backgrounds: BackgroundMap,
    models: LazilyLoadedModels,
    left_mouse_icon: three_d::Texture2DRef,
}

impl Assets {
    fn load(context: three_d::Context) -> Self {
        let left_mouse_icon =
            load_texture("left_mouse", &context).expect("Missing left_mouse.png texture");
        Self {
            backgrounds: BackgroundMap::load(context.clone()),
            models: LazilyLoadedModels::new(context),
            left_mouse_icon,
        }
    }
}

const INPUT_PANEL_HEIGHT: f32 = 25.;
const INPUT_FONT: egui::FontId = egui::FontId::monospace(15.0);

fn input_panel(
    input_text: &mut String,
    enabled: bool,
    request_focus: bool,
    egui_context: &egui::Context,
) -> bool {
    egui::TopBottomPanel::bottom("input")
        .exact_height(INPUT_PANEL_HEIGHT)
        .show(egui_context, |ui| {
            let response = ui.add_enabled(
                enabled,
                egui::TextEdit::singleline(input_text)
                    .font(INPUT_FONT)
                    .desired_width(f32::INFINITY)
                    .lock_focus(true),
            );

            if request_focus {
                response.request_focus();
            }

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
const TEXT_PANEL_HEIGHT: f32 = 100.;
const TEXT_BOX_MARGIN: f32 = 12.;
const TEXT_BOX_FONT: egui::FontId = egui::FontId::monospace(12.0);

fn text_box_panel<S: Into<String>>(
    lines: impl IntoIterator<Item = S>,
    egui_context: &egui::Context,
) -> bool {
    let response = egui::TopBottomPanel::bottom("text_box")
        .frame(egui::Frame {
            inner_margin: egui::Margin::symmetric(TEXT_BOX_MARGIN, 6.),
            fill: TEXT_BOX_COLOR,
            ..Default::default()
        })
        .exact_height(TEXT_PANEL_HEIGHT)
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
        })
        .response;
    response.interact(egui::Sense::click()).clicked()
}

const CLICK_ICON_OFFSET: f32 = 5.;

fn draw_frame_click_icon(
    icon: &three_d::Texture2DRef,
    screen: three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
) {
    let alpha = ((frame_input.accumulated_time / 1000. * 3.).sin() + 1.) / 2.;
    let width = icon.width() as f32;
    let height = icon.height() as f32;
    let icon = three_d::Gm::new(
        three_d::Rectangle::new(
            &frame_input.context,
            three_d::vec2(
                WINDOW_WIDTH_F - CLICK_ICON_OFFSET - width / 2.,
                INPUT_PANEL_HEIGHT + TEXT_PANEL_HEIGHT - CLICK_ICON_OFFSET - height / 2.,
            ),
            three_d::degrees(0.),
            width,
            height,
        ),
        three_d::ColorMaterial {
            color: three_d::Srgba::new(255, 255, 255, (alpha * 255.).round() as u8),
            texture: Some(icon.clone()),
            render_states: three_d::RenderStates {
                write_mask: three_d::WriteMask::COLOR,
                blend: three_d::Blend::STANDARD_TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    screen.render(default_render_camera(frame_input.viewport), [icon], &[]);
}

struct CachedLoader(HashMap<String, three_d::Texture2DRef>, three_d::Context);

impl CachedLoader {
    fn new(context: three_d::Context) -> Self {
        Self(HashMap::new(), context)
    }
}

impl TextureLoader<three_d::Texture2DRef, three_d_asset::Error> for CachedLoader {
    fn load_texture(
        &mut self,
        name: String,
    ) -> Result<three_d::Texture2DRef, three_d_asset::Error> {
        if let Some(texture) = self.0.get(&name) {
            return Ok(texture.clone());
        }

        let texture = load_texture(&name, &self.1)?;
        self.0.insert(name, texture.clone());
        Ok(texture)
    }
}

fn load_texture(
    name: &str,
    context: &three_d::Context,
) -> Result<three_d::Texture2DRef, three_d_asset::Error> {
    let path = format!("assets/texture/{name}.png");

    let texture: three_d::CpuTexture = three_d_asset::io::load_and_deserialize(path)?;
    Ok(three_d::Texture2DRef::from_cpu_texture(context, &texture))
}

struct LazilyLoadedModels {
    texture_loader: CachedLoader,
    loaded_models: HashMap<ModelId, Model<three_d::Texture2DRef>>,
}

impl LazilyLoadedModels {
    fn new(context: three_d::Context) -> Self {
        Self {
            texture_loader: CachedLoader::new(context),
            loaded_models: HashMap::new(),
        }
    }

    fn lookup_model(&mut self, model_id: &ModelId) -> &Model<three_d::Texture2DRef> {
        if !self.loaded_models.contains_key(model_id) {
            let model = model::load_raw_model_from_path(model_id.file_path())
                .unwrap()
                .load(&mut self.texture_loader)
                .unwrap();
            self.loaded_models.insert(model_id.clone(), model);
        }
        self.loaded_models.get(model_id).unwrap()
    }
}

#[derive(Default)]
struct Camera {
    camera_x: f32,
    is_dragging: bool,
}

impl Camera {
    fn get_render_camera(&self, viewport: three_d::Viewport) -> three_d::Camera {
        let mut render_camera = three_d::Camera::new_orthographic(
            viewport,
            three_d::vec3(
                self.camera_x + viewport.width as f32 * 0.5,
                viewport.height as f32 * 0.5,
                1.0,
            ),
            three_d::vec3(
                self.camera_x + viewport.width as f32 * 0.5,
                viewport.height as f32 * 0.5,
                0.0,
            ),
            three_d::vec3(0.0, 1.0, 0.0),
            viewport.height as f32,
            0.0,
            10.0,
        );
        render_camera.disable_tone_and_color_mapping();
        render_camera
    }

    fn handle_inputs(&mut self, events: &mut [three_d::Event]) {
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
}

fn default_render_camera(viewport: three_d::Viewport) -> three_d::Camera {
    let mut render_camera = three_d::Camera::new_2d(viewport);
    render_camera.disable_tone_and_color_mapping();
    render_camera
}
