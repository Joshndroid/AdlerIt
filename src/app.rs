use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId, RichText, TextEdit};

use crate::{
    hash,
    theme::{self, ThemeMode},
};

pub struct AdlerApp {
    input: String,
    theme_mode: ThemeMode,
    copy_label: &'static str,
}

impl AdlerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        let app = Self {
            input: String::new(),
            theme_mode: ThemeMode::System,
            copy_label: "Copy",
        };
        theme::apply(&cc.egui_ctx, app.theme_mode);
        app
    }
}

impl eframe::App for AdlerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let old_theme = self.theme_mode;

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("AdlerIt").strong());
            ui.label(RichText::new("Adler-32 calculator").weak());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("theme")
                    .selected_text(self.theme_mode.label())
                    .show_ui(ui, |ui| {
                        for mode in ThemeMode::ALL {
                            ui.selectable_value(&mut self.theme_mode, mode, mode.label());
                        }
                    });
            });
        });
        ui.add_space(8.0);
        ui.separator();

        ui.add_space(18.0);
        ui.label(RichText::new("INPUT").small().strong());
        ui.add_space(6.0);
        let response = ui.add_sized(
            [ui.available_width(), 220.0],
            TextEdit::multiline(&mut self.input)
                .hint_text("Type or paste text / a number string…")
                .font(FontId::monospace(16.0))
                .desired_rows(9),
        );
        if response.changed() {
            self.copy_label = "Copy";
        }

        ui.add_space(20.0);
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

        if old_theme != self.theme_mode {
            theme::apply(&ctx, self.theme_mode);
        }
    }
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
