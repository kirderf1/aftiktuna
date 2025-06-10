use aftiktuna::asset::background;
use aftiktuna::core::area::BackgroundId;
use aftiktuna::location::{AreaData, LocationData};
use aftiktuna_three_d::{asset, render};
use std::fs::{self, File};
use three_d::egui;

const SIDE_PANEL_WIDTH: u32 = 200;

const SIZE: (u32, u32) = (
    aftiktuna_three_d::WINDOW_WIDTH as u32 + SIDE_PANEL_WIDTH,
    aftiktuna_three_d::WINDOW_HEIGHT as u32,
);

fn main() {
    let locations_directory = fs::canonicalize("./assets/location").unwrap();
    let path = rfd::FileDialog::new()
        .set_title("Pick a location file")
        .add_filter("JSON", &["json"])
        .set_directory(locations_directory)
        .pick_file();
    let Some(path) = path else {
        return;
    };

    let mut location_data =
        serde_json::from_reader::<_, LocationData>(File::open(path).unwrap()).unwrap();
    let mut area_index = 0;

    let window = three_d::Window::new(three_d::WindowSettings {
        title: "Aftiktuna: Location Editor".to_string(),
        min_size: SIZE,
        max_size: Some(SIZE),
        ..Default::default()
    })
    .unwrap();

    let background_types = background::load_index_map_backgrounds()
        .unwrap()
        .into_keys()
        .collect::<Vec<_>>();
    let background_map = asset::BackgroundMap::load(window.gl()).unwrap();
    let mut camera = aftiktuna_three_d::Camera::default();
    let mut gui = three_d::GUI::new(&window.gl());

    window.render_loop(move |mut frame_input| {
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_context| {
                side_panel(egui_context, |ui| {
                    egui::ComboBox::from_id_salt("area").show_index(
                        ui,
                        &mut area_index,
                        location_data.areas.len(),
                        |index| location_data.areas[index].name.clone(),
                    );
                    ui.separator();
                    area_editor_ui(ui, &mut location_data.areas[area_index], &background_types);
                });
            },
        );

        let area = &location_data.areas[area_index];
        camera.handle_inputs(&mut frame_input.events);
        camera.clamp(area.objects.len());

        let backgorund_data = background_map.get_or_default(&area.background);
        let background = render::render_objects_for_primary_background(
            backgorund_data,
            area.background_offset.unwrap_or(0),
            camera.camera_x,
            &frame_input.context,
        );

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));
        render::draw_in_order(
            &background,
            &render::get_render_camera(&camera, frame_input.viewport),
            &screen,
        );

        screen.write(|| gui.render()).unwrap();

        three_d::FrameOutput::default()
    });
}

fn side_panel(egui_context: &egui::Context, panel_contents: impl FnOnce(&mut egui::Ui)) {
    egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&egui_context.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(SIDE_PANEL_WIDTH as f32)
        .show(egui_context, panel_contents);
}

fn area_editor_ui(ui: &mut egui::Ui, area: &mut AreaData, background_types: &[BackgroundId]) {
    ui.label("Background:");
    egui::ComboBox::from_id_salt("background")
        .selected_text(&area.background.0)
        .show_ui(ui, |ui| {
            for background_id in background_types {
                if ui
                    .selectable_label(background_id == &area.background, &background_id.0)
                    .clicked()
                {
                    area.background = background_id.clone();
                }
            }
        });
    ui.label("Background offset:");
    let mut has_offset = area.background_offset.is_some();
    ui.add(egui::Checkbox::without_text(&mut has_offset));
    if has_offset && area.background_offset.is_none() {
        area.background_offset = Some(0);
    }
    if !has_offset && area.background_offset.is_some() {
        area.background_offset = None;
    }
    if let Some(offset) = &mut area.background_offset {
        ui.add(egui::Slider::new(offset, 0..=20));
    }
}
