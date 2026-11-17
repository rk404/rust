//! Czcionka systemowa: zamiast domyslnej (cienkiej) czcionki egui interfejs
//! uzywa czcionki systemu (Segoe UI, San Francisco, Noto Sans ...), czytanej
//! prosto z dysku - bez dodatkowych zaleznosci. Domyslne czcionki egui
//! zostaja jako zapasowe (symbole, emoji, brakujace znaki).

use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Nazwa, pod ktora czcionka systemowa jest rejestrowana w egui.
const FONT_NAME: &str = "qrgen-system";
/// Zmienna srodowiskowa z reczna sciezka do czcionki (np. do testow).
const FONT_OVERRIDE_ENV: &str = "QRGEN_FONT";
/// Rozsadny limit rozmiaru pliku czcionki (rozne wersje CJK sa wieksze).
const MAX_FONT_BYTES: u64 = 40 * 1024 * 1024;

/// Kandydaci w kolejnosci preferencji. Wszystkie maja polskie znaki.
fn candidates() -> Vec<PathBuf> {
    let mut list: Vec<PathBuf> = Vec::new();
    if let Some(custom) = std::env::var_os(FONT_OVERRIDE_ENV) {
        list.push(PathBuf::from(custom));
    }

    #[cfg(windows)]
    {
        let windir = std::env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
        for name in ["segoeui.ttf", "arial.ttf", "tahoma.ttf"] {
            list.push(windir.join("Fonts").join(name));
        }
    }

    #[cfg(target_os = "macos")]
    {
        for path in [
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
        ] {
            list.push(PathBuf::from(path));
        }
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        for path in [
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/google-noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/cantarell/Cantarell-Regular.otf",
            "/usr/share/fonts/opentype/cantarell/Cantarell-Regular.otf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        ] {
            list.push(PathBuf::from(path));
        }
    }

    list
}

/// Czy bajty wygladaja na plik czcionki (TrueType, OpenType, kolekcja).
/// Chroni przed przekazaniem do egui losowego pliku, ktory bylby przyczyna paniki.
fn looks_like_font(bytes: &[u8]) -> bool {
    matches!(
        bytes.get(..4),
        Some([0x00, 0x01, 0x00, 0x00]) | Some(b"OTTO") | Some(b"true") | Some(b"ttcf")
    )
}

/// Wczytuje plik czcionki; None, gdy go nie ma, jest za duzy lub nie wyglada
/// na czcionke.
fn read_font(path: &Path) -> Option<Vec<u8>> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_FONT_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    looks_like_font(&bytes).then_some(bytes)
}

/// Ustawia czcionke systemowa jako pierwsza w rodzinie proporcjonalnej.
/// Zwraca sciezke uzytej czcionki albo None (zostaje domyslna egui).
pub fn install_system_font(ctx: &egui::Context) -> Option<PathBuf> {
    let (path, bytes) = candidates()
        .into_iter()
        .find_map(|path| read_font(&path).map(|bytes| (path, bytes)))?;

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        FONT_NAME.to_string(),
        Arc::new(egui::FontData::from_owned(bytes)),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, FONT_NAME.to_string());
    ctx.set_fonts(fonts);
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_font_headers() {
        assert!(looks_like_font(&[0, 1, 0, 0, 9, 9]));
        assert!(looks_like_font(b"OTTO\0\0"));
        assert!(looks_like_font(b"ttcf\0\0"));
        assert!(looks_like_font(b"true\0\0"));
        assert!(!looks_like_font(b"<html>"));
        assert!(!looks_like_font(b"ab"));
        assert!(!looks_like_font(&[]));
    }

    #[test]
    fn rejects_non_font_files() {
        let dir = std::env::temp_dir().join(format!("qrgen_font_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("fake.ttf");
        std::fs::write(&fake, b"to nie jest czcionka").unwrap();
        assert!(read_font(&fake).is_none());
        assert!(read_font(&dir.join("brak.ttf")).is_none());
        assert!(read_font(&dir).is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn candidate_list_is_not_empty() {
        assert!(!candidates().is_empty());
    }
}
