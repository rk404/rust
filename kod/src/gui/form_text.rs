use super::QrGenApp;
use eframe::egui;

impl QrGenApp {
    pub(super) fn show_text_form(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            let mut clear = false;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📝 Dane do zakodowania").strong());
                clear = super::clear_button(ui);
            });
            if clear {
                self.forms.text.data.clear();
                self.invalidate_preview();
                self.status = "Wyczyszczono dane do zakodowania.".to_string();
            }
            ui.add_space(4.0);
            if super::form_multiline(ui, "Tekst:", &mut self.forms.text.data, 5) {
                self.invalidate_preview();
            }
        });
        self.show_target_path_hint(ui);
    }
}
