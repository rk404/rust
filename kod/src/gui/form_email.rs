use super::QrGenApp;
use eframe::egui;

impl QrGenApp {
    pub(super) fn show_email_form(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            let mut clear = false;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("✉ Wiadomość e-mail").strong());
                clear = super::clear_button(ui);
            });
            if clear {
                self.forms.email = Default::default();
                self.invalidate_preview();
                self.status = "Wyczyszczono dane wiadomości.".to_string();
            }
            ui.add_space(4.0);

            let mut changed = false;
            changed |= super::form_singleline(
                ui,
                "Odbiorca (e-mail):",
                &mut self.forms.email.to,
                "adres@firma.pl",
            );
            changed |= super::form_singleline(ui, "Temat:", &mut self.forms.email.subject, "");
            changed |= super::form_multiline(ui, "Treść:", &mut self.forms.email.body, 4);
            if changed {
                self.invalidate_preview();
            }
        });
        self.show_target_path_hint(ui);
    }
}
