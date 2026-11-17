use super::theme::ThemeMode;
use super::{widgets, QrGenApp};
use crate::output;
use crate::platform;
use eframe::egui;
use std::path::PathBuf;

impl QrGenApp {
    /// Plywajace okno "Ustawienia": zapis plikow i motyw. Wyglad kodu
    /// (kolory, korekcja bledow, rozmiar) jest przy podgladzie.
    pub(super) fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let mut open = self.settings_open;

        egui::Window::new("Ustawienia")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(460.0)
            // Pod przyciskiem "Ustawienia", przy prawej krawedzi okna.
            .anchor(egui::Align2::RIGHT_TOP, [-20.0, 64.0])
            .show(ctx, |ui| {
                widgets::section_title(ui, "Zapis plików");
                ui.label("Katalog zapisu (pusty = Dokumenty)");
                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 112.0).max(160.0);
                    ui.add_sized(
                        [width, 30.0],
                        egui::TextEdit::singleline(&mut self.output_dir)
                            .margin(egui::Margin::symmetric(8, 6)),
                    );
                    if ui.button("Przeglądaj…").clicked() {
                        let mut dialog = rfd::FileDialog::new();
                        let current = PathBuf::from(self.output_dir.trim());
                        if current.is_dir() {
                            dialog = dialog.set_directory(current);
                        } else if let Some(docs) = platform::documents_dir() {
                            dialog = dialog.set_directory(docs);
                        }
                        if let Some(dir) = dialog.pick_folder() {
                            self.output_dir = dir.display().to_string();
                        }
                    }
                });

                egui::Grid::new("settings_output_grid")
                    .num_columns(2)
                    .spacing([16.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("Nazwa pliku (poza vCard)");
                        ui.add_sized(
                            [160.0, 30.0],
                            egui::TextEdit::singleline(&mut self.file_stem)
                                .margin(egui::Margin::symmetric(8, 6)),
                        );
                        ui.end_row();

                        ui.label("Format obrazu");
                        let mut ext: &'static str = match self.image_ext.as_str() {
                            "png" => "png",
                            "bmp" => "bmp",
                            _ => "jpg",
                        };
                        if widgets::segmented(
                            ui,
                            &mut ext,
                            &[("jpg", "JPG"), ("png", "PNG"), ("bmp", "BMP")],
                        ) {
                            self.image_ext = ext.to_string();
                        }
                        ui.end_row();
                    });

                ui.checkbox(
                    &mut self.overwrite,
                    "Nadpisuj istniejące pliki (bez dopisku _2, _3, …)",
                );
                ui.label(
                    egui::RichText::new(format!(
                        "Pliki trafią do: {}",
                        output::resolve_full_path(&self.resolved_output_dir())
                    ))
                    .small()
                    .weak(),
                );

                ui.add_space(8.0);
                ui.separator();
                widgets::section_title(ui, "Motyw");
                widgets::segmented(
                    ui,
                    &mut self.theme_mode,
                    &[
                        (ThemeMode::Auto, "Auto (system)"),
                        (ThemeMode::Light, "Jasny"),
                        (ThemeMode::Dark, "Ciemny"),
                    ],
                );
            });

        if !open && self.settings_open {
            // Zamkniecie okna zapisuje ustawienia od razu.
            self.settings_open = false;
            self.persist_settings();
        } else {
            self.settings_open = open;
        }
    }
}
