use aftiktuna::asset::model::{self, Model, TextureLayer};
use aftiktuna::asset::TextureLoader;
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::ModelId;
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::game_interface::{self, Game, GameResult};
use aftiktuna::view::area::{ObjectRenderData, RenderData, RenderProperties};
use aftiktuna::view::Frame;
use background::BackgroundMap;
use std::collections::HashMap;
use three_d::{egui, Object};
use winit::dpi;
use winit::event_loop::EventLoop;
use winit::platform::windows::WindowBuilderExtWindows;
use winit::window::{Icon, WindowBuilder, WindowButtons};

mod background;

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
    assets: Assets,
    game: Game,
    frame: Frame,
    text_box_text: Vec<String>,
    input_text: String,
    camera: Camera,
}

impl App {
    fn init(context: three_d::Context) -> Self {
        let gui = three_d::GUI::new(&context);

        Self {
            gui,
            assets: Assets::load(context),
            game: game_interface::setup_new(),
            frame: Frame::Introduction,
            text_box_text: Vec::new(),
            input_text: String::new(),
            camera: Camera::default(),
        }
    }

    fn handle_frame(&mut self, mut frame_input: three_d::FrameInput) -> three_d::FrameOutput {
        if let GameResult::Frame(frame_getter) = self.game.next_result() {
            self.frame = frame_getter.get();
            self.text_box_text.extend(self.frame.as_text());
        }

        self.gui.update(
            &mut frame_input.events,
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
        self.camera.handle_inputs(&mut frame_input.events);

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        render_frame(
            &self.frame,
            &self.camera,
            &screen,
            &frame_input,
            &mut self.assets,
        );

        screen.write(|| self.gui.render()).unwrap();
        three_d::FrameOutput::default()
    }
}

struct Assets {
    backgrounds: BackgroundMap,
    models: LazilyLoadedModels,
}

impl Assets {
    fn load(context: three_d::Context) -> Self {
        Self {
            backgrounds: BackgroundMap::load(context.clone()),
            models: LazilyLoadedModels::new(context),
        }
    }
}

fn render_frame(
    frame: &Frame,
    camera: &Camera,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
    assets: &mut Assets,
) {

    match frame {
        Frame::Introduction | Frame::LocationChoice(_) | Frame::Error(_) => {
            let background_objects = assets.backgrounds.get_render_objects_for_primary(
                &BackgroundId::location_choice(),
                0,
                0.,
                &frame_input.context,
            );

            let render_camera = default_render_camera(frame_input.viewport);
            screen
                .write::<three_d::RendererError>(|| {
                    for object in background_objects {
                        object.render(&render_camera, &[]);
                    }
                    Ok(())
                })
                .unwrap();
        }
        Frame::AreaView { render_data, .. } => {
            draw_area_view(render_data, camera, screen, frame_input, assets);
        }
        Frame::Dialogue { data, .. } => {
            let background_object = assets
                .backgrounds
                .get_render_object_for_secondary(&data.background, &frame_input.context);

            let render_camera = default_render_camera(frame_input.viewport);
            screen.render(&render_camera, [background_object], &[]);
        }
        Frame::StoreView { view, .. } => {
            let background_object = assets
                .backgrounds
                .get_render_object_for_secondary(&view.background, &frame_input.context);

            let render_camera = default_render_camera(frame_input.viewport);
            screen.render(&render_camera, [background_object], &[]);
        }
        Frame::Ending { stop_type } => {
            let (r, g, b) = match stop_type {
                aftiktuna::StopType::Win => (0.78, 0.78, 0.78),
                aftiktuna::StopType::Lose => (0., 0., 0.),
            };
            screen.clear(three_d::ClearState::color(r, g, b, 1.));
        }
    }
}

fn draw_area_view(
    render_data: &RenderData,
    camera: &Camera,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
    assets: &mut Assets,
) {
    let background_objects = assets.backgrounds.get_render_objects_for_primary(
        &render_data.background,
        render_data.background_offset.unwrap_or(0),
        camera.camera_x,
        &frame_input.context,
    );
    let entity_objects = position_objects(&render_data.objects, &mut assets.models)
        .into_iter()
        .flat_map(|(pos, object)| {
            get_render_objects_for_entity(
                assets.models.lookup_model(&object.model_id),
                pos,
                &object.properties,
                &frame_input.context,
            )
        })
        .collect::<Vec<_>>();

    let render_camera = camera.get_render_camera(frame_input.viewport);
    screen
        .write::<three_d::RendererError>(|| {
            for object in background_objects {
                object.render(&render_camera, &[]);
            }
            for object in entity_objects {
                object.render(&render_camera, &[]);
            }
            Ok(())
        })
        .unwrap();
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

        let path = format!("assets/texture/{name}.png");

        let texture: three_d::CpuTexture = three_d_asset::io::load_and_deserialize(path)?;
        let texture = three_d::Texture2DRef::from_cpu_texture(&self.1, &texture);
        self.0.insert(name, texture.clone());
        Ok(texture)
    }
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

fn get_render_objects_for_entity(
    model: &Model<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    properties: &RenderProperties,
    context: &three_d::Context,
) -> Vec<impl three_d::Object> {
    model
        .layers
        .iter()
        .flat_map(|layer| get_render_object_for_layer(layer, pos, properties, context))
        .collect()
}

fn get_render_object_for_layer(
    layer: &TextureLayer<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    properties: &RenderProperties,
    context: &three_d::Context,
) -> Option<impl three_d::Object> {
    if !layer.conditions.meets_conditions(properties) {
        return None;
    }

    let (width, height) = layer
        .positioning
        .size
        .map(|(width, height)| (f32::from(width), f32::from(height)))
        .unwrap_or_else(|| (layer.texture.width() as f32, layer.texture.height() as f32));
    let direction = if !layer.positioning.fixed && properties.direction == Direction::Left {
        -1.
    } else {
        1.
    };
    let rectangle = three_d::Rectangle::new(
        context,
        three_d::vec2(
            pos.x,
            pos.y + height / 2. - f32::from(layer.positioning.y_offset),
        ),
        three_d::degrees(0.),
        width * direction,
        height,
    );
    let material = three_d::ColorMaterial {
        texture: Some(layer.texture.clone()),
        render_states: three_d::RenderStates {
            write_mask: three_d::WriteMask::COLOR,
            blend: three_d::Blend::STANDARD_TRANSPARENCY,
            ..Default::default()
        },
        ..Default::default()
    };

    Some(three_d::Gm::new(rectangle, material))
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

fn position_objects<'a>(
    objects: &'a Vec<ObjectRenderData>,
    models: &mut LazilyLoadedModels,
) -> Vec<(three_d::Vec2, &'a ObjectRenderData)> {
    let mut positioned_objects = Vec::new();
    let mut positioner = Positioner::new();

    for data in objects {
        let pos = positioner.position_object(
            data.coord,
            models.lookup_model(&data.model_id).is_displacing(),
        );

        positioned_objects.push((pos, data));
    }
    positioned_objects
}

fn position_from_coord(coord: Coord, count: i32) -> three_d::Vec2 {
    three_d::vec2(
        coord_to_center_x(coord) - count as f32 * 15.,
        (190 - count * 15) as f32,
    )
}

#[derive(Default)]
struct Positioner {
    coord_counts: HashMap<Coord, i32>,
}

impl Positioner {
    pub fn new() -> Self {
        Self::default()
    }

    fn position_object(&mut self, coord: Coord, is_displacing: bool) -> three_d::Vec2 {
        if is_displacing {
            let count_ref = self.coord_counts.entry(coord).or_insert(0);
            let count = *count_ref;
            *count_ref = count + 1;
            position_from_coord(coord, count)
        } else {
            position_from_coord(coord, 0)
        }
    }
}

// Coordinates are mapped like this so that when the left edge of the window is 0,
// coord 3 will be placed in the middle of the window.
fn coord_to_center_x(coord: Coord) -> f32 {
    40. + 120. * coord as f32
}
