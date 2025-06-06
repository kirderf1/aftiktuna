use super::camera::Camera;
use crate::asset::Assets;
use aftiktuna::asset::background::{BGData, PortraitBGData};
use aftiktuna::asset::color::AftikColorData;
use aftiktuna::asset::model::{Model, ModelAccess, TextureLayer};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::{AftikColorId, ModelId};
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::core::store::StoreStock;
use aftiktuna::view::area::{ObjectRenderData, RenderData, RenderProperties};
use aftiktuna::view::Frame;
use aftiktuna::{asset, view};
use std::collections::HashMap;
use three_d::Object;

pub fn render_frame(
    frame: &Frame,
    cached_objects: &[(three_d::Vec2, ObjectRenderData)],
    camera: &Camera,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
    assets: &mut Assets,
) {
    match frame {
        Frame::Introduction | Frame::LocationChoice(_) | Frame::Error(_) => {
            let background_objects = render_objects_for_primary_background(
                assets
                    .backgrounds
                    .get_or_default(&BackgroundId::location_choice()),
                0,
                0.,
                &frame_input.context,
            );

            let render_camera = crate::default_render_camera(frame_input.viewport);
            draw_in_order(&background_objects, &render_camera, screen);
        }
        Frame::AreaView { render_data, .. } => {
            draw_area_view(
                render_data,
                cached_objects,
                camera,
                screen,
                frame_input,
                assets,
            );
        }
        Frame::Dialogue { data, .. } => {
            draw_secondary_background(
                assets.backgrounds.get_or_default(&data.background),
                screen,
                frame_input,
            );

            let x = match data.direction {
                Direction::Left => crate::WINDOW_WIDTH_F - 300.,
                Direction::Right => 300.,
            };
            let objects = get_render_objects_for_entity(
                assets.models.lookup_model(&ModelId::portrait()),
                three_d::vec2(x, 0.),
                &RenderProperties {
                    direction: data.direction,
                    aftik_color: data.color.clone(),
                    is_badly_hurt: data.is_badly_hurt,
                    ..RenderProperties::default()
                },
                &mut assets.aftik_colors,
                &frame_input.context,
            );
            let render_camera = crate::default_render_camera(frame_input.viewport);
            draw_in_order(&objects, &render_camera, screen);
        }
        Frame::StoreView { view, .. } => {
            draw_store_view(view, screen, frame_input, assets);
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
    cached_objects: &[(three_d::Vec2, ObjectRenderData)],
    camera: &Camera,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
    assets: &mut Assets,
) {
    let background_objects = render_objects_for_primary_background(
        assets.backgrounds.get_or_default(&render_data.background),
        render_data.background_offset.unwrap_or(0),
        camera.camera_x,
        &frame_input.context,
    );
    let entity_objects = cached_objects
        .iter()
        .flat_map(|(pos, object)| {
            let mut render_objects = get_render_objects_for_entity(
                assets.models.lookup_model(&object.model_id),
                *pos,
                &object.properties,
                &mut assets.aftik_colors,
                &frame_input.context,
            );
            if object.properties.is_alive {
                if let Some(item_model_id) = &object.wielded_item {
                    let item_model = assets.models.lookup_model(item_model_id);
                    let direction_mod = match object.properties.direction {
                        Direction::Left => -1,
                        Direction::Right => 1,
                    };
                    let offset = three_d::vec2(
                        f32::from(direction_mod * item_model.wield_offset.0),
                        f32::from(-item_model.wield_offset.1),
                    );
                    render_objects.extend(get_render_objects_for_entity(
                        item_model,
                        pos + offset,
                        &RenderProperties {
                            direction: object.properties.direction,
                            ..RenderProperties::default()
                        },
                        &mut assets.aftik_colors,
                        &frame_input.context,
                    ));
                }
            }
            render_objects
        })
        .collect::<Vec<_>>();

    let render_camera = super::get_render_camera(camera, frame_input.viewport);
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

    draw_camera_arrows(
        camera.has_space_to_drag(render_data.area_size),
        &assets.side_arrow_texture,
        screen,
        frame_input,
    );
}

fn draw_camera_arrows(
    [left_drag, right_drag]: [bool; 2],
    arrow_texture: &three_d::Texture2DRef,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
) {
    if !left_drag && !right_drag {
        return;
    }

    let alpha = ((frame_input.accumulated_time / 1000. * 3.).sin() + 1.) / 2.;
    let arrow_material = three_d::ColorMaterial {
        color: three_d::Srgba::new(255, 255, 255, (alpha * 255.).round() as u8),
        texture: Some(arrow_texture.clone()),
        render_states: three_d::RenderStates {
            write_mask: three_d::WriteMask::COLOR,
            blend: three_d::Blend::STANDARD_TRANSPARENCY,
            ..Default::default()
        },
        ..Default::default()
    };
    let texture_width = arrow_texture.width() as f32;
    let texture_height = arrow_texture.height() as f32;
    let mut arrow_objects = vec![];
    if left_drag {
        arrow_objects.push(three_d::Gm::new(
            three_d::Rectangle::new(
                &frame_input.context,
                three_d::vec2(10. + texture_width / 2., 250.),
                three_d::degrees(0.),
                -texture_width,
                texture_height,
            ),
            arrow_material.clone(),
        ));
    }
    if right_drag {
        arrow_objects.push(three_d::Gm::new(
            three_d::Rectangle::new(
                &frame_input.context,
                three_d::vec2(crate::WINDOW_WIDTH_F - 10. - texture_width / 2., 250.),
                three_d::degrees(0.),
                texture_width,
                texture_height,
            ),
            arrow_material,
        ));
    }

    let default_camera = crate::default_render_camera(frame_input.viewport);
    screen
        .write::<three_d::RendererError>(|| {
            for object in arrow_objects {
                object.render(&default_camera, &[]);
            }
            Ok(())
        })
        .unwrap();
}

fn draw_store_view(
    view: &view::StoreView,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
    assets: &mut Assets,
) {
    draw_secondary_background(
        assets.backgrounds.get_or_default(&view.background),
        screen,
        frame_input,
    );

    let objects = get_render_objects_for_entity(
        assets.models.lookup_model(&ModelId::portrait()),
        three_d::vec2(crate::WINDOW_WIDTH_F - 200., 0.),
        &RenderProperties {
            direction: Direction::Left,
            aftik_color: view.shopkeeper_color.clone(),
            ..RenderProperties::default()
        },
        &mut assets.aftik_colors,
        &frame_input.context,
    );
    let render_camera = crate::default_render_camera(frame_input.viewport);
    draw_in_order(&objects, &render_camera, screen);
    const STOCK_PANEL_COLOR: three_d::Vec4 = three_d::vec4(0.2, 0.1, 0.4, 0.6);
    screen.render(
        &render_camera,
        [three_d::Gm::new(
            rect(30., 170., 400., 400., &frame_input.context),
            crate::color_material(STOCK_PANEL_COLOR),
        )],
        &[],
    );
    let text = view
        .items
        .iter()
        .enumerate()
        .flat_map(|(index, stock)| {
            let price = stock.price.buy_price();
            let quantity = stock.quantity;
            let y = 550. - (index as f32 * 24.);
            [
                crate::make_text_obj(
                    &view::text::capitalize(stock.item.noun_data().singular()),
                    three_d::vec2(40., y),
                    three_d::vec4(1., 1., 1., 1.),
                    &assets.builtin_fonts.text_gen_size_16,
                    &frame_input.context,
                ),
                crate::make_text_obj(
                    &format!("| {price}p"),
                    three_d::vec2(210., y),
                    three_d::vec4(1., 1., 1., 1.),
                    &assets.builtin_fonts.text_gen_size_16,
                    &frame_input.context,
                ),
                crate::make_text_obj(
                    &format!("| {quantity}"),
                    three_d::vec2(300., y),
                    three_d::vec4(1., 1., 1., 1.),
                    &assets.builtin_fonts.text_gen_size_16,
                    &frame_input.context,
                ),
            ]
        })
        .collect::<Vec<_>>();
    draw_in_order(&text, &render_camera, screen);

    screen.render(
        &render_camera,
        [three_d::Gm::new(
            rect(450., 535., 320., 35., &frame_input.context),
            crate::color_material(STOCK_PANEL_COLOR),
        )],
        &[],
    );
    screen.render(
        &render_camera,
        [crate::make_text_obj(
            &format!("Crew points: {}p", view.points),
            three_d::vec2(460., 545.),
            three_d::vec4(1., 1., 1., 1.),
            &assets.builtin_fonts.text_gen_size_20,
            &frame_input.context,
        )],
        &[],
    );
}

pub fn find_stock_at(pos: three_d::Vec2, store_view: &view::StoreView) -> Option<&StoreStock> {
    for (index, stock) in store_view.items.iter().enumerate() {
        if crate::Rect::new(30., 546. - (index as f32 * 24.), 400., 24.).contains(pos) {
            return Some(stock);
        }
    }
    None
}

fn rect(x: f32, y: f32, width: f32, height: f32, context: &three_d::Context) -> three_d::Rectangle {
    three_d::Rectangle::new(
        context,
        three_d::vec2(x + width / 2., y + height / 2.),
        three_d::degrees(0.),
        width,
        height,
    )
}

fn render_objects_for_primary_background(
    background: &BGData<three_d::Texture2DRef>,
    background_offset: Coord,
    camera_x: f32,
    context: &three_d::Context,
) -> Vec<impl three_d::Object> {
    let offset = background_offset as f32 * 120.;

    background
        .primary
        .0
        .layers
        .iter()
        .flat_map(|layer| {
            let width = layer.texture.width() as f32;
            let height = layer.texture.height() as f32;
            let layer_x = f32::from(layer.offset.x) + camera_x * (1. - layer.move_factor) - offset;
            let layer_y = f32::from(layer.offset.y);
            let material = crate::texture_material(&layer.texture);

            if layer.is_looping {
                let repeat_start = f32::floor((camera_x - layer_x) / width) as i16;
                let repeat_end = f32::floor((camera_x + 800. - layer_x) / width) as i16;
                (repeat_start..=repeat_end)
                    .map(|repeat_index| {
                        three_d::Gm::new(
                            rect(
                                layer_x + width * f32::from(repeat_index),
                                layer_y,
                                width,
                                height,
                                context,
                            ),
                            material.clone(),
                        )
                    })
                    .collect()
            } else {
                vec![three_d::Gm::new(
                    rect(layer_x, layer_y, width, height, context),
                    material,
                )]
            }
        })
        .collect()
}

fn draw_secondary_background(
    background: &BGData<three_d::Texture2DRef>,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
) {
    match &background.portrait {
        &PortraitBGData::Color([r, g, b]) => {
            screen.clear(three_d::ClearState::color(
                f32::from(r) / 255.,
                f32::from(g) / 255.,
                f32::from(b) / 255.,
                1.,
            ));
        }
        PortraitBGData::Texture(texture) => {
            let background_object = three_d::Gm::new(
                three_d::Rectangle::new(
                    &frame_input.context,
                    three_d::vec2(crate::WINDOW_WIDTH_F / 2., crate::WINDOW_HEIGHT_F / 2.),
                    three_d::degrees(0.),
                    crate::WINDOW_WIDTH_F,
                    crate::WINDOW_HEIGHT_F,
                ),
                crate::texture_material(texture),
            );

            let render_camera = crate::default_render_camera(frame_input.viewport);
            screen.render(&render_camera, [background_object], &[]);
        }
    };
}

fn get_render_objects_for_entity(
    model: &Model<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    properties: &RenderProperties,
    aftik_colors: &mut HashMap<AftikColorId, AftikColorData>,
    context: &three_d::Context,
) -> Vec<impl three_d::Object> {
    let aftik_color = properties
        .aftik_color
        .as_ref()
        .map_or(asset::color::DEFAULT_COLOR, |aftik_color| {
            lookup_or_log_aftik_color(aftik_color, aftik_colors)
        });
    model
        .layers
        .iter()
        .flat_map(|layer| get_render_object_for_layer(layer, pos, properties, aftik_color, context))
        .collect()
}

fn lookup_or_log_aftik_color(
    color_id: &AftikColorId,
    aftik_colors_map: &mut HashMap<AftikColorId, AftikColorData>,
) -> AftikColorData {
    aftik_colors_map.get(color_id).copied().unwrap_or_else(|| {
        eprintln!("Missing aftik color data for color {color_id:?}!");
        aftik_colors_map.insert(color_id.clone(), asset::color::DEFAULT_COLOR);
        asset::color::DEFAULT_COLOR
    })
}

fn get_render_object_for_layer(
    layer: &TextureLayer<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    properties: &RenderProperties,
    aftik_color: AftikColorData,
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
    let left_x = pos.x - width / 2.;
    let rectangle = three_d::Rectangle::new(
        context,
        three_d::vec2(
            left_x.floor() + width / 2.,
            pos.y + height / 2. - f32::from(layer.positioning.y_offset),
        ),
        three_d::degrees(0.),
        width * direction,
        height,
    );

    let color = layer.color.get_color(&aftik_color);
    let material = crate::texture_color_material(
        &layer.texture,
        three_d::vec4(
            f32::from(color.r) / 255.,
            f32::from(color.g) / 255.,
            f32::from(color.b) / 255.,
            1.,
        ),
    );

    Some(three_d::Gm::new(rectangle, material))
}

fn draw_in_order(
    objects: &[impl three_d::Object],
    camera: &three_d::Camera,
    screen: &three_d::RenderTarget<'_>,
) {
    screen
        .write::<three_d::RendererError>(|| {
            for object in objects {
                object.render(&camera, &[]);
            }
            Ok(())
        })
        .unwrap();
}
