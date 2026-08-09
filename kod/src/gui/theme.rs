pub use crate::settings::ThemeMode;
use eframe::egui;

pub fn apply_theme(ctx: &egui::Context, light: bool) {
    ctx.set_visuals(solarized_visuals(light));
}

// Paleta kolorow Solarized (jasna/ciemna odmiana) zaaplikowana na egui::Visuals.
pub fn solarized_visuals(light: bool) -> egui::Visuals {
    const BASE03: egui::Color32 = egui::Color32::from_rgb(0x00, 0x2b, 0x36);
    const BASE02: egui::Color32 = egui::Color32::from_rgb(0x07, 0x36, 0x42);
    const BASE01: egui::Color32 = egui::Color32::from_rgb(0x58, 0x6e, 0x75);
    const BASE00: egui::Color32 = egui::Color32::from_rgb(0x65, 0x7b, 0x83);
    const BASE0: egui::Color32 = egui::Color32::from_rgb(0x83, 0x94, 0x96);
    const BASE1: egui::Color32 = egui::Color32::from_rgb(0x93, 0xa1, 0xa1);
    const BASE2: egui::Color32 = egui::Color32::from_rgb(0xee, 0xe8, 0xd5);
    const BASE3: egui::Color32 = egui::Color32::from_rgb(0xfd, 0xf6, 0xe3);
    const BLUE: egui::Color32 = egui::Color32::from_rgb(0x26, 0x8b, 0xd2);
    const ORANGE: egui::Color32 = egui::Color32::from_rgb(0xcb, 0x4b, 0x16);
    const RED: egui::Color32 = egui::Color32::from_rgb(0xdc, 0x32, 0x2f);

    let mut visuals = if light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };

    let (bg, widget_bg, text_primary, text_secondary, extreme_bg, active_text) = if light {
        (BASE3, BASE2, BASE00, BASE01, egui::Color32::WHITE, BASE3)
    } else {
        (BASE03, BASE02, BASE0, BASE1, BASE02, BASE03)
    };
    let hover_bg = BLUE.gamma_multiply(0.28);
    let active_bg = BLUE.gamma_multiply(0.55);

    visuals.override_text_color = Some(text_primary);
    visuals.hyperlink_color = BLUE;
    visuals.warn_fg_color = ORANGE;
    visuals.error_fg_color = RED;
    visuals.window_fill = bg;
    visuals.panel_fill = bg;
    visuals.faint_bg_color = widget_bg;
    visuals.extreme_bg_color = extreme_bg;
    visuals.code_bg_color = widget_bg;

    visuals.selection.bg_fill = active_bg;
    visuals.selection.stroke = egui::Stroke::new(1.0, BLUE);

    visuals.widgets.noninteractive.bg_fill = bg;
    visuals.widgets.noninteractive.weak_bg_fill = bg;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, widget_bg);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text_secondary);

    visuals.widgets.inactive.bg_fill = widget_bg;
    visuals.widgets.inactive.weak_bg_fill = widget_bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, widget_bg);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_primary);

    visuals.widgets.hovered.bg_fill = hover_bg;
    visuals.widgets.hovered.weak_bg_fill = hover_bg;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, BLUE);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, text_primary);

    visuals.widgets.active.bg_fill = active_bg;
    visuals.widgets.active.weak_bg_fill = active_bg;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, BLUE);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, active_text);

    visuals.widgets.open = visuals.widgets.active;

    visuals
}
