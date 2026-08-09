use super::{QrGenApp, View};
use crate::formats::FormatKind;
use eframe::egui;

const TILE_SIZE: [f32; 2] = [300.0, 92.0];
/// Wewnetrzny margines kafelka - tekst nie dotyka krawedzi.
const TILE_PADDING: f32 = 14.0;

/// Kafelek wyboru formatu: rysowany recznie. Lewa 1/3 kafelka to duza,
/// wysrodkowana ikona; pozostale 2/3 zajmuje tytul z podtytulem - dzieki temu
/// kazdy kafelek wyglada identycznie niezaleznie od dlugosci tekstu.
fn tile(ui: &mut egui::Ui, icon: &str, title: &str, subtitle: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(TILE_SIZE.into(), egui::Sense::click());
    let visuals = ui.style().interact(&response);

    ui.painter()
        .rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);
    ui.painter().rect_stroke(
        rect,
        visuals.corner_radius,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    let icon_zone_width = rect.width() / 3.0;
    let icon_center = egui::pos2(rect.left() + icon_zone_width / 2.0, rect.center().y);
    // Strefa tekstu: pozostale 2/3 kafelka pomniejszone o marginesy,
    // zeby tekst nigdy nie dotykal krawedzi przycisku.
    let text_zone_left = rect.left() + icon_zone_width;
    let text_center_x = text_zone_left + (rect.right() - TILE_PADDING - text_zone_left) / 2.0;

    // Rysowanie przyciete do obrysu kafelka - nawet zbyt dlugi tekst
    // nie wyjdzie poza przycisk.
    let painter = ui.painter().with_clip_rect(rect.shrink(2.0));

    let text_color = visuals.text_color();
    painter.text(
        icon_center,
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(40.0),
        text_color,
    );
    painter.text(
        egui::pos2(text_center_x, rect.center().y - 12.0),
        egui::Align2::CENTER_CENTER,
        title,
        egui::FontId::proportional(17.0),
        text_color,
    );
    painter.text(
        egui::pos2(text_center_x, rect.center().y + 13.0),
        egui::Align2::CENTER_CENTER,
        subtitle,
        egui::FontId::proportional(13.0),
        ui.visuals().weak_text_color(),
    );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

impl QrGenApp {
    /// Ekran startowy: wybor formatu kodowanych danych + wejscie do odczytu.
    pub(super) fn show_start(&mut self, ui: &mut egui::Ui) {
        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            ui.heading("Co chcesz zakodować?");
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Wybierz format danych, aby przejść do formularza.").weak(),
            );
            ui.add_space(20.0);

            let mut chosen: Option<FormatKind> = None;
            egui::Grid::new("start_tiles")
                .num_columns(2)
                .spacing([16.0, 16.0])
                .show(ui, |ui| {
                    for (i, kind) in FormatKind::ALL.iter().enumerate() {
                        if tile(ui, kind.icon(), kind.label(), kind.subtitle()).clicked() {
                            chosen = Some(*kind);
                        }
                        if i % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
            if let Some(kind) = chosen {
                self.switch_view(View::Form(kind));
            }

            ui.add_space(24.0);
            ui.separator();
            ui.add_space(12.0);
            if ui
                .add_sized(
                    [TILE_SIZE[0], 40.0],
                    egui::Button::new("🔍 Odczytaj kod QR"),
                )
                .clicked()
            {
                self.switch_view(View::Decode);
            }
            ui.label(
                egui::RichText::new("Wczytaj obraz z kodem QR, aby zobaczyć jego zawartość.")
                    .small()
                    .weak(),
            );
        });
    }
}
