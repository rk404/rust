use super::QrGenApp;
use eframe::egui;

impl QrGenApp {
    pub(super) fn show_link_form(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            let mut clear = false;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔗 Link do strony WWW").strong());
                clear = super::clear_button(ui);
            });
            if clear {
                self.forms.link.url.clear();
                self.invalidate_preview();
                self.status = "Wyczyszczono adres.".to_string();
            }
            ui.add_space(4.0);
            if super::form_singleline(
                ui,
                "Adres strony (URL):",
                &mut self.forms.link.url,
                "np. www.firma.pl lub https://firma.pl/oferta",
            ) {
                self.invalidate_preview();
            }
            if self.forms.link.will_add_scheme() {
                ui.label(
                    egui::RichText::new(format!(
                        "Zakodowany zostanie adres: {}",
                        self.forms.link.normalized_url()
                    ))
                    .small()
                    .weak(),
                );
            }
        });
        self.show_target_path_hint(ui);
    }
}
