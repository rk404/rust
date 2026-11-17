pub use crate::settings::ThemeMode;
use eframe::egui::{self, Color32};

/// Paleta aplikacji (jasna i ciemna odmiana). Kontrast tekstu podstawowego
/// i pomocniczego wzgledem tla spelnia WCAG AA (min. 4,5:1).
struct Palette {
    bg: Color32,
    surface: Color32,
    surface_alt: Color32,
    border: Color32,
    input_border: Color32,
    text: Color32,
    muted: Color32,
    accent: Color32,
    error: Color32,
}

fn palette(light: bool) -> Palette {
    let c = |r, g, b| Color32::from_rgb(r, g, b);
    if light {
        Palette {
            bg: c(0xf5, 0xf3, 0xee),
            surface: c(0xff, 0xff, 0xff),
            surface_alt: c(0xef, 0xec, 0xe4),
            border: c(0xdd, 0xd8, 0xcc),
            input_border: c(0xcf, 0xc9, 0xbb),
            text: c(0x1f, 0x29, 0x33),
            muted: c(0x4d, 0x59, 0x64),
            accent: c(0x1b, 0x6f, 0xb5),
            error: c(0xb4, 0x23, 0x18),
        }
    } else {
        Palette {
            bg: c(0x17, 0x1b, 0x20),
            surface: c(0x20, 0x26, 0x2d),
            surface_alt: c(0x2a, 0x31, 0x3a),
            border: c(0x38, 0x41, 0x4b),
            input_border: c(0x4a, 0x55, 0x61),
            text: c(0xe6, 0xe9, 0xec),
            muted: c(0xa8, 0xb2, 0xbc),
            accent: c(0x5a, 0xa9, 0xe6),
            error: c(0xf1, 0x83, 0x7b),
        }
    }
}

/// Kolor akcentu (przycisk glowny, zaznaczenia, odnosniki).
pub fn accent(dark_mode: bool) -> Color32 {
    palette(!dark_mode).accent
}

/// Kolor tekstu na tle akcentu.
pub fn on_accent(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(0x0b, 0x17, 0x22)
    } else {
        Color32::WHITE
    }
}

/// Kolor sukcesu (zielony) czytelny na biezacym tle.
pub fn success(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(0x5f, 0xc3, 0x8a)
    } else {
        Color32::from_rgb(0x1e, 0x6b, 0x3f)
    }
}

pub fn apply_theme(ctx: &egui::Context, light: bool) {
    ctx.set_visuals(visuals(light));
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.interact_size.y = 30.0;
        // Domyslne "small" w egui to 9 px - za male do czytania.
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::proportional(12.0),
        );
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
    });
}

pub fn visuals(light: bool) -> egui::Visuals {
    let p = palette(light);
    let mut visuals = if light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };

    visuals.override_text_color = Some(p.text);
    visuals.weak_text_color = Some(p.muted);
    visuals.hyperlink_color = p.accent;
    visuals.error_fg_color = p.error;
    visuals.window_fill = p.surface;
    visuals.window_stroke = egui::Stroke::new(1.0, p.border);
    visuals.window_corner_radius = egui::CornerRadius::same(10);
    visuals.panel_fill = p.bg;
    visuals.faint_bg_color = p.surface_alt;
    visuals.extreme_bg_color = p.surface;
    visuals.code_bg_color = p.surface_alt;
    visuals.selection.bg_fill = p.accent.gamma_multiply(0.35);
    visuals.selection.stroke = egui::Stroke::new(1.0, p.accent);

    let radius = egui::CornerRadius::same(6);
    let w = &mut visuals.widgets;

    w.noninteractive.bg_fill = p.bg;
    w.noninteractive.weak_bg_fill = p.bg;
    w.noninteractive.bg_stroke = egui::Stroke::new(1.0, p.border);
    w.noninteractive.fg_stroke = egui::Stroke::new(1.0, p.text);
    w.noninteractive.corner_radius = radius;

    w.inactive.bg_fill = p.surface;
    w.inactive.weak_bg_fill = p.surface;
    w.inactive.bg_stroke = egui::Stroke::new(1.0, p.input_border);
    // fg_stroke rysuje m.in. uchwyt paska przewijania - ma byc stonowany,
    // a nie w kolorze tekstu (tekst i tak ma override_text_color).
    w.inactive.fg_stroke = egui::Stroke::new(1.0, p.input_border);
    w.inactive.corner_radius = radius;

    w.hovered.bg_fill = p.surface_alt;
    w.hovered.weak_bg_fill = p.surface_alt;
    w.hovered.bg_stroke = egui::Stroke::new(1.0, p.accent);
    w.hovered.fg_stroke = egui::Stroke::new(1.5, p.text);
    w.hovered.corner_radius = radius;

    // UWAGA: egui rysuje tekst pogrubiony (`.strong()`) kolorem
    // widgets.active.fg_stroke. Gdyby ten kolor byl taki jak tlo (jak w
    // poprzedniej palecie), wszystkie naglowki bylyby niewidoczne.
    w.active.bg_fill = p.surface_alt;
    w.active.weak_bg_fill = p.surface_alt;
    w.active.bg_stroke = egui::Stroke::new(1.0, p.accent);
    w.active.fg_stroke = egui::Stroke::new(2.0, p.text);
    w.active.corner_radius = radius;

    w.open = w.active;
    visuals
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wzgledna luminancja wg WCAG.
    fn luminance(c: Color32) -> f32 {
        let f = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    }

    fn contrast(a: Color32, b: Color32) -> f32 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    #[test]
    fn text_is_readable_on_every_surface_in_both_themes() {
        for light in [true, false] {
            let p = palette(light);
            for bg in [p.bg, p.surface, p.surface_alt] {
                assert!(contrast(p.text, bg) >= 7.0, "tekst, light={light}");
                assert!(contrast(p.muted, bg) >= 4.5, "tekst pomocniczy, light={light}");
            }
            assert!(contrast(p.accent, p.surface) >= 3.0, "akcent, light={light}");
            assert!(contrast(p.error, p.surface) >= 4.5, "blad, light={light}");
        }
        assert!(contrast(on_accent(false), accent(false)) >= 4.5);
        assert!(contrast(on_accent(true), accent(true)) >= 4.5);
    }

    #[test]
    fn strong_text_differs_from_background() {
        for light in [true, false] {
            let v = visuals(light);
            let strong = v.strong_text_color();
            assert!(contrast(strong, v.panel_fill) >= 7.0, "light={light}");
            assert!(contrast(strong, v.extreme_bg_color) >= 7.0, "light={light}");
        }
    }
}
