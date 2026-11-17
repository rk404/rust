use super::{widgets, QrGenApp};
use crate::formats::FormatKind;
use crate::platform;
use eframe::egui;

impl QrGenApp {
    /// Odnosniki do plikow w pasku stanu: Otwórz folder, Otwórz plik, Drukuj.
    /// Dodawane od prawej, wiec kolejnosc na ekranie jest odwrotna do kodu.
    pub(super) fn show_file_links(&mut self, ui: &mut egui::Ui) {
        let file_ready = self.saved_is_current
            && self
                .last_saved_image
                .as_ref()
                .map(|p| p.exists())
                .unwrap_or(false);

        if widgets::link_button(ui, "Drukuj", file_ready) {
            if let Some(path) = self.last_saved_image.clone() {
                match platform::print_file(&path) {
                    Ok(()) => {
                        self.status = format!("Wysłano zadanie drukowania: {}", path.display())
                    }
                    Err(e) => self.status = e,
                }
            }
        }
        if widgets::link_button(ui, "Otwórz plik", file_ready) {
            if let Some(path) = self.last_saved_image.clone() {
                match platform::open_path(&path) {
                    Ok(()) => self.status = format!("Otwarto plik: {}", path.display()),
                    Err(e) => self.status = e,
                }
            }
        }
        if widgets::link_button(ui, "Otwórz folder", true) {
            let dir = self
                .last_saved_image
                .as_ref()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| self.resolved_output_dir());
            match platform::open_path(&dir) {
                Ok(()) => self.status = "Otwarto katalog z plikami.".to_string(),
                Err(e) => self.status = e,
            }
        }
    }

    pub(super) fn copy_data_to_clipboard(&mut self, ctx: &egui::Context, kind: FormatKind) {
        match self.forms.payload(kind) {
            Ok(payload) => {
                ctx.copy_text(payload);
                self.status = "Skopiowano dane QR do schowka.".to_string();
            }
            Err(e) => self.status = format!("Błąd: {e}"),
        }
    }

    pub(super) fn copy_image_to_clipboard(&mut self, kind: FormatKind) {
        let payload = match self.forms.payload(kind) {
            Ok(p) => p,
            Err(e) => {
                self.status = format!("Błąd: {e}");
                return;
            }
        };
        let img = match crate::render::render_image(&payload, &self.appearance) {
            Ok(img) => img,
            Err(e) => {
                self.status = format!("Błąd: {e}");
                return;
            }
        };

        let width = img.width() as usize;
        let height = img.height() as usize;
        let mut rgba = Vec::with_capacity(width * height * 4);
        for pixel in img.pixels() {
            rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
        }
        let image_data = arboard::ImageData {
            width,
            height,
            bytes: rgba.into(),
        };

        let Some(clipboard) = self.clipboard() else {
            return;
        };
        match clipboard.set_image(image_data) {
            Ok(()) => self.status = "Skopiowano obraz QR do schowka.".to_string(),
            Err(e) => self.status = format!("Błąd kopiowania obrazu do schowka: {e}"),
        }
    }
}
