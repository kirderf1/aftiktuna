use crate::core::display::{AftikColorId, ModelId};
use crate::core::position::Direction;
use crate::core::store::StoreStock;
use crate::macroquad_interface::texture;
use crate::macroquad_interface::texture::RenderAssets;
use crate::view;
use crate::view::area::RenderProperties;
use crate::view::StoreView;
use macroquad::color::Color;
use macroquad::math::{Rect, Vec2};
use macroquad::{camera, color, shapes, text};

const STORE_UI_COLOR: Color = Color::new(0.2, 0.1, 0.4, 0.6);

pub fn draw_store_view(assets: &mut RenderAssets, store_view: &StoreView) {
    camera::set_default_camera();
    assets
        .lookup_background(&store_view.background)
        .portrait
        .draw();
    draw_shopkeeper_portrait(assets, store_view.shopkeeper_color.clone());
    draw_store_stock(store_view);
    draw_points_for_store(store_view.points);
}

fn draw_shopkeeper_portrait(assets: &mut RenderAssets, aftik_color: Option<AftikColorId>) {
    texture::draw_object(
        &ModelId::portrait(),
        &RenderProperties {
            direction: Direction::Left,
            aftik_color,
            ..RenderProperties::default()
        },
        false,
        Vec2::new(super::WINDOW_WIDTH_F - 200., super::WINDOW_HEIGHT_F),
        assets,
    );
}

const TEXT_SIZE: f32 = 24.;

fn draw_store_stock(store_view: &StoreView) {
    shapes::draw_rectangle(30., 30., 400., 400., STORE_UI_COLOR);
    for (index, stock) in store_view.items.iter().enumerate() {
        text::draw_text(
            &view::text::capitalize(stock.item.noun_data().singular()),
            40.,
            50. + (index as f32 * TEXT_SIZE),
            TEXT_SIZE,
            color::WHITE,
        );
        let price = stock.price.buy_price();
        text::draw_text(
            &format!("| {price}p"),
            210.,
            50. + (index as f32 * TEXT_SIZE),
            TEXT_SIZE,
            color::WHITE,
        );
        let quantity = stock.quantity;
        text::draw_text(
            &format!("| {quantity}"),
            300.,
            50. + (index as f32 * TEXT_SIZE),
            TEXT_SIZE,
            color::WHITE,
        );
    }
}

pub fn find_stock_at(pos: Vec2, store_view: &StoreView) -> Option<&StoreStock> {
    for (index, stock) in store_view.items.iter().enumerate() {
        if Rect::new(30., 30. + (index as f32 * TEXT_SIZE), 400., TEXT_SIZE).contains(pos) {
            return Some(stock);
        }
    }
    None
}

fn draw_points_for_store(points: i32) {
    let text = format!("Crew points: {points}p");
    shapes::draw_rectangle(450., 30., 320., 35., STORE_UI_COLOR);
    text::draw_text(&text, 460., 55., 32., color::WHITE);
}
