use super::{DecodeState, QrGenApp, View};
use crate::decode::{self, DecodedQr};
use eframe::egui;
use std::path::Path;

impl QrGenApp {
    /// Widok odczytu: wczytanie obrazu z kodem QR i podglad jego zawartosci.
    pub(super) fn show_decode_view(&mut self, ui: &mut egui::Ui) {
        // Wklejenie obrazu ze schowka (Ctrl+V / Cmd+V).
        let paste_pressed = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::V));
        if paste_pressed {
            self.decode_from_clipboard(ui.ctx());
        }

        ui.group(|ui| {
            ui.label("Wczytaj obraz z kodem QR:");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("Wczytaj obraz...").clicked() {
                    let mut dialog = rfd::FileDialog::new()
                        .add_filter("Obrazy", &["png", "jpg", "jpeg", "bmp"])
                        .add_filter("Wszystkie pliki", &["*"]);
                    if let Some(docs) = crate::platform::documents_dir() {
                        dialog = dialog.set_directory(docs);
                    }
                    if let Some(path) = dialog.pick_file() {
                        self.decode_from_file(&path, ui.ctx());
                    }
                }
                if ui.button("Wklej ze schowka").clicked() {
                    self.decode_from_clipboard(ui.ctx());
                }
            });
            ui.label(
                egui::RichText::new(
                    "Możesz też przeciągnąć plik obrazu na okno aplikacji lub wcisnąć Ctrl+V.",
                )
                .small()
                .weak(),
            );
        });

        if let Some(error) = &self.decode.error {
            ui.add_space(8.0);
            ui.colored_label(ui.visuals().error_fg_color, error);
        }

        let has_result = self.decode.result.is_some();
        if has_result {
            ui.add_space(8.0);
            self.show_decode_result(ui);
        }
    }

    fn show_decode_result(&mut self, ui: &mut egui::Ui) {
        let Some(result) = &self.decode.result else {
            return;
        };
        let raw = result.raw.clone();
        let vcard = result.vcard.clone();
        let source = self.decode.source.clone();

        ui.group(|ui| {
            ui.horizontal(|ui| {
                if let Some(tex) = &self.decode.texture {
                    let size = tex.size_vec2();
                    let scale = (120.0 / size.x.max(size.y)).min(1.0);
                    ui.add(egui::Image::new(tex).fit_to_exact_size(size * scale));
                }
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Odczytana zawartość").strong());
                    if !source.is_empty() {
                        ui.label(egui::RichText::new(format!("Źródło: {source}")).small().weak());
                    }
                    if ui.button("Kopiuj treść").clicked() {
                        ui.ctx().copy_text(raw.clone());
                        self.status = "Skopiowano odczytaną treść do schowka.".to_string();
                    }
                });
            });
            ui.add_space(4.0);
            let width = ui.available_width().max(240.0);
            let mut raw_display = raw.clone();
            ui.add_sized(
                [width, 110.0],
                egui::TextEdit::multiline(&mut raw_display)
                    .desired_rows(4)
                    .interactive(false),
            );
        });

        if let Some(vcard) = vcard {
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.label(egui::RichText::new("Rozpoznano wizytówkę (vCard)").strong());
                ui.add_space(4.0);
                egui::Grid::new("decoded_vcard_grid")
                    .num_columns(2)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        let mut row = |label: &str, value: &str| {
                            if !value.trim().is_empty() {
                                ui.label(egui::RichText::new(label).weak());
                                ui.label(value);
                                ui.end_row();
                            }
                        };
                        row("Imię:", &vcard.first_name);
                        row("Nazwisko:", &vcard.last_name);
                        row("Numer telefonu:", &vcard.phone_work);
                        row("Telefon komórkowy:", &vcard.phone_mobile);
                        row("E-mail:", &vcard.email);
                        row("Strona WWW:", &vcard.website);
                        row("Firma:", &vcard.company);
                        row("Stanowisko:", &vcard.title_pl);
                        row("Fax:", &vcard.fax);
                        row("Adres:", &vcard.street);
                        row("Miasto:", &vcard.city);
                        row("Kod pocztowy:", &vcard.postal_code);
                        row("Kraj:", &vcard.country);
                    });
                ui.add_space(6.0);
                if ui.button("Użyj w formularzu vCard").clicked() {
                    self.forms.vcard = vcard;
                    self.switch_view(View::Form(crate::formats::FormatKind::VCard));
                    self.status =
                        "Wczytano odczytane dane do formularza vCard.".to_string();
                }
            });
        }
    }

    pub(super) fn decode_from_file(&mut self, path: &Path, ctx: &egui::Context) {
        let source = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        // Miniatura wczytanego obrazu (niezaleznie od wyniku dekodowania).
        let texture = image::open(path).ok().map(|img| {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            ctx.load_texture("decoded_source", color, egui::TextureOptions::LINEAR)
        });

        self.set_decode_outcome(decode::decode_file(path), texture, source);
    }

    fn decode_from_clipboard(&mut self, ctx: &egui::Context) {
        let image_data = {
            let Some(clipboard) = self.clipboard() else {
                return;
            };
            match clipboard.get_image() {
                Ok(img) => img,
                Err(e) => {
                    self.decode.error =
                        Some(format!("Brak obrazu w schowku ({e})."));
                    return;
                }
            }
        };

        let width = image_data.width;
        let height = image_data.height;
        let rgba: Vec<u8> = image_data.bytes.into_owned();

        let color = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
        let texture = Some(ctx.load_texture(
            "decoded_source",
            color,
            egui::TextureOptions::LINEAR,
        ));

        self.set_decode_outcome(
            decode::decode_rgba(width, height, &rgba),
            texture,
            "schowek".to_string(),
        );
    }

    fn set_decode_outcome(
        &mut self,
        outcome: Result<DecodedQr, String>,
        texture: Option<egui::TextureHandle>,
        source: String,
    ) {
        match outcome {
            Ok(decoded) => {
                self.decode = DecodeState {
                    result: Some(decoded),
                    error: None,
                    texture,
                    source,
                };
                self.status = "Odczytano kod QR.".to_string();
            }
            Err(e) => {
                self.decode = DecodeState {
                    result: None,
                    error: Some(e),
                    texture,
                    source,
                };
            }
        }
    }
}
