use super::QrGenApp;
use crate::output;
use crate::platform;
use crate::render::{self, Appearance};
use eframe::egui;
use qrcode::EcLevel;
use std::path::PathBuf;

impl QrGenApp {
    /// Plywajace okno "Ustawienia": wyglad kodu + miejsce zapisu.
    pub(super) fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let mut open = self.settings_open;
        let mut changed = false;

        egui::Window::new("Ustawienia")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Rozmiar obrazu").strong());
                ui.add_space(4.0);
                egui::Grid::new("settings_size_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Docelowy bok obrazu:");
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.appearance.target_px)
                                    .range(0..=render::MAX_DIM)
                                    .speed(25)
                                    .suffix(" px"),
                            )
                            .changed();
                        ui.end_row();

                        ui.label("Skala (px na moduł):");
                        let scale_active = self.appearance.target_px == 0;
                        changed |= ui
                            .add_enabled(
                                scale_active,
                                egui::DragValue::new(&mut self.appearance.scale)
                                    .range(1..=render::MAX_SCALE),
                            )
                            .changed();
                        ui.end_row();
                    });
                ui.label(
                    egui::RichText::new(
                        "0 px = rozmiar wynika ze skali. Przy zadanym boku skala jest dobierana \
                         automatycznie; rzeczywisty rozmiar może być nieco mniejszy, bo każdy \
                         moduł kodu musi mieć całkowitą liczbę pikseli.",
                    )
                    .small()
                    .weak(),
                );

                ui.add_space(10.0);
                ui.separator();
                ui.label(egui::RichText::new("Wygląd kodu").strong());
                ui.add_space(4.0);
                egui::Grid::new("settings_appearance_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Ramka (moduły):");
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.appearance.border)
                                    .range(0..=render::MAX_BORDER),
                            )
                            .changed();
                        ui.end_row();

                        ui.label("Korekcja błędów:");
                        let ecc_response = egui::ComboBox::from_id_salt("settings_ecc")
                            .selected_text(render::ecc_to_letter(self.appearance.ecc))
                            .show_ui(ui, |ui| {
                                for level in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
                                    changed |= ui
                                        .selectable_value(
                                            &mut self.appearance.ecc,
                                            level,
                                            render::ecc_to_letter(level),
                                        )
                                        .changed();
                                }
                            });
                        changed |= ecc_response.response.changed();
                        ui.end_row();

                        ui.label("Kolor kodu:");
                        changed |= ui.color_edit_button_srgb(&mut self.appearance.fill).changed();
                        ui.end_row();

                        ui.label("Kolor tła:");
                        changed |= ui.color_edit_button_srgb(&mut self.appearance.back).changed();
                        ui.end_row();
                    });

                if ui.button("Przywróć domyślny wygląd").clicked() {
                    self.appearance = Appearance::default();
                    changed = true;
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label(egui::RichText::new("Zapis plików").strong());
                ui.add_space(4.0);

                ui.label("Katalog zapisu (pusty = Dokumenty):");
                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 102.0).max(160.0);
                    changed |= ui
                        .add_sized(
                            [width, 26.0],
                            egui::TextEdit::singleline(&mut self.output_dir),
                        )
                        .changed();
                    if ui.button("Przeglądaj...").clicked() {
                        let mut dialog = rfd::FileDialog::new();
                        let current = PathBuf::from(self.output_dir.trim());
                        if current.is_dir() {
                            dialog = dialog.set_directory(current);
                        } else if let Some(docs) = platform::documents_dir() {
                            dialog = dialog.set_directory(docs);
                        }
                        if let Some(dir) = dialog.pick_folder() {
                            self.output_dir = dir.display().to_string();
                            changed = true;
                        }
                    }
                });

                egui::Grid::new("settings_output_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Nazwa pliku (poza vCard):");
                        changed |= ui
                            .add_sized(
                                [140.0, 24.0],
                                egui::TextEdit::singleline(&mut self.file_stem),
                            )
                            .changed();
                        ui.end_row();

                        ui.label("Format obrazu:");
                        egui::ComboBox::from_id_salt("settings_ext")
                            .selected_text(self.image_ext.to_uppercase())
                            .show_ui(ui, |ui| {
                                for ext in ["jpg", "png", "bmp"] {
                                    changed |= ui
                                        .selectable_value(
                                            &mut self.image_ext,
                                            ext.to_string(),
                                            ext.to_uppercase(),
                                        )
                                        .changed();
                                }
                            });
                        ui.end_row();
                    });

                changed |= ui
                    .checkbox(
                        &mut self.overwrite,
                        "Nadpisuj istniejące pliki (bez dopisku _2, _3, ...)",
                    )
                    .changed();

                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Pliki trafią do: {}",
                        output::resolve_full_path(&self.resolved_output_dir())
                    ))
                    .small()
                    .weak(),
                );
            });

        if changed {
            self.invalidate_preview();
        }
        if !open && self.settings_open {
            // Zamkniecie okna zapisuje ustawienia od razu.
            self.settings_open = false;
            self.persist_settings();
        } else {
            self.settings_open = open;
        }
    }
}
