mod actions;
mod decode_view;
mod form_email;
mod form_link;
mod form_text;
mod form_vcard;
mod preview;
mod settings_window;
pub mod theme;
mod widgets;

use crate::cli::Job;
use crate::decode::DecodedQr;
use crate::formats::{FormatKind, FormsState};
use crate::output::{self, SaveRequest};
use crate::render::Appearance;
use crate::settings::{self, Settings, ThemeMode};
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Rozmiar modulu w podgladzie na zywo (px) - niezalezny od ustawien zapisu.
const PREVIEW_SCALE: u32 = 8;

/// Najwiekszy bok tekstury wysylanej do GPU. Wiele kart graficznych (i
/// sterownikow programowych) ma limit 8192 px, a duze obrazy zjadaja pamiec -
/// podglad i miniatury zawsze zmniejszamy do tego rozmiaru.
pub(crate) const MAX_TEXTURE_SIDE: u32 = 2048;

/// Zmniejsza obraz RGBA do `MAX_TEXTURE_SIDE` (z zachowaniem proporcji).
/// `nearest` zachowuje ostre moduly kodu QR w podgladzie.
pub(crate) fn texture_image(
    rgba: &[u8],
    width: usize,
    height: usize,
    nearest: bool,
) -> egui::ColorImage {
    let max_side = width.max(height) as u32;
    if max_side <= MAX_TEXTURE_SIDE {
        return egui::ColorImage::from_rgba_unmultiplied([width, height], rgba);
    }
    let ratio = MAX_TEXTURE_SIDE as f32 / max_side as f32;
    let new_w = ((width as f32 * ratio) as u32).max(1);
    let new_h = ((height as f32 * ratio) as u32).max(1);
    let Some(source) = image::RgbaImage::from_raw(width as u32, height as u32, rgba.to_vec())
    else {
        return egui::ColorImage::from_rgba_unmultiplied([width, height], rgba);
    };
    let filter = if nearest {
        image::imageops::FilterType::Nearest
    } else {
        image::imageops::FilterType::Triangle
    };
    let small = image::imageops::resize(&source, new_w, new_h, filter);
    egui::ColorImage::from_rgba_unmultiplied([new_w as usize, new_h as usize], small.as_raw())
}

/// Aktywny widok aplikacji.
#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Form(FormatKind),
    Decode,
}

/// Stan widoku odczytu kodu QR.
#[derive(Default)]
struct DecodeState {
    result: Option<DecodedQr>,
    /// Rozpoznany rodzaj tresci (vCard, e-mail, link, tekst).
    kind: Option<decode_view::DecodedKind>,
    error: Option<String>,
    /// Miniatura wczytanego obrazu.
    texture: Option<egui::TextureHandle>,
    /// Opis zrodla (nazwa pliku / "schowek").
    source: String,
}

/// Tryb pracy widoczny w pasku gornym.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Generate,
    Decode,
}

/// Rodzaj komunikatu w pasku stanu (decyduje o kolorze).
#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusKind {
    Info,
    Success,
    Error,
}

/// Szerokosc prawej kolumny (podglad, wyglad, akcje).
const RESULT_PANEL_WIDTH: f32 = 400.0;

pub struct QrGenApp {
    view: View,
    /// Ostatnio uzywany format - wraca po przejsciu z Odczytu do Generuj.
    last_format: FormatKind,
    forms: FormsState,
    appearance: Appearance,

    // Ustawienia zapisu (edytowane w oknie "Ustawienia").
    output_dir: String,
    file_stem: String,
    image_ext: String,
    overwrite: bool,
    /// Czy dla vCard generowac tez dokument Word.
    docx_enabled: bool,

    status: String,
    /// Podglad na zywo (tylko w pamieci - nie zapisuje niczego na dysk).
    preview: Option<egui::TextureHandle>,
    /// Podglad jest nieaktualny wzgledem formularza/wygladu i wymaga odswiezenia.
    preview_dirty: bool,
    /// Komunikat zamiast podgladu (brak danych lub blad kodowania).
    preview_message: Option<(String, bool)>,
    /// Informacja pod podgladem: wersja QR, liczba modulow, rozmiar pliku.
    preview_info: Option<String>,
    /// Czy ostatnio zapisany plik odpowiada biezacym danym.
    saved_is_current: bool,
    last_saved_image: Option<PathBuf>,
    last_saved_docx: Option<PathBuf>,

    settings_open: bool,
    /// Okno deweloperskie (--debug-ui): podglad ukladu i edycja stylu egui.
    debug_ui_open: bool,
    clipboard: Option<arboard::Clipboard>,
    decode: DecodeState,

    // Motyw.
    system_theme_light: bool,
    last_theme_check: Instant,
    theme_mode: ThemeMode,
    applied_light: Option<bool>,
}

pub fn run_gui(job: Job, settings: Settings) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 800.0])
            .with_min_inner_size([980.0, 640.0]),
        ..Default::default()
    };

    let result = eframe::run_native(
        "qrgen - generator kodów QR",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(QrGenApp::new(cc, job, settings)) as Box<dyn eframe::App>)
        }),
    );

    if let Err(e) = result {
        eprintln!("Nie udało się uruchomić interfejsu graficznego: {e}");
    }
}

impl QrGenApp {
    fn new(cc: &eframe::CreationContext<'_>, job: Job, settings: Settings) -> Self {
        let system_theme_light = Self::read_system_light(&cc.egui_ctx);
        // Bez jawnego formatu otwieramy ostatnio uzywany (domyslnie vCard).
        let initial_format = job
            .format
            .or(settings.last_format)
            .unwrap_or(FormatKind::VCard);

        let mut app = Self {
            view: View::Form(initial_format),
            last_format: initial_format,
            forms: job.forms.clone(),
            appearance: job.appearance,
            output_dir: settings.output_dir.clone(),
            file_stem: settings.file_stem.clone(),
            image_ext: settings.image_ext.clone(),
            overwrite: job.overwrite,
            docx_enabled: job.docx,
            status: String::new(),
            preview: None,
            preview_dirty: true,
            preview_message: None,
            preview_info: None,
            saved_is_current: false,
            last_saved_image: None,
            last_saved_docx: None,
            settings_open: false,
            debug_ui_open: job.debug_ui,
            clipboard: None,
            decode: DecodeState::default(),
            system_theme_light,
            last_theme_check: Instant::now(),
            theme_mode: settings.theme_mode,
            applied_light: None,
        };
        theme::apply_theme(&cc.egui_ctx, app.effective_light());
        app.applied_light = Some(app.effective_light());

        // Jawna sciezka -o: plik z rozszerzeniem ustawia katalog+nazwe+format,
        // sama sciezka katalogu ustawia tylko katalog zapisu.
        if let Some(explicit) = &job.output {
            app.apply_explicit_output(explicit);
        }

        // Dane przekazane z wiersza polecen / JSON: generujemy od razu.
        if job.content_provided {
            if let View::Form(kind) = app.view {
                if app.forms.has_data(kind) {
                    app.save();
                }
            }
        }
        app
    }

    fn apply_explicit_output(&mut self, explicit: &str) {
        let trimmed = explicit.trim();
        if trimmed.is_empty() {
            return;
        }
        // Sciezke wzgledna absolutyzujemy od razu - po zapisaniu w ustawieniach
        // nie moze zalezec od przyszlego katalogu roboczego.
        let path = {
            let p = PathBuf::from(trimmed);
            if p.is_absolute() {
                p
            } else {
                std::env::current_dir().map(|cwd| cwd.join(&p)).unwrap_or(p)
            }
        };
        match output::classify_output(&path.display().to_string()) {
            Some(output::OutputTarget::Dir(dir)) => self.output_dir = dir.display().to_string(),
            Some(output::OutputTarget::File { dir, stem, ext }) => {
                if let Some(dir) = dir {
                    self.output_dir = dir.display().to_string();
                }
                if !stem.is_empty() {
                    self.file_stem = stem;
                }
                if output::SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                    self.image_ext = ext;
                }
            }
            None => {}
        }
    }

    fn active_format(&self) -> Option<FormatKind> {
        match self.view {
            View::Form(kind) => Some(kind),
            _ => None,
        }
    }

    /// Katalog zapisu z uwzglednieniem domyslnego (Dokumenty).
    fn resolved_output_dir(&self) -> PathBuf {
        output::resolve_output_dir(&self.output_dir)
    }

    /// Rdzen nazwy pliku dla aktywnego formatu.
    fn current_stem(&self, kind: FormatKind) -> String {
        match kind {
            FormatKind::VCard => output::vcard_stem(&self.forms.vcard),
            _ => {
                let stem = output::sanitize_stem(&self.file_stem);
                if stem.is_empty() {
                    "qrcode".to_string()
                } else {
                    stem
                }
            }
        }
    }

    fn snapshot_settings(&self) -> Settings {
        let mut settings = Settings::default();
        settings.set_appearance(&self.appearance);
        settings.output_dir = self.output_dir.clone();
        settings.file_stem = self.file_stem.clone();
        settings.image_ext = self.image_ext.clone();
        settings.overwrite = self.overwrite;
        settings.theme_mode = self.theme_mode;
        settings.last_format = self.active_format();
        settings.forms = self.forms.clone();
        settings
    }

    fn persist_settings(&self) {
        if let Err(e) = settings::save(&self.snapshot_settings()) {
            eprintln!("Nie udało się zapisać ustawień: {e}");
        }
    }

    fn switch_view(&mut self, view: View) {
        if self.view != view {
            if let View::Form(kind) = view {
                self.last_format = kind;
            }
            self.view = view;
            self.invalidate_preview();
            self.status.clear();
            self.persist_settings();
        }
    }

    fn effective_light(&self) -> bool {
        match self.theme_mode {
            ThemeMode::Auto => self.system_theme_light,
            ThemeMode::Light => true,
            ThemeMode::Dark => false,
        }
    }

    /// Czy system uzywa jasnego motywu. Windows: odczyt rejestru (jak dotad);
    /// macOS i Linux: motyw zgłaszany przez system okienkowy (winit/egui).
    /// Gdy system motywu nie podaje, przyjmujemy jasny.
    fn read_system_light(ctx: &egui::Context) -> bool {
        #[cfg(windows)]
        {
            let _ = ctx;
            crate::platform::windows_uses_light_theme()
        }
        #[cfg(not(windows))]
        {
            ctx.system_theme()
                .map(|theme| theme == egui::Theme::Light)
                .unwrap_or(true)
        }
    }

    /// Sledzi motyw systemowy i aplikuje palete Solarized, gdy efektywny
    /// motyw sie zmieni. Rejestr Windows odpytujemy co 2 s; poza Windows
    /// odczyt jest tani (pamiec egui), wiec robimy go w kazdej klatce -
    /// dzieki temu ciemny motyw macOS dziala od pierwszej klatki.
    fn sync_theme(&mut self, ctx: &egui::Context) {
        let poll_due =
            !cfg!(windows) || self.last_theme_check.elapsed() >= Duration::from_secs(2);
        if self.theme_mode == ThemeMode::Auto && poll_due {
            self.system_theme_light = Self::read_system_light(ctx);
            self.last_theme_check = Instant::now();
        }

        let effective = self.effective_light();
        if self.applied_light != Some(effective) {
            theme::apply_theme(ctx, effective);
            self.applied_light = Some(effective);
        }

        ctx.request_repaint_after(Duration::from_secs(2));
    }

    /// Dane lub wyglad sie zmienily: podglad zostanie odswiezony w nastepnej
    /// klatce, a zapisany plik przestaje odpowiadac formularzowi.
    fn invalidate_preview(&mut self) {
        self.preview_dirty = true;
        self.saved_is_current = false;
        // Komunikat "Zapisano..." dotyczy poprzedniej wersji danych.
        if self.status.starts_with("Zapisano") {
            self.status.clear();
        }
    }

    /// Odswieza podglad na zywo, gdy dane sie zmienily. Renderuje w pamieci,
    /// ze stala, niewielka skala - niezaleznie od docelowego rozmiaru pliku,
    /// wiec nawet przy duzych ustawieniach wpisywanie pozostaje plynne.
    fn refresh_preview(&mut self, ctx: &egui::Context) {
        if !self.preview_dirty {
            return;
        }
        self.preview_dirty = false;
        let Some(kind) = self.active_format() else {
            self.preview = None;
            self.preview_message = None;
            self.preview_info = None;
            return;
        };
        let payload = match self.forms.payload(kind) {
            Ok(p) => p,
            Err(_) => {
                self.preview = None;
                self.preview_info = None;
                self.preview_message =
                    Some(("Wpisz dane, aby zobaczyć podgląd kodu.".to_string(), false));
                return;
            }
        };
        let preview_appearance = Appearance {
            scale: PREVIEW_SCALE,
            target_px: 0,
            ..self.appearance
        };
        match crate::render::render_image(&payload, &preview_appearance) {
            Ok(img) => {
                let rgba = image::DynamicImage::ImageRgb8(img.clone()).to_rgba8();
                let color_image = texture_image(
                    rgba.as_raw(),
                    img.width() as usize,
                    img.height() as usize,
                    true,
                );
                self.preview = Some(ctx.load_texture(
                    "qr_preview",
                    color_image,
                    egui::TextureOptions::NEAREST,
                ));
                self.preview_message = None;
                self.preview_info = Some(self.describe_code(&payload));
            }
            Err(e) => {
                self.preview = None;
                self.preview_info = None;
                self.preview_message = Some((format!("Nie można utworzyć kodu: {e}"), true));
            }
        }
    }

    /// Opis kodu pod podgladem: wersja QR, liczba modulow i rozmiar pliku.
    fn describe_code(&self, payload: &str) -> String {
        let Ok(code) =
            qrcode::QrCode::with_error_correction_level(payload.as_bytes(), self.appearance.ecc)
        else {
            return String::new();
        };
        let version = match code.version() {
            qrcode::Version::Normal(n) | qrcode::Version::Micro(n) => n,
        };
        let modules = code.width() as u32;
        let file = match crate::render::image_side(modules, &self.appearance) {
            Ok((_, side)) => format!("plik {side}×{side} px"),
            Err(e) => format!("plik: {e}"),
        };
        format!("Wersja {version} · {modules}×{modules} modułów · {file}")
    }

    /// Trwaly uchwyt do schowka - tworzony raz (na X11/Wayland natychmiastowe
    /// zwolnienie uchwytu gubi zawartosc schowka).
    fn clipboard(&mut self) -> Option<&mut arboard::Clipboard> {
        if self.clipboard.is_none() {
            match arboard::Clipboard::new() {
                Ok(c) => self.clipboard = Some(c),
                Err(e) => {
                    self.status = format!("Błąd dostępu do schowka: {e}");
                    return None;
                }
            }
        }
        self.clipboard.as_mut()
    }

    /// Zapisuje kod QR na dysk w docelowej rozdzielczosci (dla vCard takze .docx).
    /// Podglad na zywo jest niezalezny od zapisu.
    fn save(&mut self) {
        let Some(kind) = self.active_format() else {
            return;
        };
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

        let stem = self.current_stem(kind);
        let vcard_for_docx =
            (kind == FormatKind::VCard && self.docx_enabled).then(|| self.forms.vcard.clone());
        let request = SaveRequest {
            payload: &payload,
            appearance: &self.appearance,
            output: None,
            default_dir: self.resolved_output_dir(),
            stem: &stem,
            extension: &self.image_ext,
            docx: vcard_for_docx.as_ref(),
            overwrite: self.overwrite,
        };

        match output::save_rendered(&request, &img) {
            Ok(result) => {
                self.saved_is_current = true;
                self.status = match &result.docx_path {
                    Some(docx) => format!(
                        "Zapisano: {} oraz {}",
                        result.image_path.display(),
                        docx.display()
                    ),
                    None => format!("Zapisano: {}", result.image_path.display()),
                };
                self.last_saved_image = Some(result.image_path);
                self.last_saved_docx = result.docx_path;
                self.persist_settings();
            }
            Err(e) => self.status = format!("Błąd zapisu: {e}"),
        }
    }

    /// Obsluga plikow upuszczonych na okno - przelacza na widok odczytu.
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if dropped.is_empty() {
            return;
        }
        for file in dropped {
            if let Some(path) = file.path {
                self.view = View::Decode;
                self.decode_from_file(&path, ctx);
                break;
            }
        }
    }

    fn current_mode(&self) -> Mode {
        match self.view {
            View::Decode => Mode::Decode,
            View::Form(_) => Mode::Generate,
        }
    }

    /// Pasek gorny: nazwa, przelacznik trybu (Generuj / Odczyt), Ustawienia.
    fn show_top_bar(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::new()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(20, 10));
        egui::Panel::top("top_bar").frame(frame).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("qrgen").size(18.0).strong());
                ui.add_space(12.0);
                let mut mode = self.current_mode();
                if widgets::segmented(
                    ui,
                    &mut mode,
                    &[(Mode::Generate, "Generuj"), (Mode::Decode, "Odczyt")],
                ) {
                    match mode {
                        Mode::Generate => self.switch_view(View::Form(self.last_format)),
                        Mode::Decode => self.switch_view(View::Decode),
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new("Ustawienia")
                                .selected(self.settings_open)
                                .min_size(egui::vec2(0.0, 32.0)),
                        )
                        .clicked()
                    {
                        self.settings_open = !self.settings_open;
                        if !self.settings_open {
                            self.persist_settings();
                        }
                    }
                });
            });
        });
    }

    /// Rodzaj komunikatu wywiedziony z tresci (bledy zaczynaja sie od "Błąd",
    /// "Nie udało się" lub "Brak").
    fn status_kind(&self) -> StatusKind {
        let s = self.status.as_str();
        if s.starts_with("Błąd") || s.starts_with("Nie udało") || s.starts_with("Brak") {
            StatusKind::Error
        } else if s.is_empty() {
            StatusKind::Info
        } else {
            StatusKind::Success
        }
    }

    /// Pasek stanu na dole okna: komunikat po lewej, akcje na plikach po prawej.
    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::new()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(20, 8));
        let in_form = matches!(self.view, View::Form(_));
        egui::Panel::bottom("status_bar")
            .frame(frame)
            .show(ui, |ui| {
                let dark = ui.visuals().dark_mode;
                let weak = ui.visuals().weak_text_color();
                let (text, color) = match (self.status.is_empty(), self.status_kind()) {
                    (true, _) if in_form => (self.target_path_hint(), weak),
                    (true, _) => (String::new(), weak),
                    (false, StatusKind::Success) => {
                        (format!("✔ {}", self.status), theme::success(dark))
                    }
                    (false, StatusKind::Error) => {
                        (format!("⚠ {}", self.status), ui.visuals().error_fg_color)
                    }
                    (false, StatusKind::Info) => (self.status.clone(), weak),
                };
                egui::Sides::new().shrink_left().truncate().show(
                    ui,
                    |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(text).color(color)).truncate(),
                        );
                    },
                    |ui| {
                        if in_form {
                            self.show_file_links(ui);
                        }
                    },
                );
            });
    }

    /// Informacja, gdzie trafia zapisywane pliki (gdy brak innego komunikatu).
    fn target_path_hint(&self) -> String {
        let Some(kind) = self.active_format() else {
            return String::new();
        };
        let stem = self.current_stem(kind);
        let dir = output::resolve_full_path(&self.resolved_output_dir());
        let mut text = format!(
            "Pliki trafią do: {dir}{}{stem}.{}",
            std::path::MAIN_SEPARATOR,
            self.image_ext
        );
        if kind == FormatKind::VCard && self.docx_enabled {
            text.push_str(&format!(" + {stem}.docx"));
        }
        text
    }

    /// Czysci formularz aktywnego formatu.
    fn clear_current_form(&mut self, kind: FormatKind) {
        match kind {
            FormatKind::Text => self.forms.text = Default::default(),
            FormatKind::Link => self.forms.link = Default::default(),
            FormatKind::Email => self.forms.email = Default::default(),
            FormatKind::VCard => self.forms.vcard = Default::default(),
        }
        self.invalidate_preview();
        self.status = "Wyczyszczono formularz.".to_string();
    }

    /// Widok Generuj: lewa karta z formularzem, prawa kolumna z wynikiem.
    fn show_generate_view(&mut self, ui: &mut egui::Ui, kind: FormatKind) {
        let result_frame = egui::Frame::new()
            .fill(ui.visuals().panel_fill)
            .inner_margin(egui::Margin {
            left: 0,
            right: 20,
            top: 20,
            bottom: 20,
        });
        egui::Panel::right("result_panel")
            .exact_size(RESULT_PANEL_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(result_frame)
            .show(ui, |ui| self.show_result_panel(ui, kind));

        let form_frame = egui::Frame::new()
            .fill(ui.visuals().panel_fill)
            .inner_margin(20);
        egui::CentralPanel::default()
            .frame(form_frame)
            .show(ui, |ui| {
                widgets::card(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    ui.horizontal(|ui| {
                        let mut selected = kind;
                        let options: Vec<(FormatKind, &str)> =
                            FormatKind::ALL.iter().map(|k| (*k, k.label())).collect();
                        if widgets::segmented(ui, &mut selected, &options) {
                            self.switch_view(View::Form(selected));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Wyczyść").clicked() {
                                self.clear_current_form(kind);
                            }
                        });
                    });
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // Prawy margines na plywajacy pasek przewijania.
                            egui::Frame::new()
                                .inner_margin(egui::Margin {
                                    right: 14,
                                    ..Default::default()
                                })
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    match kind {
                                        FormatKind::Text => self.show_text_form(ui),
                                        FormatKind::Link => self.show_link_form(ui),
                                        FormatKind::Email => self.show_email_form(ui),
                                        FormatKind::VCard => self.show_vcard_form(ui),
                                    }
                                });
                        });
                });
            });
    }

    /// Skroty klawiszowe: Ctrl/Cmd+S zapisuje kod (gdy zapis jest mozliwy).
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S));
        if pressed && self.can_save() {
            self.save();
        }
    }

    /// Czy przycisk "Zapisz" jest aktywny: jest podglad, a plik jeszcze
    /// nie odpowiada biezacym danym.
    fn can_save(&self) -> bool {
        matches!(self.view, View::Form(_)) && self.preview.is_some() && !self.saved_is_current
    }

    /// Okno deweloperskie (--debug-ui): "podglad ukladu" (debug-on-hover
    /// podswietla prostokaty widgetow pod myszka) oraz pelna edycja stylu
    /// egui na zywo - wartosci dobrane suwakami przepisuje sie do kodu.
    fn show_debug_window(&mut self, ctx: &egui::Context) {
        if !self.debug_ui_open {
            return;
        }
        let mut open = self.debug_ui_open;
        egui::Window::new("🔧 Debug UI")
            .open(&mut open)
            .default_width(360.0)
            .vscroll(true)
            .show(ctx, |ui| {
                // Podglad ukladu istnieje w egui tylko w buildach debug
                // (cfg(debug_assertions)) - w release egui go wycina.
                #[cfg(debug_assertions)]
                {
                    let mut debug_on_hover = ui.ctx().debug_on_hover();
                    if ui
                        .checkbox(
                            &mut debug_on_hover,
                            "Podgląd układu (podświetlaj widget pod myszką)",
                        )
                        .changed()
                    {
                        ui.ctx().set_debug_on_hover(debug_on_hover);
                    }
                }
                #[cfg(not(debug_assertions))]
                ui.label(
                    egui::RichText::new(
                        "Podgląd układu jest dostępny tylko w buildzie deweloperskim \
                         (cargo run -- --debug-ui).",
                    )
                    .small()
                    .weak(),
                );
                ui.separator();
                ui.ctx().clone().settings_ui(ui);
            });
        self.debug_ui_open = open;
    }
}

impl eframe::App for QrGenApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.sync_theme(ui.ctx());
        self.handle_dropped_files(ui.ctx());
        self.refresh_preview(&ui.ctx().clone());
        self.handle_shortcuts(ui.ctx());

        let ctx = ui.ctx().clone();
        self.show_top_bar(ui);
        self.show_status_bar(ui);
        match self.view {
            View::Form(kind) => self.show_generate_view(ui, kind),
            View::Decode => self.show_decode_view(ui),
        }

        self.show_settings_window(&ctx);
        self.show_debug_window(&ctx);
    }
}

impl Drop for QrGenApp {
    fn drop(&mut self) {
        self.persist_settings();
    }
}
