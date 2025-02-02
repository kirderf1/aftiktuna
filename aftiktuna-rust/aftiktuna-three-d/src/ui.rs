use three_d::egui;

#[derive(Default)]
pub struct UiResult {
    pub triggered_input: bool,
    pub clicked_text_box: bool,
}

pub fn update_ui(app: &mut crate::App, frame_input: &mut three_d::FrameInput) -> UiResult {
    let mut ui_result = UiResult::default();
    app.gui.update(
        &mut frame_input.events,
        frame_input.accumulated_time,
        frame_input.viewport,
        frame_input.device_pixel_ratio,
        |egui_context| {
            ui_result.triggered_input = input_panel(
                &mut app.input_text,
                app.game.ready_to_take_input(),
                std::mem::take(&mut app.request_input_focus),
                egui_context,
            );
            ui_result.clicked_text_box = text_box_panel(&app.text_box_text, egui_context);
        },
    );
    ui_result
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
const TEXT_PANEL_HEIGHT: f32 = 100.;
const TEXT_BOX_MARGIN: f32 = 12.;
const TEXT_BOX_FONT: egui::FontId = egui::FontId::monospace(12.0);

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
                                .line_height(Some(16.))
                                .color(egui::Color32::WHITE),
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
                crate::WINDOW_WIDTH_F - CLICK_ICON_OFFSET - width / 2.,
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
        super::default_render_camera(frame_input.viewport),
        [icon],
        &[],
    );
}
