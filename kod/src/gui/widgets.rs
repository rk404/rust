//! Wspolne widzety interfejsu: karty, przelaczniki, pola z etykieta nad polem.

use super::theme;
use eframe::egui::{self, Color32, RichText, Stroke};

/// Karta: bialy panel z ramka i zaokraglonymi rogami.
pub fn card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let border = ui.visuals().widgets.noninteractive.bg_stroke.color;
    egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(10.0)
        .inner_margin(16.0)
        .show(ui, add_contents)
        .inner
}

/// Przelacznik segmentowy (jedna opcja aktywna). Zwraca true przy zmianie.
pub fn segmented<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    current: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    let selected_fill = ui.visuals().extreme_bg_color;
    let border = ui.visuals().widgets.noninteractive.bg_stroke.color;
    let strong = ui.visuals().strong_text_color();
    let muted = ui.visuals().weak_text_color();
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(8.0)
        .inner_margin(3.0)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.horizontal(|ui| {
                for (value, label) in options {
                    let selected = *current == *value;
                    let text = RichText::new(*label)
                        .color(if selected { strong } else { muted })
                        .size(14.0);
                    let button = egui::Button::new(text)
                        .min_size(egui::vec2(0.0, 28.0))
                        .corner_radius(6.0)
                        .fill(if selected {
                            selected_fill
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(if selected {
                            Stroke::new(1.0, border)
                        } else {
                            Stroke::NONE
                        });
                    if ui.add(button).clicked() && !selected {
                        *current = *value;
                        changed = true;
                    }
                }
            });
        });
    changed
}

/// Naglowek sekcji formularza (male litery kapitalikami, kolor pomocniczy).
pub fn section_title(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(text.to_uppercase())
            .size(11.0)
            .strong()
            .color(ui.visuals().weak_text_color()),
    );
}

fn field_label(ui: &mut egui::Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .size(12.0)
            .color(ui.visuals().weak_text_color()),
    );
}

/// Pole jednowierszowe z etykieta nad polem, na cala dostepna szerokosc.
pub fn text_field(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str) -> bool {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;
        ui.spacing_mut().interact_size.y = 32.0;
        field_label(ui, label);
        ui.add(
            egui::TextEdit::singleline(value)
                .hint_text(hint)
                .desired_width(f32::INFINITY)
                .margin(egui::Margin::symmetric(8, 6)),
        )
        .changed()
    })
    .inner
}

/// Pole wielowierszowe z etykieta nad polem.
pub fn text_area(ui: &mut egui::Ui, label: &str, value: &mut String, rows: usize) -> bool {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;
        field_label(ui, label);
        ui.add(
            egui::TextEdit::multiline(value)
                .desired_rows(rows)
                .desired_width(f32::INFINITY)
                .margin(egui::Margin::symmetric(8, 6)),
        )
        .changed()
    })
    .inner
}

/// Przycisk glowny w kolorze akcentu.
pub fn primary_button(ui: &mut egui::Ui, label: &str, enabled: bool, width: f32) -> bool {
    let dark = ui.visuals().dark_mode;
    let fill = theme::accent(dark);
    let button = egui::Button::new(
        RichText::new(label)
            .size(15.0)
            .strong()
            .color(theme::on_accent(dark)),
    )
    .fill(fill)
    .stroke(Stroke::NONE)
    .min_size(egui::vec2(width, 38.0));
    ui.add_enabled(enabled, button).clicked()
}

/// Przycisk pomocniczy (zwykla ramka).
pub fn secondary_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
    let button = egui::Button::new(label).min_size(egui::vec2(0.0, 38.0));
    ui.add_enabled(enabled, button).clicked()
}

/// Odnosnik-przycisk bez ramki, w kolorze akcentu.
pub fn link_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
    let color = theme::accent(ui.visuals().dark_mode);
    let button = egui::Button::new(RichText::new(label).color(color)).frame(false);
    ui.add_enabled(enabled, button)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

/// Naglowek karty (wiekszy od zwyklego tekstu).
pub fn card_title(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(15.0)
            .strong()
            .color(ui.visuals().strong_text_color()),
    );
}
