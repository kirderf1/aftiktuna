use crate::Assets;
use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::placement;
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::ModelId;
use aftiktuna::core::position::Direction;
use aftiktuna::core::store::StoreStock;
use aftiktuna::view::area::{ObjectRenderData, RenderData, RenderProperties};
use aftiktuna::view::{self, Frame};
use aftiktuna_three_d::{render, Camera};
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
            let background_objects = render::render_objects_for_primary_background(
                assets
                    .backgrounds
                    .get_or_default(&BackgroundId::location_choice()),
                0,
                0.,
                &frame_input.context,
            );

            let render_camera = render::default_render_camera(frame_input.viewport);
            render::draw_in_order(&background_objects, &render_camera, screen);
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
            render::draw_secondary_background(
                assets.backgrounds.get_or_default(&data.background),
                screen,
                frame_input,
            );

            let x = match data.direction {
                Direction::Left => aftiktuna_three_d::WINDOW_WIDTH_F - 300.,
                Direction::Right => 300.,
            };
            let objects = render::get_render_objects_for_entity(
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
            let render_camera = render::default_render_camera(frame_input.viewport);
            render::draw_in_order(&objects, &render_camera, screen);

            if data.darkness > 0. {
                let light_pos = three_d::vec2(
                    match data.direction {
                        Direction::Left => 530.,
                        Direction::Right => 270.,
                    },
                    320.,
                );
                render::render_darkness(
                    light_pos,
                    250.,
                    data.darkness,
                    frame_input.viewport,
                    screen,
                    &frame_input.context,
                );
            }
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
    let background_objects = render::render_objects_for_primary_background(
        assets.backgrounds.get_or_default(&render_data.background),
        render_data.background_offset,
        camera.camera_x,
        &frame_input.context,
    );
    let entity_objects = cached_objects
        .iter()
        .flat_map(|(pos, object)| {
            let mut render_objects = render::get_render_objects_for_entity(
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
                    render_objects.extend(render::get_render_objects_for_entity(
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

    let render_camera = render::get_render_camera(camera, frame_input.viewport);
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

    if render_data.darkness > 0. {
        let light_pos = three_d::vec2(
            placement::coord_to_center_x(render_data.character_coord) - camera.camera_x,
            300.,
        );
        render::render_darkness(
            light_pos,
            200.,
            render_data.darkness,
            frame_input.viewport,
            screen,
            &frame_input.context,
        );
    }

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
                three_d::vec2(
                    aftiktuna_three_d::WINDOW_WIDTH_F - 10. - texture_width / 2.,
                    250.,
                ),
                three_d::degrees(0.),
                texture_width,
                texture_height,
            ),
            arrow_material,
        ));
    }

    let default_camera = render::default_render_camera(frame_input.viewport);
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
    render::draw_secondary_background(
        assets.backgrounds.get_or_default(&view.background),
        screen,
        frame_input,
    );

    let objects = render::get_render_objects_for_entity(
        assets.models.lookup_model(&ModelId::portrait()),
        three_d::vec2(aftiktuna_three_d::WINDOW_WIDTH_F - 200., 0.),
        &RenderProperties {
            direction: Direction::Left,
            aftik_color: view.shopkeeper_color.clone(),
            ..RenderProperties::default()
        },
        &mut assets.aftik_colors,
        &frame_input.context,
    );
    let render_camera = render::default_render_camera(frame_input.viewport);
    render::draw_in_order(&objects, &render_camera, screen);
    const STOCK_PANEL_COLOR: three_d::Vec4 = three_d::vec4(0.2, 0.1, 0.4, 0.6);
    screen.render(
        &render_camera,
        [three_d::Gm::new(
            render::rect(30., 170., 400., 400., &frame_input.context),
            render::color_material(STOCK_PANEL_COLOR),
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
    render::draw_in_order(&text, &render_camera, screen);

    screen.render(
        &render_camera,
        [three_d::Gm::new(
            render::rect(450., 535., 320., 35., &frame_input.context),
            render::color_material(STOCK_PANEL_COLOR),
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
        if aftiktuna_three_d::Rect::new(30., 546. - (index as f32 * 24.), 400., 24.).contains(pos) {
            return Some(stock);
        }
    }
    None
}
