use super::QrGenApp;
use eframe::egui;

impl QrGenApp {
    pub(super) fn show_vcard_form(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            let mut clear = false;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📇 Dane wizytówki (vCard)").strong());
                clear = super::clear_button(ui);
            });
            if clear {
                self.forms.vcard = Default::default();
                self.invalidate_preview();
                self.status = "Wyczyszczono dane wizytówki.".to_string();
            }
            ui.add_space(4.0);

            let mut changed = false;
            {
                let v = &mut self.forms.vcard;
                changed |= super::form_singleline(ui, "Imię:", &mut v.first_name, "Jan");
                changed |= super::form_singleline(ui, "Nazwisko:", &mut v.last_name, "Kowalski");
                changed |=
                    super::form_singleline(ui, "Numer telefonu:", &mut v.phone_work, "58 307 00 00");
                changed |= super::form_singleline(
                    ui,
                    "Telefon komórkowy:",
                    &mut v.phone_mobile,
                    "600 000 000",
                );
                changed |=
                    super::form_singleline(ui, "E-mail:", &mut v.email, "jan.kowalski@firma.pl");
                changed |= super::form_singleline(
                    ui,
                    "Strona WWW (URL):",
                    &mut v.website,
                    "https://firma.pl",
                );
                changed |= super::form_singleline(ui, "Firma:", &mut v.company, "");
                changed |= super::form_singleline(
                    ui,
                    "Stanowisko:",
                    &mut v.title_pl,
                    "Kierownik Logistyki Dostaw",
                );
                changed |= super::form_singleline(
                    ui,
                    "Stanowisko (EN):",
                    &mut v.title_en,
                    "Supply Chain Manager",
                );
                changed |= super::form_singleline(ui, "Fax:", &mut v.fax, "");
                changed |= super::form_singleline(ui, "Adres:", &mut v.street, "ul. Portowa 1");
                changed |= super::form_singleline(ui, "Miasto:", &mut v.city, "Gdańsk");
                changed |= super::form_singleline(ui, "Kod pocztowy:", &mut v.postal_code, "80-001");
                changed |= super::form_singleline(ui, "Kraj:", &mut v.country, "Polska");
            }
            if changed {
                self.invalidate_preview();
            }

            ui.add_space(4.0);
            ui.checkbox(
                &mut self.docx_enabled,
                "Generuj dokument Word (.docx) z danymi do wizytówki",
            );
            ui.label(
                egui::RichText::new(
                    "Blok angielski w dokumencie powstaje tylko, gdy podano Stanowisko (EN).",
                )
                .small()
                .weak(),
            );
        });
        self.show_target_path_hint(ui);
    }
}
