use eframe::egui::{self, Color32};

/// Material flat blue (Material Blue 500), used as the single accent color.
const ACCENT: Color32 = Color32::from_rgb(33, 150, 243);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

pub fn apply(ctx: &egui::Context, mode: ThemeMode) {
    let dark = match mode {
        ThemeMode::System => ctx.system_theme().unwrap_or(egui::Theme::Dark) == egui::Theme::Dark,
        ThemeMode::Light => false,
        ThemeMode::Dark => true,
    };

    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.selection.bg_fill = ACCENT;
    visuals.hyperlink_color = ACCENT;
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.hovered.bg_stroke.color = ACCENT;
    ctx.set_visuals(visuals);
}
