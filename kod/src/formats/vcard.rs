use serde::{Deserialize, Serialize};

/// Dane wizytowki vCard. Pole `title_en` nie trafia do kodu QR - sluzy tylko
/// do angielskiego bloku w generowanym dokumencie Word.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VCardData {
    pub first_name: String,
    pub last_name: String,
    pub phone_work: String,
    pub phone_mobile: String,
    pub email: String,
    pub website: String,
    pub company: String,
    pub title_pl: String,
    pub title_en: String,
    pub fax: String,
    pub street: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
}

/// Escaping wartosci wg RFC 2426: \ ; , oraz nowe linie.
fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(c),
        }
    }
    out
}

fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') | Some('N') => out.push('\n'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

impl VCardData {
    pub fn full_name(&self) -> String {
        let full = format!("{} {}", self.first_name.trim(), self.last_name.trim());
        full.trim().to_string()
    }

    /// Buduje tekst vCard 3.0 (CRLF, UTF-8). Linie z pustymi wartosciami sa
    /// pomijane. Celowo bez foldingu dlugich linii - oszczedza to pojemnosc
    /// kodu QR, a czytniki mobilne akceptuja niezawijane vCardy.
    pub fn payload(&self) -> Result<String, String> {
        let first = self.first_name.trim();
        let last = self.last_name.trim();
        if first.is_empty() && last.is_empty() {
            return Err("podaj imie lub nazwisko".to_string());
        }

        let mut lines: Vec<String> = Vec::new();
        lines.push("BEGIN:VCARD".to_string());
        lines.push("VERSION:3.0".to_string());
        lines.push(format!("N:{};{};;;", escape(last), escape(first)));
        lines.push(format!("FN:{}", escape(&self.full_name())));

        let mut push_if = |prefix: &str, value: &str| {
            let value = value.trim();
            if !value.is_empty() {
                lines.push(format!("{prefix}{}", escape(value)));
            }
        };
        push_if("ORG:", &self.company);
        push_if("TITLE:", &self.title_pl);
        push_if("TEL;TYPE=WORK,VOICE:", &self.phone_work);
        push_if("TEL;TYPE=CELL:", &self.phone_mobile);
        push_if("TEL;TYPE=FAX,WORK:", &self.fax);
        push_if("EMAIL;TYPE=INTERNET,WORK:", &self.email);
        push_if("URL:", &self.website);

        let has_address = [&self.street, &self.city, &self.postal_code, &self.country]
            .iter()
            .any(|part| !part.trim().is_empty());
        if has_address {
            lines.push(format!(
                "ADR;TYPE=WORK:;;{};{};;{};{}",
                escape(self.street.trim()),
                escape(self.city.trim()),
                escape(self.postal_code.trim()),
                escape(self.country.trim()),
            ));
        }

        lines.push("END:VCARD".to_string());
        Ok(lines.join("\r\n"))
    }
}

/// Czy tekst wyglada na vCard (np. odczytany z kodu QR).
pub fn looks_like_vcard(raw: &str) -> bool {
    raw.trim_start()
        .get(..11)
        .is_some_and(|head| head.eq_ignore_ascii_case("BEGIN:VCARD"))
}

/// Parsuje vCard (2.1/3.0/4.0 w zakresie wspolnych wlasciwosci) na pola
/// formularza. Zwraca None, gdy tekst nie jest vCardem.
pub fn parse_vcard(raw: &str) -> Option<VCardData> {
    if !looks_like_vcard(raw) {
        return None;
    }

    // Unfolding (RFC 2426): linia zaczynajaca sie od spacji/tabulatora jest
    // kontynuacja poprzedniej.
    let mut unfolded: Vec<String> = Vec::new();
    for line in raw.replace("\r\n", "\n").replace('\r', "\n").lines() {
        if let Some(cont) = line.strip_prefix(' ').or_else(|| line.strip_prefix('\t')) {
            if let Some(prev) = unfolded.last_mut() {
                prev.push_str(cont);
                continue;
            }
        }
        unfolded.push(line.to_string());
    }

    let mut data = VCardData::default();
    for line in &unfolded {
        let Some((name_part, value)) = split_property(line) else {
            continue;
        };
        let mut params = name_part.split(';');
        let name = params.next().unwrap_or("").trim().to_ascii_uppercase();
        // Parametr moze byc w formie TYPE=CELL lub (vCard 2.1) golym CELL.
        let param_text = name_part.to_ascii_uppercase();

        match name.as_str() {
            "N" => {
                let parts = split_components(value);
                data.last_name = unescape(parts.first().map(String::as_str).unwrap_or(""))
                    .trim()
                    .to_string();
                data.first_name = unescape(parts.get(1).map(String::as_str).unwrap_or(""))
                    .trim()
                    .to_string();
            }
            "FN" => {
                if data.first_name.is_empty() && data.last_name.is_empty() {
                    let full = unescape(value);
                    let mut words = full.split_whitespace();
                    data.first_name = words.next().unwrap_or("").to_string();
                    data.last_name = words.collect::<Vec<_>>().join(" ");
                }
            }
            "ORG" => {
                let parts = split_components(value);
                data.company = unescape(parts.first().map(String::as_str).unwrap_or(""))
                    .trim()
                    .to_string();
            }
            "TITLE" => {
                if data.title_pl.is_empty() {
                    data.title_pl = unescape(value).trim().to_string();
                }
            }
            "TEL" => {
                let number = unescape(value).trim().to_string();
                if param_text.contains("CELL") {
                    if data.phone_mobile.is_empty() {
                        data.phone_mobile = number;
                    }
                } else if param_text.contains("FAX") {
                    if data.fax.is_empty() {
                        data.fax = number;
                    }
                } else if data.phone_work.is_empty() {
                    data.phone_work = number;
                }
            }
            "EMAIL" => {
                if data.email.is_empty() {
                    data.email = unescape(value).trim().to_string();
                }
            }
            "URL" => {
                if data.website.is_empty() {
                    data.website = unescape(value).trim().to_string();
                }
            }
            "ADR" if data.street.is_empty() && data.city.is_empty() => {
                let parts: Vec<String> = split_components(value)
                    .iter()
                    .map(|p| unescape(p))
                    .collect();
                // Skladowe ADR: skrytka; rozszerzony; ulica; miejscowosc;
                // region; kod pocztowy; kraj.
                data.street = parts.get(2).cloned().unwrap_or_default().trim().to_string();
                data.city = parts.get(3).cloned().unwrap_or_default().trim().to_string();
                data.postal_code = parts.get(5).cloned().unwrap_or_default().trim().to_string();
                data.country = parts.get(6).cloned().unwrap_or_default().trim().to_string();
            }
            _ => {}
        }
    }
    Some(data)
}

/// Dzieli wartosc zlozona na skladowe po nieescapowanych srednikach
/// (";" w tresci jest zapisywany jako "\;"). Skladowe pozostaja escapowane.
fn split_components(value: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                current.push('\\');
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            ';' => parts.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    parts.push(current);
    parts
}

/// Dzieli linie wlasciwosci na (nazwa+parametry, wartosc) po pierwszym ':'
/// nie bedacym czescia parametru w cudzyslowie.
fn split_property(line: &str) -> Option<(&str, &str)> {
    let mut in_quotes = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            ':' if !in_quotes => return Some((&line[..i], &line[i + 1..])),
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> VCardData {
        VCardData {
            first_name: "Jan".to_string(),
            last_name: "Kowalski".to_string(),
            phone_work: "58 307 00 00".to_string(),
            phone_mobile: "600 000 000".to_string(),
            email: "jan.kowalski@firma.pl".to_string(),
            website: "https://firma.pl".to_string(),
            company: "Firma Sp. z o.o.".to_string(),
            title_pl: "Kierownik Logistyki Dostaw".to_string(),
            title_en: "Supply Chain Manager".to_string(),
            fax: "58 307 00 01".to_string(),
            street: "ul. Portowa 1".to_string(),
            city: "Gdańsk".to_string(),
            postal_code: "80-001".to_string(),
            country: "Polska".to_string(),
        }
    }

    #[test]
    fn payload_contains_expected_lines() {
        let text = sample().payload().unwrap();
        assert!(text.starts_with("BEGIN:VCARD\r\nVERSION:3.0"));
        assert!(text.contains("N:Kowalski;Jan;;;"));
        assert!(text.contains("FN:Jan Kowalski"));
        assert!(text.contains("TEL;TYPE=CELL:600 000 000"));
        assert!(text.contains("ADR;TYPE=WORK:;;ul. Portowa 1;Gdańsk;;80-001;Polska"));
        assert!(text.ends_with("END:VCARD"));
        // title_en nie trafia do vCard
        assert!(!text.contains("Supply Chain"));
    }

    #[test]
    fn empty_fields_omitted() {
        let data = VCardData {
            first_name: "Jan".to_string(),
            ..Default::default()
        };
        let text = data.payload().unwrap();
        assert!(!text.contains("ORG:"));
        assert!(!text.contains("TEL"));
        assert!(!text.contains("ADR"));
    }

    #[test]
    fn requires_name() {
        assert!(VCardData::default().payload().is_err());
    }

    #[test]
    fn escaping_special_chars() {
        let data = VCardData {
            first_name: "Jan".to_string(),
            company: "A;B,C\\D\nE".to_string(),
            ..Default::default()
        };
        let text = data.payload().unwrap();
        assert!(text.contains("ORG:A\\;B\\,C\\\\D\\nE"));
    }

    #[test]
    fn parse_roundtrip() {
        let original = sample();
        let parsed = parse_vcard(&original.payload().unwrap()).unwrap();
        // title_en nie jest kodowane, wiec porownujemy bez niego
        let mut expected = original.clone();
        expected.title_en = String::new();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_folded_lines() {
        let raw = "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Kowal\r\n ski;Jan;;;\r\nEND:VCARD";
        let parsed = parse_vcard(raw).unwrap();
        assert_eq!(parsed.last_name, "Kowalski");
    }

    #[test]
    fn parse_rejects_non_vcard() {
        assert!(parse_vcard("https://example.com").is_none());
    }
}
