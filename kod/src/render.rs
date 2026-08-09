use image::{Rgb, RgbImage};
use qrcode::{Color, EcLevel, QrCode};

// Granice bezpieczenstwa: wieksze wartosci nie maja praktycznego sensu,
// a chronia przed przypadkowa alokacja wielogigabajtowego obrazu.
pub const MAX_SCALE: u32 = 100;
pub const MAX_BORDER: u32 = 50;
pub const MAX_DIM: u32 = 20_000;

/// Ustawienia wygladu kodu QR - oddzielone od danych do zakodowania.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Appearance {
    pub scale: u32,
    pub border: u32,
    /// Docelowy bok obrazu w pikselach; 0 = rozmiar wynika ze skali.
    /// Gdy > 0, skala jest dobierana tak, aby obraz byl mozliwie bliski
    /// (nie wiekszy niz) zadanej wartosci.
    pub target_px: u32,
    pub ecc: EcLevel,
    pub fill: [u8; 3],
    pub back: [u8; 3],
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            scale: 10,
            border: 4,
            target_px: 0,
            ecc: EcLevel::M,
            fill: [0, 0, 0],
            back: [255, 255, 255],
        }
    }
}

pub fn ecc_from_letter(s: &str) -> Option<EcLevel> {
    match s.trim().to_ascii_uppercase().as_str() {
        "L" => Some(EcLevel::L),
        "M" => Some(EcLevel::M),
        "Q" => Some(EcLevel::Q),
        "H" => Some(EcLevel::H),
        _ => None,
    }
}

pub fn ecc_to_letter(e: EcLevel) -> &'static str {
    match e {
        EcLevel::L => "L",
        EcLevel::M => "M",
        EcLevel::Q => "Q",
        EcLevel::H => "H",
    }
}

/// Parsuje kolor w formacie "RRGGBB" (hex) na tablice [r,g,b].
pub fn parse_hex_color(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r, g, b])
}

/// Koduje dane i renderuje kod QR jako obraz RGB, z uwzglednieniem
/// skali (rozmiar modulu w pikselach), ramki (w modulach) i kolorow.
///
/// Dane sa kodowane w trybie bajtowym jako UTF-8 bez naglowka ECI - wspolczesne
/// czytniki (zxing, iOS, Android) domyslnie interpretuja tryb bajtowy jako UTF-8,
/// a naglowek ECI pogarsza zgodnosc ze starszymi skanerami. Poprawnosc polskich
/// znakow potwierdza test round-trip w tests/roundtrip.rs.
pub fn render_image(payload: &str, ap: &Appearance) -> Result<RgbImage, String> {
    if payload.is_empty() {
        return Err("brak danych do zakodowania".to_string());
    }
    if ap.scale < 1 || ap.scale > MAX_SCALE {
        return Err(format!("skala musi byc w zakresie 1-{MAX_SCALE}"));
    }
    if ap.border > MAX_BORDER {
        return Err(format!("ramka musi byc w zakresie 0-{MAX_BORDER}"));
    }
    if ap.target_px > MAX_DIM {
        return Err(format!("docelowy rozmiar nie moze przekraczac {MAX_DIM} px"));
    }

    let code = QrCode::with_error_correction_level(payload.as_bytes(), ap.ecc)
        .map_err(|e| format!("nie udalo sie zakodowac danych: {:?}", e))?;

    let size = code.width() as u32;
    let modules = size
        .checked_add(ap.border.checked_mul(2).ok_or("ramka zbyt duza")?)
        .ok_or("ramka zbyt duza")?;
    // Docelowa rozdzielczosc: skala = najwiekszy calkowity mnoznik, przy ktorym
    // obraz nie przekracza zadanego boku (kod QR wymaga calkowitej liczby
    // pikseli na modul, wiec dokladna wartosc nie zawsze jest osiagalna).
    let scale = if ap.target_px > 0 {
        (ap.target_px / modules).clamp(1, MAX_SCALE)
    } else {
        ap.scale
    };
    let dim = modules
        .checked_mul(scale)
        .ok_or("wynikowy obraz bylby zbyt duzy")?;
    if dim > MAX_DIM {
        return Err(format!(
            "wynikowy obraz mialby {dim}x{dim} px (limit {MAX_DIM}); zmniejsz skale lub ramke"
        ));
    }

    let mut img = RgbImage::new(dim, dim);
    let fill = Rgb(ap.fill);
    let back = Rgb(ap.back);
    let border = ap.border;

    for y in 0..dim {
        let qy = y / scale;
        for x in 0..dim {
            let qx = x / scale;
            let is_dark = if qx >= border && qy >= border {
                let mx = qx - border;
                let my = qy - border;
                if mx < size && my < size {
                    code[(mx as usize, my as usize)] == Color::Dark
                } else {
                    false
                }
            } else {
                false
            };
            img.put_pixel(x, y, if is_dark { fill } else { back });
        }
    }

    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn appearance() -> Appearance {
        Appearance::default()
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("000000"), Some([0, 0, 0]));
        assert_eq!(parse_hex_color("FFFFFF"), Some([255, 255, 255]));
        assert_eq!(parse_hex_color("#0033AA"), Some([0x00, 0x33, 0xAA]));
        assert_eq!(parse_hex_color("zzzzzz"), None);
        assert_eq!(parse_hex_color("FFF"), None);
    }

    #[test]
    fn test_render_basic() {
        let img = render_image("https://example.com", &appearance()).unwrap();
        assert!(img.width() > 0 && img.height() > 0);
    }

    #[test]
    fn test_empty_data_errors() {
        assert!(render_image("", &appearance()).is_err());
    }

    #[test]
    fn test_oversized_scale_rejected() {
        let ap = Appearance {
            scale: MAX_SCALE + 1,
            ..appearance()
        };
        assert!(render_image("test", &ap).is_err());
        let ap = Appearance {
            scale: 0,
            ..appearance()
        };
        assert!(render_image("test", &ap).is_err());
    }

    #[test]
    fn test_oversized_border_rejected() {
        let ap = Appearance {
            border: MAX_BORDER + 1,
            ..appearance()
        };
        assert!(render_image("test", &ap).is_err());
    }

    #[test]
    fn test_target_px_overrides_scale() {
        let ap = Appearance {
            scale: 1,
            target_px: 1000,
            ..appearance()
        };
        let img = render_image("https://example.com", &ap).unwrap();
        // obraz mozliwie bliski 1000 px, ale nie wiekszy
        assert!(img.width() <= 1000);
        assert!(img.width() > 800);

        // cel mniejszy niz liczba modulow -> minimalna skala 1
        let ap = Appearance {
            target_px: 10,
            ..appearance()
        };
        let img = render_image("https://example.com", &ap).unwrap();
        assert!(img.width() >= 21);
    }

    #[test]
    fn test_dim_cap_enforced() {
        // duze dane (wysoka wersja QR) x maksymalna skala przekraczaja MAX_DIM
        let data = "x".repeat(1000);
        let ap = Appearance {
            scale: MAX_SCALE,
            border: MAX_BORDER,
            ..appearance()
        };
        assert!(render_image(&data, &ap).is_err());
    }
}
