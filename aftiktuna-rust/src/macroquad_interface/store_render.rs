use crate::core::position::Direction;
use crate::core::PricedItem;
use crate::macroquad_interface::texture;
use crate::macroquad_interface::texture::RenderAssets;
use crate::view;
use crate::view::area::{AftikColor, RenderProperties};
use crate::view::StoreView;
use egui_macroquad::macroquad::color::Color;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::{camera, color, shapes, text};

const STORE_UI_COLOR: Color = Color::new(0.2, 0.1, 0.4, 0.6);

pub fn draw_store_view(assets: &RenderAssets, store_view: &StoreView) {
    camera::set_default_camera();
    texture::draw_background_portrait(assets.lookup_background(&store_view.background));
    draw_shopkeeper_portrait(assets, store_view.shopkeeper_color);
    draw_store_stock(store_view);
    draw_points_for_store(store_view.points);
}

fn draw_shopkeeper_portrait(assets: &RenderAssets, aftik_color: Option<AftikColor>) {
    texture::draw_object(
        &assets.portrait,
        &RenderProperties {
            direction: Direction::Left,
            aftik_color,
            ..RenderProperties::default()
        },
        false,
        Vec2::new(600., 600.),
        &assets.aftik_colors,
    );
}

const TEXT_SIZE: f32 = 32.;

fn draw_store_stock(store_view: &StoreView) {
    shapes::draw_rectangle(30., 30., 400., 400., STORE_UI_COLOR);
    for (index, priced_item) in store_view.items.iter().enumerate() {
        text::draw_text(
            &view::capitalize(priced_item.item.noun_data().singular()),
            50.,
            55. + (index as f32 * TEXT_SIZE),
            TEXT_SIZE,
            color::WHITE,
        );
        text::draw_text(
            &format!("| {}p", priced_item.price),
            290.,
            55. + (index as f32 * TEXT_SIZE),
            TEXT_SIZE,
            color::WHITE,
        );
    }
}

pub fn find_stock_at(pos: Vec2, store_view: &StoreView) -> Option<&PricedItem> {
    for (index, priced_item) in store_view.items.iter().enumerate() {
        if Rect::new(30., 30. + (index as f32 * TEXT_SIZE), 400., TEXT_SIZE).contains(pos) {
            return Some(priced_item);
        }
    }
    None
}

fn draw_points_for_store(points: i32) {
    let text = format!("Crew points: {points}p");
    shapes::draw_rectangle(450., 30., 320., 35., STORE_UI_COLOR);
    text::draw_text(&text, 460., 55., 32., color::WHITE);
}
