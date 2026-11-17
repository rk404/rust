use super::{widgets, QrGenApp};
use crate::formats::FormatKind;
use crate::render::{self, Appearance};
use eframe::egui;
use qrcode::EcLevel;

/// Najwiekszy bok kwadratu podgladu (px).
const PREVIEW_MAX_SIDE: f32 = 280.0;

/// Przycisk koloru z ramka - bez niej biala probka tla ginie na bialej karcie.
fn color_swatch(ui: &mut egui::Ui, color: &mut [u8; 3]) -> bool {
    let response = ui.color_edit_button_srgb(color);
    ui.painter().rect_stroke(
        response.rect,
        4.0,
        egui::Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color),
        egui::StrokeKind::Inside,
    );
    response.changed()
}

impl QrGenApp {
    /// Prawa kolumna widoku Generuj: akcje przypiete na dole (zawsze
    /// widoczne), nad nimi przewijane karty z podgladem i wygladem kodu.
    pub(super) fn show_result_panel(&mut self, ui: &mut egui::Ui, kind: FormatKind) {
        egui::Panel::bottom("result_actions")
            .frame(egui::Frame::new().inner_margin(egui::Margin {
                top: 12,
                ..Default::default()
            }))
            .show_separator_line(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if widgets::primary_button(ui, "Zapisz", self.can_save(), 140.0) {
                        self.save();
                    }
                    if widgets::secondary_button(ui, "Kopiuj obraz", true) {
                        self.copy_image_to_clipboard(kind);
                    }
                    if widgets::secondary_button(ui, "Kopiuj dane", true) {
                        self.copy_data_to_clipboard(ui.ctx(), kind);
                    }
                });
                ui.label(
                    egui::RichText::new("Skrót: Ctrl+S (Cmd+S na macOS) zapisuje kod.")
                        .small()
                        .weak(),
                );
            });

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.show_preview_card(ui);
                ui.add_space(14.0);
                widgets::card(ui, |ui| self.show_appearance_panel(ui));
            });
    }

    fn show_preview_card(&mut self, ui: &mut egui::Ui) {
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                widgets::card_title(ui, "Podgląd");
                ui.label(
                    egui::RichText::new("na żywo, plik nie jest zapisany")
                        .small()
                        .weak(),
                );
            });
            ui.add_space(4.0);

            let avail = ui.available_width();
            let side = avail.min(PREVIEW_MAX_SIDE);
            let (row, _) = ui.allocate_exact_size(egui::vec2(avail, side), egui::Sense::hover());
            let rect = egui::Rect::from_center_size(row.center(), egui::vec2(side, side));
            let border = ui.visuals().widgets.noninteractive.bg_stroke.color;
            if let Some(tex) = &self.preview {
                ui.put(rect, egui::Image::new(tex).fit_to_exact_size(rect.size()));
            } else {
                let (text, is_error) = self.preview_message.clone().unwrap_or_else(|| {
                    ("Wpisz dane, aby zobaczyć podgląd kodu.".to_string(), false)
                });
                let color = if is_error {
                    ui.visuals().error_fg_color
                } else {
                    ui.visuals().weak_text_color()
                };
                ui.painter()
                    .rect_filled(rect, 6.0, ui.visuals().faint_bg_color);
                ui.put(
                    rect.shrink(16.0),
                    egui::Label::new(egui::RichText::new(text).color(color)).wrap(),
                );
            }
            ui.painter().rect_stroke(
                rect,
                6.0,
                egui::Stroke::new(1.0, border),
                egui::StrokeKind::Inside,
            );
            if let Some(info) = &self.preview_info {
                ui.label(egui::RichText::new(info).small().weak());
            }
        });
    }

    /// Wyglad kodu: korekcja bledow i kolory zawsze widoczne, rozmiar i ramka
    /// w zwijanej sekcji. Zmiany od razu odswiezaja podglad.
    fn show_appearance_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width());
        let mut changed = false;
        ui.horizontal(|ui| {
            widgets::card_title(ui, "Wygląd kodu");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::link_button(ui, "Przywróć domyślny", true) {
                    self.appearance = Appearance::default();
                    changed = true;
                }
            });
        });
        ui.add_space(4.0);

        egui::Grid::new("appearance_grid")
            .num_columns(2)
            .spacing([16.0, 10.0])
            .show(ui, |ui| {
                ui.label("Korekcja błędów").on_hover_text(
                    "Im wyższy poziom, tym więcej uszkodzeń kod wytrzyma \
                     (L ≈ 7%, M ≈ 15%, Q ≈ 25%, H ≈ 30%), ale tym jest gęstszy.",
                );
                changed |= widgets::segmented(
                    ui,
                    &mut self.appearance.ecc,
                    &[
                        (EcLevel::L, "L"),
                        (EcLevel::M, "M"),
                        (EcLevel::Q, "Q"),
                        (EcLevel::H, "H"),
                    ],
                );
                ui.end_row();

                ui.label("Kolory");
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("kod").small().weak());
                    changed |= color_swatch(ui, &mut self.appearance.fill);
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("tło").small().weak());
                    changed |= color_swatch(ui, &mut self.appearance.back);
                });
                ui.end_row();
            });

        egui::CollapsingHeader::new("Rozmiar i ramka").show(ui, |ui| {
            egui::Grid::new("size_grid")
                .num_columns(2)
                .spacing([16.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Docelowy bok obrazu");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut self.appearance.target_px)
                                .range(0..=render::MAX_DIM)
                                .speed(25)
                                .suffix(" px"),
                        )
                        .on_hover_text("0 = rozmiar wynika ze skali")
                        .changed();
                    ui.end_row();

                    ui.label("Skala (px na moduł)");
                    changed |= ui
                        .add_enabled(
                            self.appearance.target_px == 0,
                            egui::DragValue::new(&mut self.appearance.scale)
                                .range(1..=render::MAX_SCALE),
                        )
                        .changed();
                    ui.end_row();

                    ui.label("Ramka (moduły)");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut self.appearance.border)
                                .range(0..=render::MAX_BORDER),
                        )
                        .changed();
                    ui.end_row();
                });
            ui.label(
                egui::RichText::new(
                    "Przy zadanym boku skala jest dobierana automatycznie; plik może być \
                     nieco mniejszy, bo każdy moduł ma całkowitą liczbę pikseli.",
                )
                .small()
                .weak(),
            );
        });

        if changed {
            self.invalidate_preview();
        }
    }
}
