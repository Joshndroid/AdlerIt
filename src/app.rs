use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, RichText, Stroke, TextEdit,
};

use crate::hash;

const ACCENT: Color32 = Color32::from_rgb(25, 118, 210);

pub struct AdlerApp {
    input: String,
    copy_label: &'static str,
}

impl AdlerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        apply_accent(&cc.egui_ctx);
        Self {
            input: String::new(),
            copy_label: "Copy",
        }
    }
}

impl eframe::App for AdlerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("AdlerIt").strong());
            ui.label(RichText::new("Adler-32 calculator").weak());
        });
        ui.add_space(8.0);
        ui.separator();

        ui.add_space(14.0);
        ui.label(RichText::new("INPUT").small().strong());
        ui.add_space(6.0);
        let response = ui.add_sized(
            [ui.available_width(), 140.0],
            TextEdit::multiline(&mut self.input)
                .hint_text("Type or paste text / a number string...")
                .font(FontId::monospace(16.0))
                .desired_rows(6),
        );
        if response.changed() {
            self.copy_label = "Copy";
        }

        ui.add_space(16.0);
        ui.label(RichText::new("ADLER-32").small().strong());
        ui.add_space(6.0);
        let value = hash::adler32(self.input.as_bytes());
        let hex = hash::hex(value);
        egui::Frame::group(ui.style())
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!("0x{hex}"))
                                .monospace()
                                .size(28.0)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(format!(
                                "Decimal: {value} · {} UTF-8 bytes",
                                self.input.len()
                            ))
                            .weak(),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(self.copy_label).clicked() {
                            ctx.copy_text(hex.clone());
                            self.copy_label = "Copied";
                        }
                    });
                });
            });
    }
}

fn apply_accent(ctx: &egui::Context) {
    ctx.global_style_mut(|style| {
        style.visuals.selection.bg_fill = ACCENT;
        style.visuals.hyperlink_color = ACCENT;
        style.visuals.widgets.active.bg_fill = ACCENT;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    });
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "JetBrains Mono".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")).into(),
    );
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, "JetBrains Mono".to_owned());
    }
    ctx.set_fonts(fonts);
}
