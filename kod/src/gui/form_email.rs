use super::{widgets, QrGenApp};
use eframe::egui;

impl QrGenApp {
    pub(super) fn show_email_form(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        changed |= widgets::text_field(
            ui,
            "Odbiorca (e-mail)",
            &mut self.forms.email.to,
            "adres@firma.pl",
        );
        changed |= widgets::text_field(ui, "Temat", &mut self.forms.email.subject, "");
        changed |= widgets::text_area(ui, "Treść", &mut self.forms.email.body, 6);
        if changed {
            self.invalidate_preview();
        }
    }
}
