use crate::formats::VCardData;
use std::io::Write;

/// Escaping tekstu wstawianego do XML.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Jeden akapit dokumentu; `bold` pogrubia caly wiersz.
fn paragraph(text: &str, bold: bool) -> String {
    if text.is_empty() {
        return "<w:p/>".to_string();
    }
    let bold_tag = if bold { "<w:b/>" } else { "" };
    format!(
        "<w:p><w:r><w:rPr><w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\"/><w:sz w:val=\"22\"/>{bold_tag}</w:rPr><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        xml_escape(text)
    )
}

/// Numer z prefiksem "+48 " dla bloku angielskiego - chyba ze numer juz jest
/// w formacie miedzynarodowym (zaczyna sie od '+'). Heurystyka; latwa do zmiany.
fn international(number: &str) -> String {
    let number = number.trim();
    if number.starts_with('+') {
        number.to_string()
    } else {
        format!("+48 {number}")
    }
}

/// Buduje tresc wizytowki: blok polski (T:/K:) oraz - gdy podano stanowisko
/// po angielsku - blok angielski (T +48/M +48), wg szablonu uzytkownika.
fn document_paragraphs(v: &VCardData) -> Vec<String> {
    let full_name = v.full_name();
    let mut paragraphs = Vec::new();

    paragraphs.push(paragraph(&full_name, true));
    if !v.title_pl.trim().is_empty() {
        paragraphs.push(paragraph(&v.title_pl.trim().to_uppercase(), false));
    }
    if !v.phone_work.trim().is_empty() {
        paragraphs.push(paragraph(&format!("T: {}", v.phone_work.trim()), false));
    }
    if !v.phone_mobile.trim().is_empty() {
        paragraphs.push(paragraph(&format!("K: {}", v.phone_mobile.trim()), false));
    }
    if !v.email.trim().is_empty() {
        paragraphs.push(paragraph(v.email.trim(), false));
    }

    if !v.title_en.trim().is_empty() {
        paragraphs.push(paragraph("", false));
        paragraphs.push(paragraph(&full_name, true));
        paragraphs.push(paragraph(&v.title_en.trim().to_uppercase(), false));
        if !v.phone_work.trim().is_empty() {
            paragraphs.push(paragraph(
                &format!("T {}", international(&v.phone_work)),
                false,
            ));
        }
        if !v.phone_mobile.trim().is_empty() {
            paragraphs.push(paragraph(
                &format!("M {}", international(&v.phone_mobile)),
                false,
            ));
        }
        if !v.email.trim().is_empty() {
            paragraphs.push(paragraph(v.email.trim(), false));
        }
    }

    paragraphs
}

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;

const RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

/// Buduje kompletny plik .docx (paczka zip) z danymi do wizytowki.
pub fn build_business_card_docx(v: &VCardData) -> Result<Vec<u8>, String> {
    if v.full_name().is_empty() {
        return Err("podaj imie lub nazwisko".to_string());
    }

    let body: String = document_paragraphs(v).concat();
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1417" w:right="1417" w:bottom="1417" w:left="1417"/></w:sectPr></w:body></w:document>"#
    );

    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        let options: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for (name, content) in [
            ("[Content_Types].xml", CONTENT_TYPES),
            ("_rels/.rels", RELS),
            ("word/document.xml", document.as_str()),
        ] {
            zip.start_file(name, options)
                .map_err(|e| format!("blad tworzenia dokumentu docx: {e}"))?;
            zip.write_all(content.as_bytes())
                .map_err(|e| format!("blad tworzenia dokumentu docx: {e}"))?;
        }
        zip.finish()
            .map_err(|e| format!("blad tworzenia dokumentu docx: {e}"))?;
    }
    Ok(buffer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn sample() -> VCardData {
        VCardData {
            first_name: "Jan".to_string(),
            last_name: "Kowalski".to_string(),
            phone_work: "58 307 00 00".to_string(),
            phone_mobile: "600 000 000".to_string(),
            email: "jan@firma.pl".to_string(),
            title_pl: "Kierownik Logistyki Dostaw".to_string(),
            title_en: "Supply Chain Manager".to_string(),
            company: "Spółka & Co. <test>".to_string(),
            ..Default::default()
        }
    }

    fn document_xml(bytes: &[u8]) -> String {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut file = archive.by_name("word/document.xml").unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        content
    }

    #[test]
    fn builds_valid_zip_with_both_blocks() {
        let bytes = build_business_card_docx(&sample()).unwrap();
        let xml = document_xml(&bytes);
        assert!(xml.contains("Jan Kowalski"));
        assert!(xml.contains("KIEROWNIK LOGISTYKI DOSTAW"));
        assert!(xml.contains("SUPPLY CHAIN MANAGER"));
        assert!(xml.contains("T: 58 307 00 00"));
        assert!(xml.contains("K: 600 000 000"));
        assert!(xml.contains("T +48 58 307 00 00"));
        assert!(xml.contains("M +48 600 000 000"));
    }

    #[test]
    fn en_block_omitted_without_title_en() {
        let mut data = sample();
        data.title_en = String::new();
        let xml = document_xml(&build_business_card_docx(&data).unwrap());
        assert!(!xml.contains("+48"));
    }

    #[test]
    fn international_number_passthrough() {
        let mut data = sample();
        data.phone_mobile = "+49 170 000 000".to_string();
        let xml = document_xml(&build_business_card_docx(&data).unwrap());
        assert!(xml.contains("M +49 170 000 000"));
        assert!(!xml.contains("+48 +49"));
    }

    #[test]
    fn xml_escaping_applied() {
        // firma nie jest w szablonie wizytowki, wiec testujemy przez nazwisko
        let mut data = sample();
        data.last_name = "Kowalski & <Syn>".to_string();
        let xml = document_xml(&build_business_card_docx(&data).unwrap());
        assert!(xml.contains("Kowalski &amp; &lt;Syn&gt;"));
    }

    #[test]
    fn requires_name() {
        assert!(build_business_card_docx(&VCardData::default()).is_err());
    }
}
