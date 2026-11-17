use super::{widgets, DecodeState, QrGenApp, View};
use crate::decode::{self, DecodedQr};
use crate::formats::{EmailForm, FormatKind, LinkForm, VCardData};
use eframe::egui;
use std::path::Path;

const SOURCE_PANEL_WIDTH: f32 = 420.0;

/// Rozpoznany rodzaj odczytanej tresci - decyduje o tym, do ktorego
/// formularza mozna ja przeniesc.
#[derive(Clone)]
pub(super) enum DecodedKind {
    VCard(Box<VCardData>),
    Email(EmailForm),
    Link(String),
    Text(String),
}

impl DecodedKind {
    pub(super) fn classify(decoded: &DecodedQr) -> DecodedKind {
        if let Some(vcard) = &decoded.vcard {
            return DecodedKind::VCard(Box::new(vcard.clone()));
        }
        if let Some(email) = EmailForm::parse_mailto(&decoded.raw) {
            return DecodedKind::Email(email);
        }
        if LinkForm::looks_like_url(&decoded.raw) {
            return DecodedKind::Link(decoded.raw.trim().to_string());
        }
        DecodedKind::Text(decoded.raw.clone())
    }

    fn badge(&self) -> &'static str {
        match self {
            DecodedKind::VCard(_) => "Rozpoznano: wizytówka vCard",
            DecodedKind::Email(_) => "Rozpoznano: wiadomość e-mail",
            DecodedKind::Link(_) => "Rozpoznano: link",
            DecodedKind::Text(_) => "Rozpoznano: tekst",
        }
    }

    fn format(&self) -> FormatKind {
        match self {
            DecodedKind::VCard(_) => FormatKind::VCard,
            DecodedKind::Email(_) => FormatKind::Email,
            DecodedKind::Link(_) => FormatKind::Link,
            DecodedKind::Text(_) => FormatKind::Text,
        }
    }
}

impl QrGenApp {
    /// Widok Odczyt: po lewej zrodlo obrazu, po prawej odczytana zawartosc.
    pub(super) fn show_decode_view(&mut self, ui: &mut egui::Ui) {
        // Wklejenie obrazu ze schowka (Ctrl+V / Cmd+V).
        let paste_pressed = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::V));
        if paste_pressed {
            self.decode_from_clipboard(ui.ctx());
        }

        let source_frame = egui::Frame::new()
            .fill(ui.visuals().panel_fill)
            .inner_margin(egui::Margin {
            left: 20,
            right: 0,
            top: 20,
            bottom: 20,
        });
        egui::Panel::left("decode_source")
            .exact_size(SOURCE_PANEL_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(source_frame)
            .show(ui, |ui| {
                self.show_drop_zone(ui);
                ui.add_space(14.0);
                if self.decode.texture.is_some() || !self.decode.source.is_empty() {
                    self.show_source_card(ui);
                    ui.add_space(14.0);
                }
                ui.label(
                    egui::RichText::new(
                        "Odczyt działa najlepiej na zrzutach ekranu i skanach. \
                         Zdjęcia o niskim kontraście mogą nie zostać rozpoznane.",
                    )
                    .small()
                    .weak(),
                );
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(20),
            )
            .show(ui, |ui| {
                widgets::card(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            self.show_decode_result(ui);
                        });
                });
            });
    }

    /// Strefa upuszczania: klikniecie otwiera wybor pliku, podswietla sie
    /// przy przeciaganiu pliku nad okno.
    fn show_drop_zone(&mut self, ui: &mut egui::Ui) {
        let dark = ui.visuals().dark_mode;
        let accent = super::theme::accent(dark);
        let hovering_file = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 200.0),
            egui::Sense::click(),
        );
        let fill = if hovering_file {
            accent.gamma_multiply(0.15)
        } else {
            ui.visuals().extreme_bg_color
        };
        let stroke_color = if hovering_file || response.hovered() {
            accent
        } else {
            ui.visuals().widgets.inactive.bg_stroke.color
        };
        ui.painter().rect_filled(rect, 10.0, fill);
        let stroke = egui::Stroke::new(1.5, stroke_color);
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
            rect.left_top(),
        ];
        for edge in corners.windows(2) {
            ui.painter()
                .extend(egui::Shape::dashed_line(edge, stroke, 6.0, 4.0));
        }

        let center = rect.center();
        ui.painter().text(
            center - egui::vec2(0.0, 26.0),
            egui::Align2::CENTER_CENTER,
            "Przeciągnij tu obraz z kodem QR",
            egui::FontId::proportional(16.0),
            ui.visuals().strong_text_color(),
        );
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            "albo wklej go ze schowka (Ctrl+V)",
            egui::FontId::proportional(13.0),
            ui.visuals().weak_text_color(),
        );
        ui.painter().text(
            center + egui::vec2(0.0, 30.0),
            egui::Align2::CENTER_CENTER,
            "Kliknij, aby wybrać plik…",
            egui::FontId::proportional(14.0),
            accent,
        );

        if response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
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
    }

    /// Karta z miniatura wczytanego obrazu i jego nazwa.
    fn show_source_card(&mut self, ui: &mut egui::Ui) {
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if let Some(tex) = &self.decode.texture {
                    let size = tex.size_vec2();
                    let scale = (96.0 / size.x.max(size.y)).min(1.0);
                    ui.add(egui::Image::new(tex).fit_to_exact_size(size * scale));
                }
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&self.decode.source).strong());
                    if let Some(tex) = &self.decode.texture {
                        let size = tex.size_vec2();
                        ui.label(
                            egui::RichText::new(format!("{:.0} × {:.0} px", size.x, size.y))
                                .small()
                                .weak(),
                        );
                    }
                });
            });
        });
    }

    fn show_decode_result(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = &self.decode.error {
            ui.colored_label(ui.visuals().error_fg_color, format!("⚠ {error}"));
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Spróbuj innego obrazu albo wklej kod ze schowka.")
                    .small()
                    .weak(),
            );
            return;
        }
        let (Some(result), Some(kind)) = (&self.decode.result, &self.decode.kind) else {
            ui.add_space(8.0);
            widgets::card_title(ui, "Odczytana zawartość");
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Wczytaj obraz z kodem QR, aby zobaczyć jego zawartość.")
                    .weak(),
            );
            return;
        };
        let raw = result.raw.clone();
        let kind = kind.clone();
        let mut open_in_form = false;

        ui.horizontal(|ui| {
            widgets::card_title(ui, "Odczytana zawartość");
            let dark = ui.visuals().dark_mode;
            ui.label(
                egui::RichText::new(kind.badge())
                    .small()
                    .strong()
                    .color(super::theme::success(dark)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Kopiuj treść").clicked() {
                    ui.ctx().copy_text(raw.clone());
                    self.status = "Skopiowano odczytaną treść do schowka.".to_string();
                }
            });
        });
        ui.add_space(4.0);
        ui.separator();

        let row = |ui: &mut egui::Ui, label: &str, value: &str| {
            if value.trim().is_empty() {
                return;
            }
            ui.label(egui::RichText::new(label).weak());
            ui.add(egui::Label::new(value).selectable(true));
            ui.end_row();
        };
        egui::Grid::new("decoded_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| match &kind {
                DecodedKind::VCard(v) => {
                    row(ui, "Imię i nazwisko", &v.full_name());
                    row(ui, "Telefon", &v.phone_work);
                    row(ui, "Telefon komórkowy", &v.phone_mobile);
                    row(ui, "E-mail", &v.email);
                    row(ui, "Strona WWW", &v.website);
                    row(ui, "Firma", &v.company);
                    row(ui, "Stanowisko", &v.title_pl);
                    row(ui, "Fax", &v.fax);
                    let address = [
                        v.street.trim(),
                        &format!("{} {}", v.postal_code.trim(), v.city.trim()),
                        v.country.trim(),
                    ]
                    .iter()
                    .filter(|part| !part.trim().is_empty())
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ");
                    row(ui, "Adres", &address);
                }
                DecodedKind::Email(e) => {
                    row(ui, "Odbiorca", &e.to);
                    row(ui, "Temat", &e.subject);
                    row(ui, "Treść", &e.body);
                }
                DecodedKind::Link(url) => row(ui, "Adres", url),
                DecodedKind::Text(text) => row(ui, "Tekst", text),
            });

        ui.add_space(10.0);
        egui::CollapsingHeader::new("Pokaż surową treść kodu").show(ui, |ui| {
            let mut raw_display = raw.clone();
            ui.add(
                egui::TextEdit::multiline(&mut raw_display)
                    .desired_width(f32::INFINITY)
                    .desired_rows(4)
                    .interactive(false),
            );
        });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Przenieś dane do formularza i zmień kod:");
            let label = format!("Otwórz w formularzu {}", kind.format().label());
            if widgets::primary_button(ui, &label, true, 0.0) {
                open_in_form = true;
            }
        });

        if open_in_form {
            self.open_decoded_in_form(kind);
        }
    }

    /// Przenosi odczytane dane do odpowiedniego formularza i przechodzi do niego.
    fn open_decoded_in_form(&mut self, kind: DecodedKind) {
        let format = kind.format();
        match kind {
            DecodedKind::VCard(v) => self.forms.vcard = *v,
            DecodedKind::Email(e) => self.forms.email = e,
            DecodedKind::Link(url) => self.forms.link = LinkForm { url },
            DecodedKind::Text(text) => self.forms.text.data = text,
        }
        self.switch_view(View::Form(format));
        self.invalidate_preview();
        self.status = format!("Wczytano odczytane dane do formularza {}.", format.label());
    }

    pub(super) fn decode_from_file(&mut self, path: &Path, ctx: &egui::Context) {
        let source = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        // Miniatura wczytanego obrazu (niezaleznie od wyniku dekodowania).
        let texture = image::open(path).ok().map(|img| {
            let rgba = img.to_rgba8();
            let color = super::texture_image(
                rgba.as_raw(),
                rgba.width() as usize,
                rgba.height() as usize,
                false,
            );
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
                    self.decode.error = Some(format!("Brak obrazu w schowku ({e})."));
                    return;
                }
            }
        };

        let width = image_data.width;
        let height = image_data.height;
        let rgba: Vec<u8> = image_data.bytes.into_owned();

        let color = super::texture_image(&rgba, width, height, false);
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
                let kind = DecodedKind::classify(&decoded);
                self.decode = DecodeState {
                    result: Some(decoded),
                    kind: Some(kind),
                    error: None,
                    texture,
                    source,
                };
                self.status = "Odczytano kod QR.".to_string();
            }
            Err(e) => {
                self.decode = DecodeState {
                    result: None,
                    kind: None,
                    error: Some(e),
                    texture,
                    source,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decoded(raw: &str) -> DecodedQr {
        DecodedQr {
            raw: raw.to_string(),
            vcard: crate::formats::vcard::parse_vcard(raw),
        }
    }

    #[test]
    fn classifies_decoded_content() {
        assert!(matches!(
            DecodedKind::classify(&decoded("BEGIN:VCARD\r\nFN:Jan Kowalski\r\nEND:VCARD")),
            DecodedKind::VCard(_)
        ));
        match DecodedKind::classify(&decoded("mailto:jan@firma.pl?subject=Oferta")) {
            DecodedKind::Email(e) => {
                assert_eq!(e.to, "jan@firma.pl");
                assert_eq!(e.subject, "Oferta");
            }
            _ => panic!("oczekiwano e-maila"),
        }
        assert!(matches!(
            DecodedKind::classify(&decoded("https://firma.pl/oferta")),
            DecodedKind::Link(_)
        ));
        assert!(matches!(
            DecodedKind::classify(&decoded("zwykły tekst https://firma.pl")),
            DecodedKind::Text(_)
        ));
    }

    #[test]
    fn each_kind_maps_to_its_form() {
        let kinds = [
            DecodedKind::VCard(Box::default()),
            DecodedKind::Email(EmailForm::default()),
            DecodedKind::Link(String::new()),
            DecodedKind::Text(String::new()),
        ];
        let formats: Vec<_> = kinds.iter().map(|k| k.format()).collect();
        assert_eq!(
            formats,
            [
                FormatKind::VCard,
                FormatKind::Email,
                FormatKind::Link,
                FormatKind::Text
            ]
        );
    }
}
