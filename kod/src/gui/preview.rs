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

        // Naglowek jest ograniczony do tej samej szerokosci co kwadrat podgladu,
        // dzieki czemu przycisk "Wyczyść" dokuje sie do prawego brzegu ramki.
        let mut clear = false;
        ui.allocate_ui_with_layout(
            egui::vec2(side, PREVIEW_HEADER_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(egui::RichText::new("Podgląd").strong());
                clear = super::clear_button(ui);
            },
        );
        if clear {
            self.invalidate_preview();
            self.status = "Wyczyszczono podgląd wygenerowanego kodu.".to_string();
        }

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
            ui.put(
                rect,
                egui::Label::new("Wygeneruj kod, aby zobaczyć podgląd.").wrap(),
            );
        }
    }
}
