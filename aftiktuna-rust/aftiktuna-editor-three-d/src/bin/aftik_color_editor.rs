use aftiktuna::asset::color::{self, AftikColorData, AftikColorEntry, RGBColor};
use aftiktuna::asset::model::{self, Model};
use aftiktuna::core::Species;
use aftiktuna::core::display::{AftikColorId, ModelId};
use aftiktuna::core::name::Adjective;
use aftiktuna::view::area::ObjectProperties;
use aftiktuna_three_d::asset::CachedLoader;
use aftiktuna_three_d::render::{self, RenderProperties};
use indexmap::IndexMap;
use std::fs::File;
use std::mem::take;
use three_d::{FrameInput, egui};

const SIZE: (u32, u32) = (800, 600);

type AftikColorMap = IndexMap<AftikColorId, AftikColorEntry>;

fn main() {
    let mut aftik_colors = load_aftik_colors_ordered();

    let window = three_d::Window::new(three_d::WindowSettings {
        title: "Aftiktuna: Aftik Color Editor".to_string(),
        min_size: SIZE,
        max_size: Some(SIZE),
        ..Default::default()
    })
    .unwrap();

    let mut gui = three_d::GUI::new(&window.gl());
    let mut texture_loader = CachedLoader::new(window.gl());
    let aftik_model = model::load_raw_model_from_path(Species::Aftik.model_id().file_path())
        .expect("Unable to load aftik model")
        .load(&mut texture_loader)
        .unwrap();
    let portrait_model = model::load_raw_model_from_path(ModelId::portrait().file_path())
        .expect("Unable to load portrait model")
        .load(&mut texture_loader)
        .unwrap();

    let mut selected_index = 0;
    let mut new_color_name = String::new();

    if aftik_colors.is_empty() {
        init_new_color(
            AftikColorId::new("mint"),
            &mut selected_index,
            &mut aftik_colors,
        );
    }

    window.render_loop(move |mut frame_input| {
        let mut save = false;

        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_context| {
                save = side_panel(
                    egui_context,
                    &mut selected_index,
                    &mut new_color_name,
                    &mut aftik_colors,
                );
            },
        );

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(
            0.78, 0.78, 0.78, 1., 1.,
        ));

        let (_, aftik_color_data) = aftik_colors.get_index(selected_index).unwrap();
        draw_examples(
            aftik_color_data.color_data,
            &aftik_model,
            &portrait_model,
            &frame_input,
        );

        screen.write(|| gui.render()).unwrap();

        if save {
            save_map(&aftik_colors);
            three_d::FrameOutput {
                exit: true,
                ..Default::default()
            }
        } else {
            three_d::FrameOutput::default()
        }
    });
}

fn load_aftik_colors_ordered() -> AftikColorMap {
    let file = File::open(color::AFTIK_COLORS_PATH).expect("Unable to open aftik color file");
    serde_json::from_reader::<_, IndexMap<_, _>>(file).expect("Unable to load aftik color data")
}

fn draw_examples(
    aftik_color_data: AftikColorData,
    aftik_model: &Model<three_d::Texture2DRef>,
    portrait_model: &Model<three_d::Texture2DRef>,
    frame_input: &FrameInput,
) {
    let objects = vec![
        (
            aftik_model,
            three_d::vec2(100., 440.),
            ObjectProperties {
                is_alive: true,
                is_badly_hurt: false,
                ..Default::default()
            },
        ),
        (
            aftik_model,
            three_d::vec2(250., 440.),
            ObjectProperties {
                is_alive: true,
                is_badly_hurt: true,
                ..Default::default()
            },
        ),
        (
            aftik_model,
            three_d::vec2(400., 450.),
            ObjectProperties {
                is_alive: false,
                ..Default::default()
            },
        ),
        (
            portrait_model,
            three_d::vec2(150., 0.),
            ObjectProperties {
                is_badly_hurt: false,
                ..Default::default()
            },
        ),
        (
            portrait_model,
            three_d::vec2(450., 0.),
            ObjectProperties {
                is_badly_hurt: true,
                ..Default::default()
            },
        ),
    ];
    let objects = objects
        .into_iter()
        .flat_map(|(model, pos, properties)| {
            render::get_render_objects_for_entity_with_color(
                model,
                pos,
                RenderProperties {
                    object: &properties,
                    aftik_color: aftik_color_data,
                },
                frame_input.accumulated_time as f32,
                &frame_input.context,
            )
        })
        .collect::<Vec<_>>();
    render::draw_in_order(
        &objects,
        &render::default_render_camera(frame_input.viewport),
        &frame_input.screen(),
    );
}

fn side_panel(
    ctx: &egui::Context,
    selected_index: &mut usize,
    new_color_name: &mut String,
    aftik_colors: &mut AftikColorMap,
) -> bool {
    egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(200.)
        .show(ctx, |ui| {
            ui.text_edit_singleline(new_color_name);

            if ui.button("Add").clicked() && !new_color_name.is_empty() {
                init_new_color(
                    AftikColorId(take(new_color_name)),
                    selected_index,
                    aftik_colors,
                );
            }

            egui::ComboBox::from_id_salt("color_id").show_index(
                ui,
                selected_index,
                aftik_colors.len(),
                |index| {
                    let (AftikColorId(name), _) = aftik_colors.get_index(index).unwrap();
                    name.to_owned()
                },
            );

            ui.separator();

            let (_, aftik_color_data) = aftik_colors.get_index_mut(*selected_index).unwrap();

            ui.label("Primary color:");
            color_picker(ui, &mut aftik_color_data.color_data.primary_color);

            ui.separator();

            ui.label("Secondary color:");
            color_picker(ui, &mut aftik_color_data.color_data.secondary_color);

            ui.separator();

            ui.button("Save").clicked()
        })
        .inner
}

fn init_new_color(
    new_id: AftikColorId,
    selected_index: &mut usize,
    aftik_colors: &mut AftikColorMap,
) {
    if !aftik_colors.contains_key(&new_id) {
        let adjective = Adjective(new_id.0.clone());
        *selected_index = aftik_colors
            .insert_full(
                new_id,
                AftikColorEntry {
                    adjective,
                    color_data: color::DEFAULT_COLOR,
                },
            )
            .0;
    }
}

fn color_picker(ui: &mut egui::Ui, color: &mut RGBColor) {
    let mut color32 = egui::Color32::from_rgb(color.r, color.g, color.b);
    egui::color_picker::color_picker_color32(ui, &mut color32, egui::color_picker::Alpha::Opaque);
    *color = RGBColor::new(color32.r(), color32.g(), color32.b());
}

fn save_map(aftik_colors: &AftikColorMap) {
    let file = File::create(color::AFTIK_COLORS_PATH).unwrap();
    serde_json_pretty::to_writer(file, aftik_colors).unwrap();
}
