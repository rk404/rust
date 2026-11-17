use super::{widgets, QrGenApp};
use eframe::egui;

impl QrGenApp {
    pub(super) fn show_link_form(&mut self, ui: &mut egui::Ui) {
        if widgets::text_field(
            ui,
            "Adres strony (URL)",
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
    }
}
