use super::{widgets, QrGenApp};
use eframe::egui;

impl QrGenApp {
    /// Formularz wizytowki podzielony na sekcje; krotkie pola w kolumnach.
    pub(super) fn show_vcard_form(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        {
            let v = &mut self.forms.vcard;

            widgets::section_title(ui, "Osoba");
            ui.columns(2, |c| {
                changed |= widgets::text_field(&mut c[0], "Imię", &mut v.first_name, "Jan");
                changed |= widgets::text_field(&mut c[1], "Nazwisko", &mut v.last_name, "Kowalski");
            });

            ui.add_space(8.0);
            widgets::section_title(ui, "Kontakt");
            ui.columns(2, |c| {
                changed |=
                    widgets::text_field(&mut c[0], "Telefon", &mut v.phone_work, "58 307 00 00");
                changed |= widgets::text_field(
                    &mut c[1],
                    "Telefon komórkowy",
                    &mut v.phone_mobile,
                    "600 000 000",
                );
            });
            ui.columns(2, |c| {
                changed |= widgets::text_field(
                    &mut c[0],
                    "E-mail",
                    &mut v.email,
                    "jan.kowalski@firma.pl",
                );
                changed |= widgets::text_field(&mut c[1], "Fax", &mut v.fax, "");
            });
            changed |= widgets::text_field(ui, "Strona WWW", &mut v.website, "https://firma.pl");

            ui.add_space(8.0);
            widgets::section_title(ui, "Firma");
            changed |= widgets::text_field(ui, "Nazwa firmy", &mut v.company, "");
            ui.columns(2, |c| {
                changed |= widgets::text_field(
                    &mut c[0],
                    "Stanowisko",
                    &mut v.title_pl,
                    "Kierownik Logistyki Dostaw",
                );
                changed |= widgets::text_field(
                    &mut c[1],
                    "Stanowisko (EN)",
                    &mut v.title_en,
                    "Supply Chain Manager",
                );
            });

            ui.add_space(8.0);
            widgets::section_title(ui, "Adres");
            changed |= widgets::text_field(ui, "Ulica i numer", &mut v.street, "ul. Portowa 1");
            ui.columns(3, |c| {
                changed |=
                    widgets::text_field(&mut c[0], "Kod pocztowy", &mut v.postal_code, "80-001");
                changed |= widgets::text_field(&mut c[1], "Miasto", &mut v.city, "Gdańsk");
                changed |= widgets::text_field(&mut c[2], "Kraj", &mut v.country, "Polska");
            });
        }
        if changed {
            self.invalidate_preview();
        }

        ui.add_space(12.0);
        egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .corner_radius(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.checkbox(
                    &mut self.docx_enabled,
                    "Zapisz też dokument Word (.docx) z danymi do wizytówki",
                );
                ui.label(
                    egui::RichText::new(
                        "Blok angielski w dokumencie powstaje tylko, gdy podano Stanowisko (EN).",
                    )
                    .small()
                    .weak(),
                );
            });
    }
}
