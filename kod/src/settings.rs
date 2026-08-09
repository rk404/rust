use crate::formats::{FormatKind, FormsState};
use crate::output;
use crate::platform;
use crate::render::{self, Appearance};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Motyw kolorystyczny aplikacji: Auto = sledzi motyw systemowy Windows,
/// Light/Dark = wybor recznie ustawiony przez uzytkownika.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Auto,
    Light,
    Dark,
}

/// Ustawienia aplikacji zapisywane w settings.json.
/// Kazde pole ma wartosc domyslna - uszkodzone lub nieznane pola sa pomijane
/// pojedynczo, bez odrzucania calego pliku.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub scale: u32,
    pub border: u32,
    /// Docelowy bok obrazu w px (0 = rozmiar wynika ze skali).
    pub target_px: u32,
    pub ecc: String,
    pub fill: [u8; 3],
    pub back: [u8; 3],
    /// Katalog zapisu plikow; pusty = folder Dokumenty uzytkownika.
    pub output_dir: String,
    /// Rdzen nazwy pliku dla formatow innych niz vCard.
    pub file_stem: String,
    /// Rozszerzenie obrazu: jpg / png / bmp.
    pub image_ext: String,
    pub overwrite: bool,
    pub theme_mode: ThemeMode,
    /// Ostatnio uzywany format - podswietlany na ekranie startowym.
    pub last_format: Option<FormatKind>,
    /// Zawartosc formularzy przywracana miedzy uruchomieniami.
    pub forms: FormsState,
}

impl Default for Settings {
    fn default() -> Self {
        let ap = Appearance::default();
        Self {
            scale: ap.scale,
            border: ap.border,
            target_px: ap.target_px,
            ecc: render::ecc_to_letter(ap.ecc).to_string(),
            fill: ap.fill,
            back: ap.back,
            output_dir: String::new(),
            file_stem: "qrcode".to_string(),
            image_ext: output::DEFAULT_EXTENSION.to_string(),
            overwrite: false,
            theme_mode: ThemeMode::Auto,
            last_format: None,
            forms: FormsState::default(),
        }
    }
}

impl Settings {
    pub fn appearance(&self) -> Appearance {
        let defaults = Appearance::default();
        Appearance {
            scale: self.scale.clamp(1, render::MAX_SCALE),
            border: self.border.min(render::MAX_BORDER),
            target_px: self.target_px.min(render::MAX_DIM),
            ecc: render::ecc_from_letter(&self.ecc).unwrap_or(defaults.ecc),
            fill: self.fill,
            back: self.back,
        }
    }

    pub fn set_appearance(&mut self, ap: &Appearance) {
        self.scale = ap.scale;
        self.border = ap.border;
        self.target_px = ap.target_px;
        self.ecc = render::ecc_to_letter(ap.ecc).to_string();
        self.fill = ap.fill;
        self.back = ap.back;
    }

    /// Katalog zapisu z uwzglednieniem domyslnego (Dokumenty).
    pub fn resolved_output_dir(&self) -> PathBuf {
        if !self.output_dir.trim().is_empty() {
            return PathBuf::from(self.output_dir.trim());
        }
        platform::documents_dir().unwrap_or_else(|| PathBuf::from("."))
    }
}

fn settings_json_path() -> Option<PathBuf> {
    platform::config_dir().map(|dir| dir.join("settings.json"))
}

fn legacy_settings_path() -> Option<PathBuf> {
    platform::config_dir().map(|dir| dir.join("settings.txt"))
}

/// Wczytuje ustawienia. Kolejnosc: settings.json, potem (jednorazowa migracja)
/// stary settings.txt, na koncu wartosci domyslne. Bledne pola w JSON sa
/// pomijane pojedynczo.
pub fn load() -> Settings {
    if let Some(path) = settings_json_path() {
        if let Ok(text) = fs::read_to_string(&path) {
            return from_json_lenient(&text);
        }
    }
    if let Some(legacy) = legacy_settings_path() {
        if let Ok(text) = fs::read_to_string(&legacy) {
            return migrate_legacy(&text);
        }
    }
    Settings::default()
}

/// Parsuje JSON pole po polu - kazde uszkodzone pole wraca do domyslnego,
/// zamiast odrzucac caly plik.
fn from_json_lenient(text: &str) -> Settings {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return Settings::default();
    };
    // Najpierw szybka sciezka: caly plik poprawny.
    if let Ok(settings) = serde_json::from_value::<Settings>(value.clone()) {
        return settings;
    }
    let mut settings = Settings::default();
    let Some(map) = value.as_object() else {
        return settings;
    };

    fn field<T: serde::de::DeserializeOwned>(
        map: &serde_json::Map<String, serde_json::Value>,
        key: &str,
        target: &mut T,
    ) {
        if let Some(v) = map.get(key) {
            if let Ok(parsed) = serde_json::from_value::<T>(v.clone()) {
                *target = parsed;
            }
        }
    }

    field(map, "scale", &mut settings.scale);
    field(map, "border", &mut settings.border);
    field(map, "target_px", &mut settings.target_px);
    field(map, "ecc", &mut settings.ecc);
    field(map, "fill", &mut settings.fill);
    field(map, "back", &mut settings.back);
    field(map, "output_dir", &mut settings.output_dir);
    field(map, "file_stem", &mut settings.file_stem);
    field(map, "image_ext", &mut settings.image_ext);
    field(map, "overwrite", &mut settings.overwrite);
    field(map, "theme_mode", &mut settings.theme_mode);
    field(map, "last_format", &mut settings.last_format);
    field(map, "forms", &mut settings.forms);
    settings
}

pub fn save(settings: &Settings) -> Result<(), String> {
    let path = settings_json_path()
        .ok_or_else(|| "nie udalo sie wyznaczyc sciezki ustawien".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("nie udalo sie utworzyc katalogu ustawien: {e}"))?;
    }
    let text = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("nie udalo sie zserializowac ustawien: {e}"))?;
    fs::write(&path, text)
        .map_err(|e| format!("nie udalo sie zapisac ustawien do {:?}: {e}", path))
}

// --- migracja ze starego formatu settings.txt (klucz=wartosc, hex) ---

fn decode_hex(text: &str) -> Option<String> {
    if !text.len().is_multiple_of(2) || !text.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes = Vec::with_capacity(text.len() / 2);
    let mut i = 0;
    while i < text.len() {
        bytes.push(u8::from_str_radix(&text[i..i + 2], 16).ok()?);
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

fn migrate_legacy(text: &str) -> Settings {
    let mut settings = Settings::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "data_hex" => {
                if let Some(data) = decode_hex(value) {
                    settings.forms.text.data = data;
                }
            }
            "output_hex" => {
                if let Some(old_output) = decode_hex(value) {
                    let path = PathBuf::from(old_output);
                    if let Some(parent) = path.parent() {
                        if !parent.as_os_str().is_empty() {
                            settings.output_dir = parent.display().to_string();
                        }
                    }
                    if let Some(stem) = path.file_stem() {
                        settings.file_stem = stem.to_string_lossy().to_string();
                    }
                    // Rozszerzenia celowo nie przenosimy - nowa wersja domyslnie
                    // zapisuje .jpg (decyzja uzytkownika), stare instalacje
                    // mialy zawsze .png.
                }
            }
            "scale_text" => {
                if let Some(v) = decode_hex(value).and_then(|s| s.parse().ok()) {
                    settings.scale = v;
                }
            }
            "border_text" => {
                if let Some(v) = decode_hex(value).and_then(|s| s.parse().ok()) {
                    settings.border = v;
                }
            }
            "ecc" => {
                if render::ecc_from_letter(value).is_some() {
                    settings.ecc = value.to_string();
                }
            }
            "fill" => {
                if let Some(rgb) = parse_rgb(value) {
                    settings.fill = rgb;
                }
            }
            "back" => {
                if let Some(rgb) = parse_rgb(value) {
                    settings.back = rgb;
                }
            }
            _ => {}
        }
    }
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lenient_json_keeps_good_fields() {
        // "scale" uszkodzone (string zamiast liczby), reszta poprawna
        let text = r#"{ "scale": "abc", "border": 7, "ecc": "H", "output_dir": "/tmp/x" }"#;
        let s = from_json_lenient(text);
        assert_eq!(s.scale, Appearance::default().scale);
        assert_eq!(s.border, 7);
        assert_eq!(s.ecc, "H");
        assert_eq!(s.output_dir, "/tmp/x");
    }

    #[test]
    fn garbage_json_falls_back_to_defaults() {
        let s = from_json_lenient("to nie jest json");
        assert_eq!(s.border, Appearance::default().border);
    }

    #[test]
    fn legacy_migration_maps_fields() {
        // "dane" hex = 64616e65; output "/tmp/kody/qr.png" -> katalog + rdzen + ext
        let hex_output: String = "/tmp/kody/qr.png"
            .bytes()
            .map(|b| format!("{b:02x}"))
            .collect();
        let text = format!(
            "data_hex=64616e65\noutput_hex={hex_output}\nscale_text=3132\necc=Q\nfill=1,2,3\n"
        );
        let s = migrate_legacy(&text);
        assert_eq!(s.forms.text.data, "dane");
        assert_eq!(s.output_dir, "/tmp/kody");
        assert_eq!(s.file_stem, "qr");
        assert_eq!(s.image_ext, "jpg"); // rozszerzenie nie jest migrowane
        assert_eq!(s.scale, 12);
        assert_eq!(s.ecc, "Q");
        assert_eq!(s.fill, [1, 2, 3]);
    }

    #[test]
    fn roundtrip_serialization() {
        let mut s = Settings::default();
        s.forms.vcard.first_name = "Jan".to_string();
        s.theme_mode = ThemeMode::Dark;
        let json = serde_json::to_string(&s).unwrap();
        let back = from_json_lenient(&json);
        assert_eq!(back.forms.vcard.first_name, "Jan");
        assert_eq!(back.theme_mode, ThemeMode::Dark);
    }
}
