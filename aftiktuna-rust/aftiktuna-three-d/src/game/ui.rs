use crate::Assets;
use aftiktuna::command_suggestion::Suggestion;
use aftiktuna::view::FullStatus;
use aftiktuna_three_d::asset::LazilyLoadedModels;
use aftiktuna_three_d::render;
use three_d::egui;

#[derive(Default)]
pub struct UiResult {
    pub triggered_input: bool,
    pub closed_status_window: bool,
    pub clicked_text_box: bool,
    pub clicked_suggestion: Option<Suggestion>,
}

pub fn update_ui(
    gui: &mut three_d::GUI,
    frame_input: &mut three_d::FrameInput,
    state: &mut super::State,
    assets: &mut Assets,
) -> UiResult {
    gui.context().style_mut(|style| {
        style.spacing.menu_margin = egui::Margin::ZERO;
        style.spacing.item_spacing = egui::Vec2::ZERO;
        style.visuals.menu_rounding = egui::Rounding::ZERO;
        style.visuals.popup_shadow = egui::Shadow::NONE;
        style.visuals.window_fill = egui::Color32::TRANSPARENT;
        style.visuals.window_stroke = egui::Stroke::NONE;
    });

    let mut ui_result = UiResult::default();
    gui.update(
        &mut frame_input.events,
        frame_input.accumulated_time,
        frame_input.viewport,
        frame_input.device_pixel_ratio,
        |egui_context| {
            egui_context.style_mut(|style| {
                style.visuals.override_text_color = None;
                style.visuals.widgets.noninteractive.bg_stroke =
                    egui::Stroke::new(1., egui::Color32::from_gray(60));
            });

            ui_result.triggered_input = input_panel(
                &mut state.input_text,
                state.game.ready_to_take_input() && state.displayed_status.is_none(),
                std::mem::take(&mut state.request_input_focus),
                egui_context,
            );

            egui_context.style_mut(|style| {
                style.visuals.override_text_color = Some(egui::Color32::WHITE);
                style.visuals.widgets.noninteractive.bg_stroke =
                    egui::Stroke::new(1., egui::Color32::WHITE);
            });

            if !state.text_box_text.is_empty() {
                ui_result.clicked_text_box = text_box_panel(&state.text_box_text, egui_context);
            }

            if let Some(status) = &state.displayed_status {
                ui_result.closed_status_window = !show_status_screen(status, egui_context);
            } else {
                ui_result.clicked_suggestion =
                    show_tooltip_and_menu(state, &mut assets.models, egui_context);
            }
        },
    );
    ui_result
}

const STATUS_DISPLAY_OUTER_MARGIN: egui::Vec2 = egui::vec2(180., 50.);
const STATUS_WINDOW_END_COMPENSATION: egui::Vec2 = egui::vec2(24., 43.);
const STATUS_DISPLAY_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(
    (0.25 * 0.95 * 255.) as u8,
    (0.2 * 0.95 * 255.) as u8,
    (0.3 * 0.95 * 255.) as u8,
    (0.95 * 255.) as u8,
);

fn show_status_screen(status: &FullStatus, egui_context: &egui::Context) -> bool {
    let mut is_open = true;
    egui::Window::new(egui::RichText::new("Crew Status").font(egui::FontId::monospace(15.0)))
        .open(&mut is_open)
        .collapsible(false)
        .fixed_rect(
            egui::Rect::from_min_max(
                egui::Pos2::ZERO,
                egui::pos2(
                    aftiktuna_three_d::WINDOW_WIDTH_F,
                    aftiktuna_three_d::WINDOW_HEIGHT_F - INPUT_PANEL_HEIGHT,
                ) - STATUS_WINDOW_END_COMPENSATION,
            )
            .shrink2(STATUS_DISPLAY_OUTER_MARGIN),
        )
        .frame(egui::Frame {
            inner_margin: egui::Margin::symmetric(12., 6.),
            fill: STATUS_DISPLAY_COLOR,
            stroke: egui::Stroke::new(1., egui::Color32::WHITE),
            ..Default::default()
        })
        .vscroll(true)
        .show(egui_context, |ui| {
            for line in &status.ship {
                ui.label(
                    egui::RichText::new(line)
                        .font(TEXT_BOX_FONT)
                        .line_height(Some(15.)),
                );
            }
            for character_text in &status.crew {
                ui.separator();
                for line in character_text {
                    ui.label(
                        egui::RichText::new(line)
                            .font(TEXT_BOX_FONT)
                            .line_height(Some(13.)),
                    );
                }
            }
        });
    is_open
}

fn show_tooltip_and_menu(
    state: &super::State,
    models: &mut LazilyLoadedModels,
    egui_context: &egui::Context,
) -> Option<Suggestion> {
    if let Some(command_tooltip) = &state.command_tooltip {
        show_hovering(
            egui::Id::new("commands"),
            egui_context,
            command_tooltip.pos - three_d::vec2(state.camera.camera_x, 0.),
            |ui| {
                let mut clicked_suggestion = None;
                for suggestion in &command_tooltip.commands {
                    let mut prepared = egui::Frame::none()
                        .outer_margin(egui::Margin::ZERO)
                        .inner_margin(egui::Margin::symmetric(TEXT_BOX_MARGIN, 1.))
                        .begin(ui);

                    prepared.content_ui.label(suggestion.text());

                    let response = prepared.allocate_space(ui).interact(egui::Sense::click());
                    if response.clicked() {
                        clicked_suggestion = Some(suggestion.clone());
                    }

                    prepared.frame.fill = if response.hovered() {
                        TEXT_BOX_HIGHLIGHT_COLOR
                    } else {
                        TEXT_BOX_COLOR
                    };

                    prepared.paint(ui);
                }
                clicked_suggestion
            },
        )
    } else {
        let tooltips_list = super::get_hovered_object_names(
            &state.cached_objects,
            state.mouse_pos + three_d::vec2(state.camera.camera_x, 0.),
            models,
        );

        if !tooltips_list.is_empty() {
            show_hovering(
                egui::Id::new("game_tooltip"),
                egui_context,
                state.mouse_pos,
                |ui| {
                    for &line in &tooltips_list {
                        egui::Frame::none()
                            .outer_margin(egui::Margin::ZERO)
                            .fill(TEXT_BOX_COLOR)
                            .inner_margin(egui::Margin::symmetric(TEXT_BOX_MARGIN, 1.))
                            .show(ui, |ui| {
                                ui.label(line);
                            });
                    }
                },
            );
        }
        None
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
const TEXT_BOX_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(
    (0.5 * 0.6 * 255.) as u8,
    (0.3 * 0.6 * 255.) as u8,
    (0.6 * 0.6 * 255.) as u8,
    (0.6 * 255.) as u8,
);
const TEXT_PANEL_HEIGHT: f32 = 100.;
const TEXT_BOX_MARGIN: f32 = 12.;
const TEXT_BOX_FONT: egui::FontId = egui::FontId::monospace(11.0);

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
                                .line_height(Some(13.)),
                        );
                    }
                });
        })
        .response;
    response.interact(egui::Sense::click()).clicked()
}

const CLICK_ICON_OFFSET: f32 = 5.;

pub fn draw_frame_click_icon(
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
                aftiktuna_three_d::WINDOW_WIDTH_F - CLICK_ICON_OFFSET - width / 2.,
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
    screen.render(
        render::default_render_camera(frame_input.viewport),
        [icon],
        &[],
    );
}

fn show_hovering<T>(
    id: egui::Id,
    egui_context: &egui::Context,
    pos: three_d::Vec2,
    content: impl Fn(&mut egui::Ui) -> T,
) -> T {
    egui::show_tooltip_at(
        egui_context,
        egui::LayerId::background(),
        id,
        egui::pos2(pos.x, aftiktuna_three_d::WINDOW_HEIGHT_F - pos.y - 4.),
        |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

            let max_rect = {
                let mut ui = ui.new_child(egui::UiBuilder::new().invisible());
                content(&mut ui);
                ui.min_rect()
            };

            ui.allocate_new_ui(
                egui::UiBuilder::new()
                    .max_rect(max_rect)
                    .layout(egui::Layout::top_down_justified(egui::Align::Min)),
                content,
            )
        },
    )
    .inner
}
