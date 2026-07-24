use image::{Rgb, RgbImage};
use qrcode::{Color, EcLevel, QrCode};

#[derive(Clone, Debug)]
pub struct Options {
    pub data: String,
    pub output: String,
    pub scale: u32,
    pub border: u32,
    pub ecc: EcLevel,
    pub fill: [u8; 3],
    pub back: [u8; 3],
}

impl Default for Options {
    fn default() -> Self {
        Self {
            data: String::new(),
            output: "qrcode.png".to_string(),
            scale: 10,
            border: 4,
            ecc: EcLevel::M,
            fill: [0, 0, 0],
            back: [255, 255, 255],
        }
    }
}

pub fn ecc_from_letter(s: &str) -> Option<EcLevel> {
    match s {
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
pub fn render_image(opts: &Options) -> Result<RgbImage, String> {
    if opts.data.is_empty() {
        return Err("brak danych do zakodowania".to_string());
    }

    let code = QrCode::with_error_correction_level(opts.data.as_bytes(), opts.ecc)
        .map_err(|e| format!("nie udalo sie zakodowac danych: {:?}", e))?;

    let size = code.width() as u32;
    let scale = opts.scale.max(1);
    let border = opts.border;
    let total_modules = size + border * 2;
    let dim = (total_modules * scale).max(1);

    let mut img = RgbImage::new(dim, dim);
    let fill = Rgb(opts.fill);
    let back = Rgb(opts.back);

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

/// Generuje kod QR i zapisuje go do pliku okreslonego w opts.output.
/// Format pliku (PNG/BMP/...) jest okreslany na podstawie rozszerzenia.
pub fn generate_and_save(opts: &Options) -> Result<String, String> {
    let img = render_image(opts)?;
    img.save(&opts.output)
        .map_err(|e| format!("nie udalo sie zapisac pliku '{}': {}", opts.output, e))?;
    Ok(format!("Kod QR zostal zapisany w pliku: {}", opts.output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("000000"), Some([0, 0, 0]));
        assert_eq!(parse_hex_color("FFFFFF"), Some([255, 255, 255]));
        assert_eq!(parse_hex_color("0033AA"), Some([0x00, 0x33, 0xAA]));
        assert_eq!(parse_hex_color("zzzzzz"), None);
        assert_eq!(parse_hex_color("FFF"), None);
    }

    #[test]
    fn test_render_basic() {
        let opts = Options {
            data: "https://example.com".to_string(),
            ..Options::default()
        };
        let img = render_image(&opts).unwrap();
        assert!(img.width() > 0 && img.height() > 0);
    }

    #[test]
    fn test_empty_data_errors() {
        let opts = Options::default();
        assert!(render_image(&opts).is_err());
    }
}
