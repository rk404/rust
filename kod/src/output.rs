use crate::docx;
use crate::formats::VCardData;
use crate::render::{self, Appearance};
use image::RgbImage;
use std::fs;
use std::path::{Path, PathBuf};

/// Domyslne rozszerzenie zapisywanego obrazu (decyzja uzytkownika: .jpg).
pub const DEFAULT_EXTENSION: &str = "jpg";
pub const SUPPORTED_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "bmp"];
const JPEG_QUALITY: u8 = 95;
const MAX_UNIQUE_SUFFIX: u32 = 99;

/// Transliteracja polskich znakow diakrytycznych na ASCII - nazwy plikow
/// przechodza przez ERP/SMB/mail, gdzie znaki spoza ASCII bywaja problemem.
pub fn transliterate_pl(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ą' => 'a',
            'ć' => 'c',
            'ę' => 'e',
            'ł' => 'l',
            'ń' => 'n',
            'ó' => 'o',
            'ś' => 's',
            'ź' => 'z',
            'ż' => 'z',
            'Ą' => 'A',
            'Ć' => 'C',
            'Ę' => 'E',
            'Ł' => 'L',
            'Ń' => 'N',
            'Ó' => 'O',
            'Ś' => 'S',
            'Ź' => 'Z',
            'Ż' => 'Z',
            _ => c,
        })
        .collect()
}

/// Sanityzacja rdzenia nazwy pliku: transliteracja, spacje na "_",
/// tylko [A-Za-z0-9._-], zbite powtorzenia "_", bez nazw zastrzezonych Windows.
pub fn sanitize_stem(s: &str) -> String {
    let mut out = String::new();
    let mut last_was_underscore = false;
    for c in transliterate_pl(s.trim()).chars() {
        let mapped = if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
            Some(c)
        } else if c.is_whitespace() || c == '_' {
            Some('_')
        } else {
            None
        };
        if let Some(m) = mapped {
            if m == '_' {
                if last_was_underscore {
                    continue;
                }
                last_was_underscore = true;
            } else {
                last_was_underscore = false;
            }
            out.push(m);
        }
    }
    let trimmed = out.trim_matches(|c| c == '_' || c == '.').to_string();

    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved.contains(&trimmed.to_ascii_uppercase().as_str()) {
        return format!("{trimmed}_");
    }
    trimmed
}

/// Rdzen nazwy plikow wizytowki: "Imie_Nazwisko" (po sanityzacji),
/// a gdy brak danych - "wizytowka".
pub fn vcard_stem(v: &VCardData) -> String {
    let stem = sanitize_stem(&format!("{} {}", v.first_name.trim(), v.last_name.trim()));
    if stem.is_empty() {
        "wizytowka".to_string()
    } else {
        stem
    }
}

/// Zamienia (mozliwie wzgledna) sciezke na absolutna - do wyswietlania
/// w interfejsie i komunikatach.
pub fn resolve_full_path(path: &Path) -> String {
    if path.is_absolute() {
        return path.display().to_string();
    }
    std::env::current_dir()
        .map(|dir| dir.join(path).display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

/// Zlecenie wygenerowania i zapisania kodu QR (plus ewentualnego .docx).
pub struct SaveRequest<'a> {
    pub payload: &'a str,
    pub appearance: &'a Appearance,
    /// Jawna sciezka (-o / pole GUI): plik z rozszerzeniem LUB katalog.
    pub output: Option<&'a str>,
    /// Katalog uzywany, gdy `output` nie wskazuje pelnej sciezki pliku.
    pub default_dir: PathBuf,
    /// Proponowany rdzen nazwy pliku (przed sanityzacja).
    pub stem: &'a str,
    /// Rozszerzenie obrazu, gdy nie wynika z `output` (jpg/jpeg/png/bmp).
    pub extension: &'a str,
    /// Some(dane) => obok obrazu powstaje dokument Word z wizytowka.
    pub docx: Option<&'a VCardData>,
    pub overwrite: bool,
}

pub struct SaveResult {
    pub image_path: PathBuf,
    pub docx_path: Option<PathBuf>,
}

/// Wyznacza katalog, rdzen i rozszerzenie na podstawie zlecenia.
fn resolve_targets(req: &SaveRequest) -> Result<(PathBuf, String, String), String> {
    let fallback_stem = {
        let s = sanitize_stem(req.stem);
        if s.is_empty() {
            "qrcode".to_string()
        } else {
            s
        }
    };

    let (dir, stem, ext) = match req.output {
        None => (req.default_dir.clone(), fallback_stem, req.extension.to_string()),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                (req.default_dir.clone(), fallback_stem, req.extension.to_string())
            } else {
                let path = PathBuf::from(trimmed);
                let ends_with_sep = trimmed.ends_with('/') || trimmed.ends_with('\\');
                let has_extension = path
                    .extension()
                    .map(|e| !e.is_empty())
                    .unwrap_or(false);
                if ends_with_sep || path.is_dir() || !has_extension {
                    // Katalog (istniejacy, zakonczony separatorem lub bez
                    // rozszerzenia - np. ERP podaje katalog, ktory dopiero powstanie).
                    (path, fallback_stem, req.extension.to_string())
                } else {
                    let ext = path
                        .extension()
                        .map(|e| e.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    let stem = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or(fallback_stem);
                    let dir = match path.parent() {
                        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
                        _ => req.default_dir.clone(),
                    };
                    (dir, stem, ext)
                }
            }
        }
    };

    let ext = ext.to_lowercase();
    if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(format!(
            "nieobslugiwane rozszerzenie '.{ext}' (dostepne: jpg, jpeg, png, bmp)"
        ));
    }
    Ok((dir, stem, ext))
}

/// Dobiera nazwy nie nadpisujace istniejacych plikow: przy kolizji dokleja
/// "_2", "_3", ... (obraz i docx zawsze dostaja ten sam sufiks, zeby byly para).
fn unique_targets(
    dir: &Path,
    stem: &str,
    ext: &str,
    want_docx: bool,
    overwrite: bool,
) -> Result<(PathBuf, Option<PathBuf>), String> {
    let candidate = |n: u32| {
        let full_stem = if n <= 1 {
            stem.to_string()
        } else {
            format!("{stem}_{n}")
        };
        let image = dir.join(format!("{full_stem}.{ext}"));
        let docx = want_docx.then(|| dir.join(format!("{full_stem}.docx")));
        (image, docx)
    };

    if overwrite {
        let (image, docx) = candidate(1);
        return Ok((image, docx));
    }

    for n in 1..=MAX_UNIQUE_SUFFIX {
        let (image, docx) = candidate(n);
        let taken = image.exists() || docx.as_ref().map(|d| d.exists()).unwrap_or(false);
        if !taken {
            return Ok((image, docx));
        }
    }
    Err(format!(
        "w katalogu {} istnieje juz {MAX_UNIQUE_SUFFIX} plikow o nazwie '{stem}' - usun stare pliki lub wlacz nadpisywanie",
        dir.display()
    ))
}

/// Zapis obrazu: JPEG z jakoscia 95 (ostre krawedzie modulow bez artefaktow),
/// PNG/BMP przez standardowy zapis wg rozszerzenia.
pub fn save_image(img: &RgbImage, path: &Path) -> Result<(), String> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let result = if ext == "jpg" || ext == "jpeg" {
        let file = fs::File::create(path)
            .map_err(|e| format!("nie udalo sie utworzyc pliku '{}': {e}", path.display()))?;
        let writer = std::io::BufWriter::new(file);
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(writer, JPEG_QUALITY);
        img.write_with_encoder(encoder)
            .map_err(|e| format!("nie udalo sie zapisac pliku '{}': {e}", path.display()))
    } else {
        img.save(path)
            .map_err(|e| format!("nie udalo sie zapisac pliku '{}': {e}", path.display()))
    };
    result
}

/// Renderuje kod QR i zapisuje obraz (oraz ewentualny .docx z wizytowka).
pub fn generate_and_save(req: &SaveRequest) -> Result<SaveResult, String> {
    let img = render::render_image(req.payload, req.appearance)?;

    let (dir, stem, ext) = resolve_targets(req)?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("nie udalo sie utworzyc katalogu '{}': {e}", dir.display()))?;
    let (image_path, docx_path) =
        unique_targets(&dir, &stem, &ext, req.docx.is_some(), req.overwrite)?;

    save_image(&img, &image_path)?;

    if let (Some(vcard), Some(docx_target)) = (req.docx, docx_path.as_ref()) {
        let bytes = docx::build_business_card_docx(vcard)?;
        fs::write(docx_target, bytes).map_err(|e| {
            format!(
                "nie udalo sie zapisac pliku '{}': {e}",
                docx_target.display()
            )
        })?;
    }

    Ok(SaveResult {
        image_path,
        docx_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transliterates_polish() {
        assert_eq!(transliterate_pl("Żółć Łąka źdźbło"), "Zolc Laka zdzblo");
    }

    #[test]
    fn sanitizes_stems() {
        assert_eq!(sanitize_stem("Żółć Łąka"), "Zolc_Laka");
        assert_eq!(sanitize_stem("Jan  Kowalski"), "Jan_Kowalski");
        assert_eq!(sanitize_stem("a/b\\c:d*e"), "abcde");
        assert_eq!(sanitize_stem("__x__"), "x");
        assert_eq!(sanitize_stem("CON"), "CON_");
        assert_eq!(sanitize_stem("...."), "");
    }

    #[test]
    fn vcard_stem_fallback() {
        assert_eq!(vcard_stem(&VCardData::default()), "wizytowka");
        let v = VCardData {
            first_name: "Rafał".to_string(),
            last_name: "Kamiński".to_string(),
            ..Default::default()
        };
        assert_eq!(vcard_stem(&v), "Rafal_Kaminski");
    }

    #[test]
    fn unique_suffix_added_when_file_exists() {
        let dir = std::env::temp_dir().join(format!("qrgen_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Jan_Kowalski.jpg"), b"x").unwrap();
        let (image, docx) = unique_targets(&dir, "Jan_Kowalski", "jpg", true, false).unwrap();
        assert!(image.ends_with("Jan_Kowalski_2.jpg"));
        assert!(docx.unwrap().ends_with("Jan_Kowalski_2.docx"));

        // z nadpisywaniem: bez sufiksu mimo istniejacego pliku
        let (image, _) = unique_targets(&dir, "Jan_Kowalski", "jpg", false, true).unwrap();
        assert!(image.ends_with("Jan_Kowalski.jpg"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolves_output_variants() {
        let default_dir = PathBuf::from("/tmp/qrgen_default");
        let ap = Appearance::default();
        let request = |output: Option<&'static str>, stem: &'static str| SaveRequest {
            payload: "x",
            appearance: &ap,
            output,
            default_dir: default_dir.clone(),
            stem,
            extension: "jpg",
            docx: None,
            overwrite: false,
        };

        // brak -o: katalog domyslny + rdzen + domyslne rozszerzenie
        let (dir, stem, ext) = resolve_targets(&request(None, "qrcode")).unwrap();
        assert_eq!((dir, stem, ext), (default_dir.clone(), "qrcode".into(), "jpg".into()));

        // -o z pelna sciezka pliku: rozszerzenie z pliku wygrywa
        let (dir, stem, ext) =
            resolve_targets(&request(Some("/dane/moj_kod.png"), "qrcode")).unwrap();
        assert_eq!(dir, PathBuf::from("/dane"));
        assert_eq!((stem.as_str(), ext.as_str()), ("moj_kod", "png"));

        // -o bez rozszerzenia: traktowane jako katalog
        let (dir, stem, _) =
            resolve_targets(&request(Some("/dane/wizytowki"), "Jan_Kowalski")).unwrap();
        assert_eq!(dir, PathBuf::from("/dane/wizytowki"));
        assert_eq!(stem, "Jan_Kowalski");

        // nieobslugiwane rozszerzenie
        assert!(resolve_targets(&request(Some("/dane/kod.gif"), "qrcode")).is_err());
    }
}
