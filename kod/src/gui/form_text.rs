use super::{widgets, QrGenApp};
use eframe::egui;

impl QrGenApp {
    pub(super) fn show_text_form(&mut self, ui: &mut egui::Ui) {
        // Pole wypelnia dostepna wysokosc (ok. 18 px na wiersz, minus etykieta).
        let rows = (((ui.available_height() - 40.0) / 18.0) as usize).max(6);
        if widgets::text_area(ui, "Tekst do zakodowania", &mut self.forms.text.data, rows) {
            self.invalidate_preview();
        }
    }
}
