use crate::qr_logic::{self, Options};
use eframe::egui;
use qrcode::EcLevel;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{fs, path::PathBuf};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const SIDE_BY_SIDE_MIN_WIDTH: f32 = 680.0;
const PREVIEW_MIN_SIZE: f32 = 220.0;
const PREVIEW_MAX_SIZE: f32 = 520.0;
const PREVIEW_HEADER_HEIGHT: f32 = 26.0;

#[derive(Clone, Debug)]
struct StoredSettings {
    data: String,
    output: String,
    scale_text: String,
    border_text: String,
    ecc: String,
    fill: [u8; 3],
    back: [u8; 3],
}

impl Default for StoredSettings {
    fn default() -> Self {
        Self::from_options(&Options::default())
    }
}

impl StoredSettings {
    fn from_options(opts: &Options) -> Self {
        Self {
            data: opts.data.clone(),
            output: opts.output.clone(),
            scale_text: opts.scale.to_string(),
            border_text: opts.border.to_string(),
            ecc: qr_logic::ecc_to_letter(opts.ecc).to_string(),
            fill: opts.fill,
            back: opts.back,
        }
    }

    fn into_options(self) -> Options {
        Options {
            data: self.data,
            output: self.output,
            scale: self.scale_text.parse().unwrap_or(10).max(1),
            border: self.border_text.parse().unwrap_or(4),
            ecc: qr_logic::ecc_from_letter(&self.ecc).unwrap_or(EcLevel::M),
            fill: self.fill,
            back: self.back,
        }
    }
}

fn open_output_directory(output_path: &str) -> Result<(), String> {
    let path = PathBuf::from(output_path);
    let dir = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    };

    #[cfg(windows)]
    {
        Command::new("explorer")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Nie udało się otworzyć katalogu: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Nie udało się otworzyć katalogu: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Nie udało się otworzyć katalogu: {e}"))?;
    }

    Ok(())
}

fn settings_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("LOCALAPPDATA"))
            .map(PathBuf::from)
            .map(|base| base.join("qrgen_rust").join("settings.txt"))
    }

    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .map(|base| base.join("qrgen_rust").join("settings.txt"))
    }
}

// Folder Dokumenty uzytkownika - uzywany jako domyslna/podpowiadana lokalizacja
// zapisu wygenerowanego kodu QR.
fn documents_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .map(|base| base.join("Documents"))
    }

    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|base| base.join("Documents"))
    }
}

// Zamienia (mozliwie wzgledna) sciezke z pola "Plik wyjsciowy" na pelna, absolutna
// sciezke do wyswietlenia w interfejsie - tak, aby zawsze bylo widac, gdzie realnie
// zostanie zapisany plik.
fn resolve_full_path(path: &str) -> String {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        return candidate.display().to_string();
    }
    std::env::current_dir()
        .map(|dir| dir.join(&candidate).display().to_string())
        .unwrap_or_else(|_| candidate.display().to_string())
}

fn resolved_output_path(output_path: &str) -> PathBuf {
    let candidate = PathBuf::from(output_path);
    if candidate.is_absolute() {
        candidate
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(candidate)
    }
}

fn open_output_file(output_path: &str) -> Result<(), String> {
    let path = resolved_output_path(output_path);
    if !path.exists() {
        return Err("Plik wyjściowy nie istnieje. Najpierw wygeneruj kod QR.".to_string());
    }

    #[cfg(windows)]
    {
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Nie udało się otworzyć pliku: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Nie udało się otworzyć pliku: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Nie udało się otworzyć pliku: {e}"))?;
    }

    Ok(())
}

fn print_output_file(output_path: &str) -> Result<(), String> {
    let path = resolved_output_path(output_path);
    if !path.exists() {
        return Err("Plik wyjściowy nie istnieje. Najpierw wygeneruj kod QR.".to_string());
    }

    #[cfg(windows)]
    {
        let escaped_path = path.display().to_string().replace('"', "''");
        let script = format!(
            r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
$path = [System.IO.Path]::GetFullPath('{escaped_path}')
if (-not (Test-Path -LiteralPath $path)) {{ throw 'Plik nie istnieje.' }}
$img = [System.Drawing.Image]::FromFile($path)
try {{
    $doc = New-Object System.Drawing.Printing.PrintDocument
    $doc.DocumentName = [System.IO.Path]::GetFileName($path)
    $doc.Add_PrintPage({{
        param($sender, $e)
        $e.Graphics.DrawImage($img, 0, 0, $e.PageBounds.Width, $e.PageBounds.Height)
    }})

    $printDialog = New-Object System.Windows.Forms.PrintDialog
    $printDialog.UseEXDialog = $true
    $printDialog.Document = $doc
    if ($printDialog.ShowDialog() -ne [System.Windows.Forms.DialogResult]::OK) {{
        exit 0
    }}

    $doc.Print()
}} finally {{
    if ($img) {{ $img.Dispose() }}
}}
"#
        );

        let _child = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-NoLogo",
                "-Sta",
                "-WindowStyle",
                "Hidden",
                "-Command",
                &script,
            ])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("Nie udało się uruchomić drukowania: {e}"))?;

        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg("Preview")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Nie udało się uruchomić drukowania: {e}"))?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Nie udało się uruchomić drukowania: {e}"))?;

        Ok(())
    }
}

fn encode_hex(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for byte in text.as_bytes() {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{:02x}", byte);
    }
    out
}

fn decode_hex(text: &str) -> Option<String> {
    if text.len() % 2 != 0 || !text.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    let mut bytes = Vec::with_capacity(text.len() / 2);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let pair = [chars[i], chars[i + 1]];
        let pair_str: String = pair.iter().collect();
        let byte = u8::from_str_radix(&pair_str, 16).ok()?;
        bytes.push(byte);
        i += 2;
    }

    String::from_utf8(bytes).ok()
}

fn parse_rgb(value: &str) -> Option<[u8; 3]> {
    let mut parts = value.split(',');
    let r: u8 = parts.next()?.trim().parse().ok()?;
    let g: u8 = parts.next()?.trim().parse().ok()?;
    let b: u8 = parts.next()?.trim().parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some([r, g, b])
}

fn load_settings() -> Option<StoredSettings> {
    let path = settings_path()?;
    let text = fs::read_to_string(path).ok()?;
    let mut settings = StoredSettings::default();
    let mut saw_any = false;

    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "data_hex" => {
                settings.data = decode_hex(value.trim())?;
                saw_any = true;
            }
            "output_hex" => {
                settings.output = decode_hex(value.trim())?;
                saw_any = true;
            }
            "scale_text" => {
                settings.scale_text = decode_hex(value.trim())?;
                saw_any = true;
            }
            "border_text" => {
                settings.border_text = decode_hex(value.trim())?;
                saw_any = true;
            }
            "ecc" => {
                settings.ecc = value.trim().to_string();
                saw_any = true;
            }
            "fill" => {
                settings.fill = parse_rgb(value.trim())?;
                saw_any = true;
            }
            "back" => {
                settings.back = parse_rgb(value.trim())?;
                saw_any = true;
            }
            _ => {}
        }
    }

    saw_any.then_some(settings)
}

fn save_settings(settings: &StoredSettings) -> Result<(), String> {
    let path =
        settings_path().ok_or_else(|| "nie udalo sie wyznaczyc sciezki ustawien".to_string())?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("nie udalo sie utworzyc katalogu ustawien: {e}"))?;
    }

    let text = format!(
        "data_hex={}\noutput_hex={}\nscale_text={}\nborder_text={}\necc={}\nfill={},{},{}\nback={},{},{}\n",
        encode_hex(&settings.data),
        encode_hex(&settings.output),
        encode_hex(&settings.scale_text),
        encode_hex(&settings.border_text),
        settings.ecc,
        settings.fill[0],
        settings.fill[1],
        settings.fill[2],
        settings.back[0],
        settings.back[1],
        settings.back[2],
    );
    fs::write(&path, text).map_err(|e| format!("nie udalo sie zapisac ustawien do {:?}: {e}", path))
}

// Odczyt motywu jasny/ciemny bezposrednio z rejestru przez API Windows (RegGetValueW),
// bez odpalania procesu reg.exe - unika "migania" okna konsoli w tle.
#[cfg(windows)]
fn windows_uses_light_theme() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

    // RegGetValueW wymaga zakonczonych zerem ciagow UTF-16 (Windows Wide String).
    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let value: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();

    let mut data: u32 = 1;
    let mut size: u32 = std::mem::size_of::<u32>() as u32;

    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut _),
            Some(&mut size),
        )
    };

    if status.is_ok() {
        data != 0
    } else {
        true
    }
}

#[cfg(not(windows))]
fn windows_uses_light_theme() -> bool {
    true
}

fn apply_theme(ctx: &egui::Context, light: bool) {
    ctx.set_visuals(solarized_visuals(light));
}

// Paleta kolorow Solarized (jasna/ciemna odmiana) zaaplikowana na egui::Visuals.
fn solarized_visuals(light: bool) -> egui::Visuals {
    const BASE03: egui::Color32 = egui::Color32::from_rgb(0x00, 0x2b, 0x36);
    const BASE02: egui::Color32 = egui::Color32::from_rgb(0x07, 0x36, 0x42);
    const BASE01: egui::Color32 = egui::Color32::from_rgb(0x58, 0x6e, 0x75);
    const BASE00: egui::Color32 = egui::Color32::from_rgb(0x65, 0x7b, 0x83);
    const BASE0: egui::Color32 = egui::Color32::from_rgb(0x83, 0x94, 0x96);
    const BASE1: egui::Color32 = egui::Color32::from_rgb(0x93, 0xa1, 0xa1);
    const BASE2: egui::Color32 = egui::Color32::from_rgb(0xee, 0xe8, 0xd5);
    const BASE3: egui::Color32 = egui::Color32::from_rgb(0xfd, 0xf6, 0xe3);
    const BLUE: egui::Color32 = egui::Color32::from_rgb(0x26, 0x8b, 0xd2);
    const ORANGE: egui::Color32 = egui::Color32::from_rgb(0xcb, 0x4b, 0x16);
    const RED: egui::Color32 = egui::Color32::from_rgb(0xdc, 0x32, 0x2f);

    let mut visuals = if light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };

    let (bg, widget_bg, text_primary, text_secondary, extreme_bg, active_text) = if light {
        (BASE3, BASE2, BASE00, BASE01, egui::Color32::WHITE, BASE3)
    } else {
        (BASE03, BASE02, BASE0, BASE1, BASE02, BASE03)
    };
    let hover_bg = BLUE.gamma_multiply(0.28);
    let active_bg = BLUE.gamma_multiply(0.55);

    visuals.override_text_color = Some(text_primary);
    visuals.hyperlink_color = BLUE;
    visuals.warn_fg_color = ORANGE;
    visuals.error_fg_color = RED;
    visuals.window_fill = bg;
    visuals.panel_fill = bg;
    visuals.faint_bg_color = widget_bg;
    visuals.extreme_bg_color = extreme_bg;
    visuals.code_bg_color = widget_bg;

    visuals.selection.bg_fill = active_bg;
    visuals.selection.stroke = egui::Stroke::new(1.0, BLUE);

    visuals.widgets.noninteractive.bg_fill = bg;
    visuals.widgets.noninteractive.weak_bg_fill = bg;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, widget_bg);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text_secondary);

    visuals.widgets.inactive.bg_fill = widget_bg;
    visuals.widgets.inactive.weak_bg_fill = widget_bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, widget_bg);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_primary);

    visuals.widgets.hovered.bg_fill = hover_bg;
    visuals.widgets.hovered.weak_bg_fill = hover_bg;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, BLUE);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, text_primary);

    visuals.widgets.active.bg_fill = active_bg;
    visuals.widgets.active.weak_bg_fill = active_bg;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, BLUE);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, active_text);

    visuals.widgets.open = visuals.widgets.active;

    visuals
}

pub fn run_gui(opts: Options, has_data: bool) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 720.0])
            .with_min_inner_size([900.0, 560.0]),
        ..Default::default()
    };

    let result = eframe::run_native(
        "qrgen - generator kodow QR",
        native_options,
        Box::new(move |cc| Ok(Box::new(QrGenApp::new(cc, opts, has_data)) as Box<dyn eframe::App>)),
    );

    if let Err(e) = result {
        eprintln!("Nie udalo sie uruchomic interfejsu graficznego: {e}");
    }
}

struct QrGenApp {
    data: String,
    output: String,
    scale_text: String,
    border_text: String,
    ecc: EcLevel,
    fill: [u8; 3],
    back: [u8; 3],
    status: String,
    preview: Option<egui::TextureHandle>,
    preview_is_current: bool,
    system_theme_light: bool,
    last_theme_check: Instant,
    theme_mode: ThemeMode,
    applied_light: Option<bool>,
    form_content_height: f32,
}

// Motyw kolorystyczny aplikacji: Auto = sledzi motyw systemowy Windows,
// Light/Dark = wybor recznie ustawiony przez uzytkownika (nadpisuje Auto).
#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Auto,
    Light,
    Dark,
}

impl QrGenApp {
    fn new(cc: &eframe::CreationContext<'_>, opts: Options, has_data: bool) -> Self {
        let system_theme_light = windows_uses_light_theme();
        apply_theme(&cc.egui_ctx, system_theme_light);

        let mut app = if let Some(saved) = load_settings() {
            let opts = saved.into_options();
            Self {
                data: opts.data.clone(),
                output: opts.output.clone(),
                scale_text: opts.scale.to_string(),
                border_text: opts.border.to_string(),
                ecc: opts.ecc,
                fill: opts.fill,
                back: opts.back,
                status: "Wczytano zapisane ustawienia uzytkownika.".to_string(),
                preview: None,
                preview_is_current: false,
                system_theme_light,
                last_theme_check: Instant::now(),
                theme_mode: ThemeMode::Auto,
                applied_light: Some(system_theme_light),
                form_content_height: 0.0,
            }
        } else {
            let defaults = Options::default();
            // Domyslnie podpowiadamy zapis w folderze Dokumenty uzytkownika,
            // zamiast wzglednej sciezki w katalogu roboczym aplikacji.
            let output = documents_dir()
                .map(|dir| dir.join(&defaults.output).display().to_string())
                .unwrap_or(defaults.output);
            Self {
                data: defaults.data,
                output,
                scale_text: defaults.scale.to_string(),
                border_text: defaults.border.to_string(),
                ecc: defaults.ecc,
                fill: defaults.fill,
                back: defaults.back,
                status: String::new(),
                preview: None,
                preview_is_current: false,
                system_theme_light,
                last_theme_check: Instant::now(),
                theme_mode: ThemeMode::Auto,
                applied_light: Some(system_theme_light),
                form_content_height: 0.0,
            }
        };

        app.apply_cli_options(opts);

        // Jesli aplikacja zostala uruchomiona z danymi z wiersza polecen,
        // od razu generujemy podglad i zapisujemy plik.
        if has_data {
            app.generate(&cc.egui_ctx);
        }
        app
    }

    fn effective_light(&self) -> bool {
        match self.theme_mode {
            ThemeMode::Auto => self.system_theme_light,
            ThemeMode::Light => true,
            ThemeMode::Dark => false,
        }
    }

    // Sledzi motyw systemowy (co 2s), gdy uzytkownik nie wybral recznie Light/Dark,
    // i aplikuje paleta Solarized za kazdym razem, gdy efektywny motyw sie zmieni.
    fn sync_theme(&mut self, ctx: &egui::Context) {
        if self.theme_mode == ThemeMode::Auto
            && self.last_theme_check.elapsed() >= Duration::from_secs(2)
        {
            self.system_theme_light = windows_uses_light_theme();
            self.last_theme_check = Instant::now();
        }

        let effective = self.effective_light();
        if self.applied_light != Some(effective) {
            apply_theme(ctx, effective);
            self.applied_light = Some(effective);
        }

        ctx.request_repaint_after(Duration::from_secs(2));
    }

    fn apply_cli_options(&mut self, opts: Options) {
        let defaults = Options::default();

        if !opts.data.is_empty() {
            self.data = opts.data;
        }
        if opts.output != defaults.output {
            self.output = opts.output;
        }
        if opts.scale != defaults.scale {
            self.scale_text = opts.scale.to_string();
        }
        if opts.border != defaults.border {
            self.border_text = opts.border.to_string();
        }
        if opts.ecc != defaults.ecc {
            self.ecc = opts.ecc;
        }
        if opts.fill != defaults.fill {
            self.fill = opts.fill;
        }
        if opts.back != defaults.back {
            self.back = opts.back;
        }
    }

    fn settings_snapshot(&self) -> StoredSettings {
        StoredSettings {
            data: self.data.clone(),
            output: self.output.clone(),
            scale_text: self.scale_text.clone(),
            border_text: self.border_text.clone(),
            ecc: qr_logic::ecc_to_letter(self.ecc).to_string(),
            fill: self.fill,
            back: self.back,
        }
    }

    fn current_options(&self) -> Options {
        Options {
            data: self.data.clone(),
            output: self.output.clone(),
            scale: self.scale_text.parse().unwrap_or(10).max(1),
            border: self.border_text.parse().unwrap_or(4),
            ecc: self.ecc,
            fill: self.fill,
            back: self.back,
        }
    }

    fn invalidate_preview(&mut self) {
        self.preview = None;
        self.preview_is_current = false;
    }

    fn copy_data_to_clipboard(&mut self, ctx: &egui::Context) {
        if self.data.trim().is_empty() {
            self.status = "Brak danych do skopiowania.".to_string();
            return;
        }

        ctx.copy_text(self.data.clone());
        self.status = "Skopiowano dane QR do schowka.".to_string();
    }

    fn copy_image_to_clipboard(&mut self) {
        let opts = self.current_options();

        let img = match qr_logic::render_image(&opts) {
            Ok(img) => img,
            Err(e) => {
                self.status = format!("Blad: {e}");
                return;
            }
        };

        let width = img.width() as usize;
        let height = img.height() as usize;
        let mut rgba = Vec::with_capacity(width * height * 4);
        for pixel in img.pixels() {
            rgba.push(pixel[0]);
            rgba.push(pixel[1]);
            rgba.push(pixel[2]);
            rgba.push(255);
        }

        let image_data = arboard::ImageData {
            width,
            height,
            bytes: rgba.into(),
        };

        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_image(image_data)) {
            Ok(()) => self.status = "Skopiowano obraz QR do schowka.".to_string(),
            Err(e) => self.status = format!("Blad kopiowania obrazu do schowka: {e}"),
        }
    }

    fn generate(&mut self, ctx: &egui::Context) {
        let opts = self.current_options();

        match qr_logic::render_image(&opts) {
            Ok(img) => {
                let size = [img.width() as usize, img.height() as usize];
                let color_image = egui::ColorImage::from_rgb(size, img.as_raw());
                let preview_texture = ctx.load_texture(
                    "qr_preview",
                    color_image,
                    egui::TextureOptions::NEAREST,
                );

                match img.save(&opts.output) {
                    Ok(()) => {
                        self.preview = Some(preview_texture);
                        self.preview_is_current = true;
                        let full_path = fs::canonicalize(&opts.output)
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|_| resolve_full_path(&opts.output));
                        self.status = format!("Zapisano: {full_path}");
                    }
                    Err(e) => {
                        self.invalidate_preview();
                        self.status = format!("Blad zapisu pliku: {e}");
                    }
                }
            }
            Err(e) => {
                self.invalidate_preview();
                self.status = format!("Blad: {e}");
            }
        }
    }

    // Przywraca domyslne wartosci wyglad kodu (skala, ramka, ECC, kolory).
    fn reset_appearance(&mut self) {
        let defaults = Options::default();
        self.scale_text = defaults.scale.to_string();
        self.border_text = defaults.border.to_string();
        self.ecc = defaults.ecc;
        self.fill = defaults.fill;
        self.back = defaults.back;
        self.invalidate_preview();
        self.status = "Przywrocono domyslny wyglad kodu.".to_string();
    }

    fn browse_output(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .add_filter("Bitmapa (BMP)", &["bmp"])
            .add_filter("Wszystkie pliki", &["*"]);

        // Podpowiadamy katalog startowy: istniejacy katalog z pola "Plik wyjsciowy",
        // a w razie jego braku - folder Dokumenty uzytkownika.
        let existing_dir = PathBuf::from(&self.output)
            .parent()
            .filter(|p| p.is_dir())
            .map(PathBuf::from);
        if let Some(dir) = existing_dir.or_else(documents_dir) {
            dialog = dialog.set_directory(dir);
        }

        if !self.output.is_empty() {
            let file_name = PathBuf::from(&self.output)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| self.output.clone());
            dialog = dialog.set_file_name(file_name);
        }
        if let Some(path) = dialog.save_file() {
            self.output = path.display().to_string();
            self.invalidate_preview();
        }
    }

    fn show_form(&mut self, ui: &mut egui::Ui) {
        let content_top = ui.cursor().top();

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Dane do zakodowania:");
                let clear_width = 70.0;
                ui.add_space((ui.available_width() - clear_width).max(0.0));
                if ui
                    .add_sized([clear_width, 20.0], egui::Button::new("Wyczysc"))
                    .clicked()
                {
                    self.data.clear();
                    self.invalidate_preview();
                    self.status = "Wyczyszczono dane do zakodowania.".to_string();
                }
            });
            let width = ui.available_width().max(240.0);
            let response = ui.add_sized(
                [width, 84.0],
                egui::TextEdit::multiline(&mut self.data).desired_rows(3),
            );
            if response.changed() {
                self.invalidate_preview();
            }
        });

        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label("Plik wyjsciowy:");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let output_width = (ui.available_width() - 102.0).max(120.0);
                let output_response = ui.add_sized(
                    [output_width, 26.0],
                    egui::TextEdit::singleline(&mut self.output),
                );
                if output_response.changed() {
                    self.invalidate_preview();
                }
                if ui.button("Przegladaj...").clicked() {
                    self.browse_output();
                }
            });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("Zapis do: {}", resolve_full_path(&self.output)))
                    .small()
                    .weak(),
            );
        });

        ui.add_space(10.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Wyglad kodu:");
                let clear_width = 70.0;
                ui.add_space((ui.available_width() - clear_width).max(0.0));
                if ui
                    .add_sized([clear_width, 20.0], egui::Button::new("Wyczysc"))
                    .clicked()
                {
                    self.reset_appearance();
                }
            });
            ui.add_space(4.0);
            egui::Grid::new("wyglad_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Skala:");
                    let scale_response = ui.add_sized(
                        [120.0, 24.0],
                        egui::TextEdit::singleline(&mut self.scale_text),
                    );
                    if scale_response.changed() {
                        self.invalidate_preview();
                    }
                    ui.end_row();

                    ui.label("Ramka:");
                    let border_response = ui.add_sized(
                        [120.0, 24.0],
                        egui::TextEdit::singleline(&mut self.border_text),
                    );
                    if border_response.changed() {
                        self.invalidate_preview();
                    }
                    ui.end_row();

                    ui.label("Korekcja bledow:");
                    let ecc_response = egui::ComboBox::from_id_salt("ecc_combo")
                        .selected_text(qr_logic::ecc_to_letter(self.ecc))
                        .show_ui(ui, |ui| {
                            for level in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
                                ui.selectable_value(
                                    &mut self.ecc,
                                    level,
                                    qr_logic::ecc_to_letter(level),
                                );
                            }
                        });
                    if ecc_response.response.changed() {
                        self.invalidate_preview();
                    }
                    ui.end_row();

                    ui.label("Kolor kodu:");
                    let fill_response = ui.color_edit_button_srgb(&mut self.fill);
                    if fill_response.changed() {
                        self.invalidate_preview();
                    }
                    ui.end_row();

                    ui.label("Kolor tla:");
                    let back_response = ui.color_edit_button_srgb(&mut self.back);
                    if back_response.changed() {
                        self.invalidate_preview();
                    }
                    ui.end_row();
                });
        });

        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label("Akcje:");
            ui.add_space(4.0);
            egui::Grid::new("action_buttons")
                .num_columns(3)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    let button_width = 138.0;
                    let button_height = 34.0;

                    if ui
                        .add_sized(
                            [button_width, button_height],
                            egui::Button::new(egui::RichText::new("Generuj").size(15.0)),
                        )
                        .clicked()
                    {
                        self.generate(ui.ctx());
                    }

                    if ui
                        .add_sized(
                            [button_width, button_height],
                            egui::Button::new(egui::RichText::new("Kopiuj dane").size(15.0)),
                        )
                        .clicked()
                    {
                        self.copy_data_to_clipboard(ui.ctx());
                    }

                    if ui
                        .add_sized(
                            [button_width, button_height],
                            egui::Button::new(egui::RichText::new("Kopiuj obraz").size(15.0)),
                        )
                        .clicked()
                    {
                        self.copy_image_to_clipboard();
                    }

                    ui.end_row();

                    if ui
                        .add_sized(
                            [button_width, button_height],
                            egui::Button::new(egui::RichText::new("Otwórz folder").size(15.0)),
                        )
                        .clicked()
                    {
                        match open_output_directory(&self.output) {
                            Ok(()) => self.status = "Otwarto katalog z plikiem.".to_string(),
                            Err(e) => self.status = e,
                        }
                    }

                    let can_open_or_print = self.preview_is_current;

                    let open_button = egui::Button::new(egui::RichText::new("Otwórz plik").size(15.0))
                        .min_size(egui::vec2(button_width, button_height));
                    if ui
                        .add_enabled(can_open_or_print, open_button)
                        .clicked()
                    {
                        match open_output_file(&self.output) {
                            Ok(()) => {
                                self.status =
                                    format!("Otwarto plik: {}", resolve_full_path(&self.output))
                            }
                            Err(e) => self.status = e,
                        }
                    }

                    let print_button = egui::Button::new(egui::RichText::new("Drukuj").size(15.0))
                        .min_size(egui::vec2(button_width, button_height));
                    if ui
                        .add_enabled(can_open_or_print, print_button)
                        .clicked()
                    {
                        self.generate(ui.ctx());
                        match print_output_file(&self.output) {
                            Ok(()) => {
                                self.status = format!(
                                    "Wysłano zadanie drukowania: {}",
                                    resolve_full_path(&self.output)
                                )
                            }
                            Err(e) => self.status = e,
                        }
                    }
                });
        });

        self.form_content_height = (ui.cursor().top() - content_top).max(0.0);

        ui.add_space(12.0);
        ui.label("Status:");
        ui.label(&self.status);
    }

    fn show_preview(&mut self, ui: &mut egui::Ui) {
        let target_height = if self.form_content_height > 0.0 {
            (self.form_content_height - PREVIEW_HEADER_HEIGHT).max(PREVIEW_MIN_SIZE)
        } else {
            PREVIEW_MAX_SIZE
        };

        let side = ui
            .available_width()
            .min(target_height)
            .clamp(PREVIEW_MIN_SIZE, PREVIEW_MAX_SIZE);

        // Naglowek jest ograniczony do tej samej szerokosci "side" co kwadrat
        // podgladu, dzieki czemu przycisk "Wyczysc" zawsze dokuje sie do prawego
        // brzegu ramki podgladu, a nie calej (szerszej) kolumny formularza.
        ui.allocate_ui_with_layout(
            egui::vec2(side, PREVIEW_HEADER_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label("Podglad:");
                let clear_width = 70.0;
                ui.add_space((ui.available_width() - clear_width).max(0.0));
                if ui
                    .add_sized([clear_width, 20.0], egui::Button::new("Wyczysc"))
                    .clicked()
                {
                    self.invalidate_preview();
                    self.status = "Wyczyszczono podglad wygenerowanego kodu.".to_string();
                }
            },
        );

        let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
            egui::StrokeKind::Inside,
        );

        if let Some(tex) = &self.preview {
            ui.put(rect, egui::Image::new(tex).fit_to_exact_size(rect.size()));
        } else {
            ui.put(
                rect,
                egui::Label::new("Wygeneruj kod, aby zobaczyc podglad.").wrap(),
            );
        }
    }
}

impl eframe::App for QrGenApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.sync_theme(ui.ctx());

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Generator kodow QR");
                    let combo_width = 130.0;
                    ui.add_space((ui.available_width() - combo_width).max(0.0));
                    egui::ComboBox::from_id_salt("theme_mode_combo")
                        .width(combo_width)
                        .selected_text(match self.theme_mode {
                            ThemeMode::Auto => "Motyw: Auto",
                            ThemeMode::Light => "Motyw: Jasny",
                            ThemeMode::Dark => "Motyw: Ciemny",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.theme_mode,
                                ThemeMode::Auto,
                                "Auto (system)",
                            );
                            ui.selectable_value(&mut self.theme_mode, ThemeMode::Light, "Jasny");
                            ui.selectable_value(&mut self.theme_mode, ThemeMode::Dark, "Ciemny");
                        });
                });
                ui.add_space(6.0);

                if ui.available_width() >= SIDE_BY_SIDE_MIN_WIDTH {
                    ui.columns(2, |columns| {
                        self.show_form(&mut columns[0]);
                        self.show_preview(&mut columns[1]);
                    });
                } else {
                    self.show_form(ui);
                    ui.add_space(18.0);
                    self.show_preview(ui);
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_output_path_keeps_absolute_path() {
        let absolute = PathBuf::from(r"C:\temp\qr.png");
        assert_eq!(
            resolved_output_path(&absolute.display().to_string()),
            absolute
        );
    }

    #[test]
    fn resolved_output_path_joins_relative_path() {
        let relative = "qr.png";
        let resolved = resolved_output_path(relative);
        assert!(resolved.ends_with("qr.png"));
    }
}

impl Drop for QrGenApp {
    fn drop(&mut self) {
        if let Err(err) = save_settings(&self.settings_snapshot()) {
            eprintln!("Nie udalo sie zapisac ustawien GUI: {err}");
        }
    }
}
