use aftiktuna::asset::color::{RGBColor, SpeciesColorData, SpeciesColorEntry};
use aftiktuna::asset::model::{self, Model};
use aftiktuna::core::SpeciesId;
use aftiktuna::core::display::{
    CreatureVariant, CreatureVariantSet, DialogueExpression, SpeciesColorId,
};
use aftiktuna::view::area::ObjectProperties;
use aftiktuna_three_d::asset::CachedLoader;
use aftiktuna_three_d::render::{self, RenderProperties};
use indexmap::IndexMap;
use std::fs::{self, File};
use std::mem::take;
use three_d::{FrameInput, egui};

const SIZE: (u32, u32) = (800, 600);

type SpeciesColorMap = IndexMap<SpeciesColorId, SpeciesColorEntry>;

fn main() {
    let locations_directory = fs::canonicalize("./assets/species_color").unwrap();
    let path = rfd::FileDialog::new()
        .set_title("Pick a location file")
        .add_filter("JSON", &["json"])
        .set_directory(locations_directory)
        .pick_file();
    let Some(path) = path else {
        return;
    };
    let [species_name, "json"] = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split('.')
        .collect::<Vec<_>>()[..]
    else {
        panic!("Unexpected file name")
    };
    let species_id = SpeciesId::from(species_name);
    let example_setup = match species_name {
        "aftik" => ExampleSetup::Aftik,
        "pagepoh" => ExampleSetup::Pagepoh,
        "scarvie" => ExampleSetup::Scarvie,
        _ => panic!("No default for example setup"),
    };

    let file = File::open(&path).expect("Unable to open color data file");
    let mut color_map =
        serde_json::from_reader::<_, IndexMap<_, _>>(file).expect("Unable to load color data");
    assert!(!color_map.is_empty());

    let window = three_d::Window::new(three_d::WindowSettings {
        title: format!("Aftiktuna Color Editor: {species_name}"),
        min_size: SIZE,
        max_size: Some(SIZE),
        ..Default::default()
    })
    .unwrap();

    let mut gui = three_d::GUI::new(&window.gl());
    let mut texture_loader = CachedLoader::new(window.gl());
    let model = model::load_raw_model_from_path(species_id.model_id().file_path())
        .expect("Unable to load model")
        .load(&mut texture_loader)
        .unwrap();
    let portrait_model =
        model::load_raw_model_from_path(species_id.portrait_model_id().file_path())
            .ok()
            .and_then(|model| model.load(&mut texture_loader).ok());

    let mut selected_index = 0;
    let mut new_color_name = String::new();

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
                    &mut color_map,
                );
            },
        );

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(
            0.78, 0.78, 0.78, 1., 1.,
        ));

        let (_, entry) = color_map.get_index(selected_index).unwrap();
        draw_examples(
            entry.color_data,
            example_setup,
            &model,
            portrait_model.as_ref(),
            &frame_input,
        );

        screen.write(|| gui.render()).unwrap();

        if save {
            let file = File::create(&path).unwrap();
            serde_json_pretty::to_writer(file, &color_map).unwrap();
            three_d::FrameOutput {
                exit: true,
                ..Default::default()
            }
        } else {
            three_d::FrameOutput::default()
        }
    });
}

#[derive(Debug, Clone, Copy)]
enum ExampleSetup {
    Aftik,
    Pagepoh,
    Scarvie,
}

fn draw_examples(
    species_color: SpeciesColorData,
    example_setup: ExampleSetup,
    model: &Model<three_d::Texture2DRef>,
    portrait_model: Option<&Model<three_d::Texture2DRef>>,
    frame_input: &FrameInput,
) {
    let objects = match example_setup {
        ExampleSetup::Aftik => vec![
            (
                model,
                three_d::vec2(100., 440.),
                ObjectProperties {
                    is_alive: true,
                    is_badly_hurt: false,
                    ..Default::default()
                },
            ),
            (
                model,
                three_d::vec2(250., 440.),
                ObjectProperties {
                    is_alive: true,
                    is_badly_hurt: true,
                    ..Default::default()
                },
            ),
            (
                model,
                three_d::vec2(400., 450.),
                ObjectProperties {
                    is_alive: false,
                    ..Default::default()
                },
            ),
            (
                portrait_model.unwrap(),
                three_d::vec2(150., 0.),
                ObjectProperties {
                    is_badly_hurt: false,
                    ..Default::default()
                },
            ),
            (
                portrait_model.unwrap(),
                three_d::vec2(450., 0.),
                ObjectProperties {
                    is_badly_hurt: true,
                    ..Default::default()
                },
            ),
        ],
        ExampleSetup::Pagepoh => vec![
            (
                model,
                three_d::vec2(100., 440.),
                ObjectProperties {
                    is_alive: true,
                    is_badly_hurt: false,
                    ..Default::default()
                },
            ),
            (
                model,
                three_d::vec2(250., 440.),
                ObjectProperties {
                    is_alive: true,
                    is_badly_hurt: true,
                    expression: DialogueExpression::Sad,
                    ..Default::default()
                },
            ),
            (
                model,
                three_d::vec2(400., 450.),
                ObjectProperties {
                    is_alive: false,
                    ..Default::default()
                },
            ),
            (
                portrait_model.unwrap(),
                three_d::vec2(150., 0.),
                ObjectProperties {
                    is_badly_hurt: false,
                    ..Default::default()
                },
            ),
            (
                portrait_model.unwrap(),
                three_d::vec2(450., 0.),
                ObjectProperties {
                    is_badly_hurt: true,
                    expression: DialogueExpression::Sad,
                    ..Default::default()
                },
            ),
        ],
        ExampleSetup::Scarvie => vec![
            (
                model,
                three_d::vec2(100., 440.),
                ObjectProperties {
                    is_alive: true,
                    is_badly_hurt: false,
                    creature_variant_set: CreatureVariantSet::from([CreatureVariant::female()]),
                    ..Default::default()
                },
            ),
            (
                model,
                three_d::vec2(250., 440.),
                ObjectProperties {
                    is_alive: true,
                    is_badly_hurt: true,
                    creature_variant_set: CreatureVariantSet::from([CreatureVariant::female()]),
                    ..Default::default()
                },
            ),
            (
                model,
                three_d::vec2(400., 450.),
                ObjectProperties {
                    is_alive: false,
                    creature_variant_set: CreatureVariantSet::from([CreatureVariant::female()]),
                    ..Default::default()
                },
            ),
            (
                model,
                three_d::vec2(100., 240.),
                ObjectProperties {
                    is_alive: true,
                    is_badly_hurt: false,
                    creature_variant_set: CreatureVariantSet::from([CreatureVariant::male()]),
                    ..Default::default()
                },
            ),
            (
                model,
                three_d::vec2(250., 240.),
                ObjectProperties {
                    is_alive: true,
                    is_badly_hurt: true,
                    creature_variant_set: CreatureVariantSet::from([CreatureVariant::male()]),
                    ..Default::default()
                },
            ),
            (
                model,
                three_d::vec2(400., 250.),
                ObjectProperties {
                    is_alive: false,
                    creature_variant_set: CreatureVariantSet::from([CreatureVariant::male()]),
                    ..Default::default()
                },
            ),
        ],
    };
    let objects = objects
        .into_iter()
        .flat_map(|(model, pos, properties)| {
            render::get_render_objects_for_entity_with_color(
                model,
                pos,
                RenderProperties {
                    object: &properties,
                    species_color,
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
    species_colors: &mut SpeciesColorMap,
) -> bool {
    egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(200.)
        .show(ctx, |ui| {
            ui.text_edit_singleline(new_color_name);

            if ui.button("Add").clicked() && !new_color_name.is_empty() {
                init_new_color(
                    SpeciesColorId(take(new_color_name)),
                    selected_index,
                    species_colors,
                );
            }

            egui::ComboBox::from_id_salt("color_id").show_index(
                ui,
                selected_index,
                species_colors.len(),
                |index| {
                    let (SpeciesColorId(name), _) = species_colors.get_index(index).unwrap();
                    name.to_owned()
                },
            );

            ui.separator();

            let (_, entry) = species_colors.get_index_mut(*selected_index).unwrap();

            ui.label("Primary color:");
            color_picker(ui, &mut entry.color_data.primary_color);

            ui.separator();

            ui.label("Secondary color:");
            color_picker(ui, &mut entry.color_data.secondary_color);

            ui.separator();

            ui.button("Save").clicked()
        })
        .inner
}

fn init_new_color(
    new_id: SpeciesColorId,
    selected_index: &mut usize,
    species_colors: &mut SpeciesColorMap,
) {
    if !species_colors.contains_key(&new_id) {
        *selected_index = species_colors
            .insert_full(new_id, SpeciesColorEntry::default())
            .0;
    }
}

fn color_picker(ui: &mut egui::Ui, color: &mut RGBColor) {
    let mut color32 = egui::Color32::from_rgb(color.r, color.g, color.b);
    egui::color_picker::color_picker_color32(ui, &mut color32, egui::color_picker::Alpha::Opaque);
    *color = RGBColor::new(color32.r(), color32.g(), color32.b());
}
