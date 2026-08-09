use crate::formats::vcard::{self, VCardData};
use std::path::Path;

/// Wynik odczytu kodu QR z obrazu.
pub struct DecodedQr {
    /// Surowa tresc zakodowana w kodzie.
    pub raw: String,
    /// Rozpoznane pola, gdy tresc jest wizytowka vCard.
    pub vcard: Option<VCardData>,
}

fn wrap(raw: String) -> DecodedQr {
    let vcard = vcard::parse_vcard(&raw);
    DecodedQr { raw, vcard }
}

/// Dekoduje pierwszy kod QR znaleziony w obrazie w skali szarosci.
/// `luma` musi miec dlugosc width*height (wiersz po wierszu).
pub fn decode_luma(width: usize, height: usize, luma: &[u8]) -> Result<DecodedQr, String> {
    if luma.len() != width * height {
        return Err("nieprawidlowe wymiary obrazu".to_string());
    }

    let mut prepared =
        rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| luma[y * width + x]);
    let grids = prepared.detect_grids();
    let grid = grids
        .first()
        .ok_or("Nie znaleziono kodu QR na obrazie.".to_string())?;
    let (_meta, raw) = grid
        .decode()
        .map_err(|e| format!("Nie udalo sie odczytac kodu QR: {e:?}"))?;
    Ok(wrap(raw))
}

/// Dekoduje kod QR z bufora RGBA (np. obraz wklejony ze schowka).
pub fn decode_rgba(width: usize, height: usize, rgba: &[u8]) -> Result<DecodedQr, String> {
    if rgba.len() != width * height * 4 {
        return Err("nieprawidlowe wymiary obrazu".to_string());
    }
    let luma: Vec<u8> = rgba
        .chunks_exact(4)
        .map(|px| {
            // standardowe wagi luminancji (BT.601)
            ((px[0] as u32 * 299 + px[1] as u32 * 587 + px[2] as u32 * 114) / 1000) as u8
        })
        .collect();
    decode_luma(width, height, &luma)
}

/// Dekoduje kod QR z pliku obrazu (PNG/JPG/BMP).
pub fn decode_file(path: &Path) -> Result<DecodedQr, String> {
    let img = image::open(path)
        .map_err(|e| format!("Nie udalo sie otworzyc obrazu '{}': {e}", path.display()))?;
    let gray = img.to_luma8();
    decode_luma(
        gray.width() as usize,
        gray.height() as usize,
        gray.as_raw(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{render_image, Appearance};

    fn decode_rendered(payload: &str, ap: &Appearance) -> DecodedQr {
        let img = render_image(payload, ap).unwrap();
        let gray = image::DynamicImage::ImageRgb8(img).to_luma8();
        decode_luma(gray.width() as usize, gray.height() as usize, gray.as_raw()).unwrap()
    }

    #[test]
    fn roundtrip_simple_text() {
        let decoded = decode_rendered("https://example.com", &Appearance::default());
        assert_eq!(decoded.raw, "https://example.com");
        assert!(decoded.vcard.is_none());
    }

    #[test]
    fn no_qr_in_blank_image() {
        let luma = vec![255u8; 100 * 100];
        assert!(decode_luma(100, 100, &luma).is_err());
    }
}
