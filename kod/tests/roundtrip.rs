// Testy round-trip: zbuduj payload -> wyrenderuj kod QR -> zdekoduj rqrr
// -> porownaj. Dowodza m.in., ze polskie znaki (UTF-8 w trybie bajtowym,
// bez naglowka ECI) przezywaja pelna droge, takze przez stratny zapis JPEG.

use qrgen_rust::decode;
use qrgen_rust::formats::{EmailForm, FormsState, LinkForm, VCardData};
use qrgen_rust::output;
use qrgen_rust::render::{render_image, Appearance};

fn decode_payload(payload: &str) -> decode::DecodedQr {
    let img = render_image(payload, &Appearance::default()).unwrap();
    let gray = image::DynamicImage::ImageRgb8(img).to_luma8();
    decode::decode_luma(gray.width() as usize, gray.height() as usize, gray.as_raw()).unwrap()
}

fn sample_vcard() -> VCardData {
    VCardData {
        first_name: "Rafał".to_string(),
        last_name: "Kamiński".to_string(),
        phone_work: "58 307 00 00".to_string(),
        phone_mobile: "600 000 000".to_string(),
        email: "rafal.kaminski@firma.pl".to_string(),
        website: "https://firma.pl".to_string(),
        company: "Żółta Spółka; z o.o.".to_string(),
        title_pl: "Kierownik Logistyki Dostaw".to_string(),
        title_en: "Supply Chain Manager".to_string(),
        fax: "58 307 00 01".to_string(),
        street: "ul. Świętojańska 1".to_string(),
        city: "Gdańsk".to_string(),
        postal_code: "80-001".to_string(),
        country: "Polska".to_string(),
    }
}

#[test]
fn text_with_polish_characters() {
    let payload = "Zażółć gęślą jaźń — ąćęłńóśźż ĄĆĘŁŃÓŚŹŻ";
    assert_eq!(decode_payload(payload).raw, payload);
}

#[test]
fn link_roundtrip() {
    let form = LinkForm {
        url: "firma.pl/oferta".to_string(),
    };
    let payload = form.payload().unwrap();
    assert_eq!(decode_payload(&payload).raw, "https://firma.pl/oferta");
}

#[test]
fn email_roundtrip() {
    let form = EmailForm {
        to: "jan@firma.pl".to_string(),
        subject: "Späte Grüße / późne pozdrowienia".to_string(),
        body: "Linia 1\nLinia 2".to_string(),
    };
    let payload = form.payload().unwrap();
    let decoded = decode_payload(&payload);
    assert_eq!(decoded.raw, payload);
    assert!(decoded.raw.starts_with("mailto:jan%40firma.pl?subject="));
}

#[test]
fn vcard_full_roundtrip_with_field_equality() {
    let original = sample_vcard();
    let payload = original.payload().unwrap();
    let decoded = decode_payload(&payload);

    let parsed = decoded.vcard.expect("odczytany kod powinien byc vCardem");
    let mut expected = original.clone();
    expected.title_en = String::new(); // title_en celowo nie trafia do vCard
    assert_eq!(parsed, expected);
}

#[test]
fn jpeg_save_survives_decode() {
    // Zapis JPEG (q95) jest stratny - test potwierdza, ze kod QR pozostaje
    // czytelny po przejsciu przez kompresje.
    let payload = sample_vcard().payload().unwrap();
    let img = render_image(&payload, &Appearance::default()).unwrap();

    let dir = std::env::temp_dir().join(format!("qrgen_roundtrip_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("roundtrip.jpg");
    output::save_image(&img, &path).unwrap();

    let decoded = decode::decode_file(&path).unwrap();
    assert_eq!(decoded.raw, payload);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn forms_state_payload_dispatch() {
    let mut forms = FormsState::default();
    forms.text.data = "abc".to_string();
    forms.vcard = sample_vcard();
    assert_eq!(
        forms.payload(qrgen_rust::formats::FormatKind::Text).unwrap(),
        "abc"
    );
    assert!(forms
        .payload(qrgen_rust::formats::FormatKind::VCard)
        .unwrap()
        .contains("Kamiński"));
    // pusty formularz link -> czytelny blad
    assert!(forms.payload(qrgen_rust::formats::FormatKind::Link).is_err());
}
