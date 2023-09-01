use crate::action::trade::PricedItem;
use crate::core::position::Direction;
use crate::macroquad_interface::texture;
use crate::macroquad_interface::texture::TextureStorage;
use crate::view;
use crate::view::{AftikColor, StoreView};
use egui_macroquad::macroquad::color::Color;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::{camera, color, shapes, text, window};

const STORE_UI_COLOR: Color = Color::new(0.2, 0.1, 0.4, 0.6);

pub fn draw_store_view(textures: &TextureStorage, store_view: &StoreView) {
    camera::set_default_camera();
    window::clear_background(textures.lookup_background(store_view.background).color);
    draw_shopkeeper_portrait(textures, store_view.shopkeeper_color);
    draw_store_stock(store_view);
    draw_points_for_store(store_view.points);
}

fn draw_shopkeeper_portrait(textures: &TextureStorage, color: Option<AftikColor>) {
    texture::draw_object(
        &textures.portrait,
        Direction::Left,
        color,
        false,
        Vec2::new(600., 600.),
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
