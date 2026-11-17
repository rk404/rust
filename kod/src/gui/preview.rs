use super::QrGenApp;
use eframe::egui;

/// Docelowy bok kwadratu podgladu - staly, identyczny na kazdej zakladce.
const PREVIEW_SIZE: f32 = 420.0;
const PREVIEW_MIN_SIZE: f32 = 220.0;
const PREVIEW_HEADER_HEIGHT: f32 = 26.0;

impl QrGenApp {
    pub(super) fn show_preview(&mut self, ui: &mut egui::Ui) {
        // Rozmiar zalezy wylacznie od szerokosci okna (responsywnosc),
        // nigdy od zawartosci formularza - podglad wyglada tak samo
        // na kazdej zakladce.
        let side = ui
            .available_width()
            .clamp(PREVIEW_MIN_SIZE, PREVIEW_SIZE);

        ui.allocate_ui_with_layout(
            egui::vec2(side, PREVIEW_HEADER_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(egui::RichText::new("Podgląd").strong());
                ui.label(egui::RichText::new("na żywo, plik nie jest zapisany").small().weak());
            },
        );

        let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
            egui::StrokeKind::Inside,
        );

        if let Some(tex) = &self.preview {
            ui.put(rect, egui::Image::new(tex).fit_to_exact_size(rect.size()));
        } else {
            let (text, is_error) = self
                .preview_message
                .clone()
                .unwrap_or_else(|| ("Wpisz dane, aby zobaczyć podgląd kodu.".to_string(), false));
            let rich = if is_error {
                egui::RichText::new(text).color(ui.visuals().error_fg_color)
            } else {
                egui::RichText::new(text).weak()
            };
            ui.put(rect.shrink(12.0), egui::Label::new(rich).wrap());
        }
    }
}
