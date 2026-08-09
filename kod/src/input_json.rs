use crate::formats::{EmailForm, FormatKind, FormsState, LinkForm, TextForm, VCardData};
use crate::render::{self, Appearance};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Schemat pliku --input (glowna sciezka integracji z ERP).
/// Wszystkie pola opcjonalne; nieznane klucze sa ignorowane.
/// Klucze celowo angielskie, 1:1 z flagami CLI - unika problemow z kodowaniem
/// polskich znakow po stronie systemow wywolujacych.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct JsonInput {
    pub format: Option<FormatKind>,
    pub output: Option<String>,
    pub overwrite: Option<bool>,
    pub docx: Option<bool>,
    pub appearance: Option<JsonAppearance>,
    pub vcard: Option<VCardData>,
    pub text: Option<TextForm>,
    pub link: Option<LinkForm>,
    pub email: Option<EmailForm>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct JsonAppearance {
    pub scale: Option<u32>,
    pub border: Option<u32>,
    /// Docelowy bok obrazu w px (nadpisuje skale).
    pub size: Option<u32>,
    pub ecc: Option<String>,
    pub fill: Option<String>,
    pub back: Option<String>,
}

/// Wczytuje i parsuje plik JSON (UTF-8; BOM tolerowany).
pub fn load(path: &Path) -> Result<JsonInput, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("nie udalo sie odczytac pliku '{}': {e}", path.display()))?;
    let text = std::str::from_utf8(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes))
        .map_err(|_| {
            format!(
                "plik '{}' nie jest poprawnym UTF-8 - zapisz go w kodowaniu UTF-8",
                path.display()
            )
        })?;
    serde_json::from_str(text)
        .map_err(|e| format!("bledny JSON w pliku '{}': {e}", path.display()))
}

impl JsonInput {
    /// Nadpisuje formularze danymi z JSON (sekcja per format, w calosci).
    pub fn apply_to_forms(&self, forms: &mut FormsState) {
        if let Some(vcard) = &self.vcard {
            forms.vcard = vcard.clone();
        }
        if let Some(text) = &self.text {
            forms.text = text.clone();
        }
        if let Some(link) = &self.link {
            forms.link = link.clone();
        }
        if let Some(email) = &self.email {
            forms.email = email.clone();
        }
    }

    /// Czy JSON zawiera jakiekolwiek dane tresci.
    pub fn has_content(&self) -> bool {
        self.vcard.is_some() || self.text.is_some() || self.link.is_some() || self.email.is_some()
    }

    /// Nadpisuje ustawienia wygladu; bledne wartosci to twardy blad
    /// (zgloszony z nazwa pola), nie ciche pominiecie.
    pub fn apply_appearance(&self, ap: &mut Appearance) -> Result<(), String> {
        let Some(ja) = &self.appearance else {
            return Ok(());
        };
        if let Some(scale) = ja.scale {
            ap.scale = scale;
        }
        if let Some(border) = ja.border {
            ap.border = border;
        }
        if let Some(size) = ja.size {
            ap.target_px = size;
        }
        if let Some(ecc) = &ja.ecc {
            ap.ecc = render::ecc_from_letter(ecc)
                .ok_or_else(|| format!("bledna wartosc appearance.ecc: '{ecc}' (dozwolone: L, M, Q, H)"))?;
        }
        if let Some(fill) = &ja.fill {
            ap.fill = render::parse_hex_color(fill)
                .ok_or_else(|| format!("bledna wartosc appearance.fill: '{fill}' (format RRGGBB)"))?;
        }
        if let Some(back) = &ja.back {
            ap.back = render::parse_hex_color(back)
                .ok_or_else(|| format!("bledna wartosc appearance.back: '{back}' (format RRGGBB)"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> JsonInput {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn full_document() {
        let input = parse(
            r#"{
                "format": "vcard",
                "output": "C:/Wizytowki",
                "overwrite": true,
                "docx": false,
                "appearance": { "scale": 12, "ecc": "H", "fill": "112233" },
                "vcard": { "first_name": "Jan", "last_name": "Kowalski", "city": "Gdańsk" }
            }"#,
        );
        assert_eq!(input.format, Some(FormatKind::VCard));
        assert_eq!(input.overwrite, Some(true));
        assert_eq!(input.docx, Some(false));

        let mut forms = FormsState::default();
        input.apply_to_forms(&mut forms);
        assert_eq!(forms.vcard.first_name, "Jan");
        assert_eq!(forms.vcard.city, "Gdańsk");
        assert_eq!(forms.vcard.fax, ""); // brakujace pola vCard = puste

        let mut ap = Appearance::default();
        input.apply_appearance(&mut ap).unwrap();
        assert_eq!(ap.scale, 12);
        assert_eq!(ap.fill, [0x11, 0x22, 0x33]);
        assert_eq!(ap.border, Appearance::default().border); // niepodane = bez zmian
    }

    #[test]
    fn minimal_document() {
        let input = parse(r#"{ "text": { "data": "abc" } }"#);
        assert!(input.has_content());
        assert_eq!(input.format, None);
    }

    #[test]
    fn unknown_keys_ignored() {
        let input = parse(r#"{ "nieznane_pole": 1, "link": { "url": "x.pl", "extra": true } }"#);
        assert_eq!(input.link.unwrap().url, "x.pl");
    }

    #[test]
    fn bad_ecc_reports_field() {
        let input = parse(r#"{ "appearance": { "ecc": "X" } }"#);
        let err = input.apply_appearance(&mut Appearance::default()).unwrap_err();
        assert!(err.contains("appearance.ecc"));
    }

    #[test]
    fn bom_and_encoding() {
        let dir = std::env::temp_dir().join(format!("qrgen_json_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();

        // BOM tolerowany
        let bom_path = dir.join("bom.json");
        fs::write(&bom_path, b"\xEF\xBB\xBF{\"format\":\"text\"}").unwrap();
        assert_eq!(load(&bom_path).unwrap().format, Some(FormatKind::Text));

        // nie-UTF-8 odrzucone z czytelnym komunikatem
        let bad_path = dir.join("latin2.json");
        fs::write(&bad_path, [b'{', 0xB1, b'}']).unwrap();
        assert!(load(&bad_path).unwrap_err().contains("UTF-8"));

        fs::remove_dir_all(&dir).ok();
    }
}
