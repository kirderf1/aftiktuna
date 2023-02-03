use eframe::egui;
use eframe::egui::ScrollArea;

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.resizable = false;
    eframe::run_native(
        "Aftiktuna",
        options,
        Box::new(|_cc| Box::<App>::default()),
    );
}

#[derive(Default)]
struct App {
    text_lines: Vec<String>,
    input: String,
}

const FONT: egui::FontId = egui::FontId::monospace(15.0);

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("input").show(ctx, |ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .font(FONT)
                    .desired_width(f32::INFINITY),
            );

            if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                self.text_lines.push(self.input.clone());
                self.input.clear();
                ctx.memory().request_focus(response.id);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for text in &self.text_lines {
                        ui.label(egui::RichText::new(text).font(FONT));
                    }
                })
        });
    }
}
