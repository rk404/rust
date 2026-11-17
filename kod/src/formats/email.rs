use serde::{Deserialize, Serialize};

/// Wiadomosc e-mail - kodowana jako URI mailto: (RFC 6068).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EmailForm {
    pub to: String,
    pub subject: String,
    pub body: String,
}

/// Procentowe kodowanie UTF-8: wszystko poza znakami "unreserved" z RFC 3986
/// (A-Z a-z 0-9 - . _ ~). Nowe linie w tresci koduja sie jako %0D%0A po
/// wczesniejszej normalizacji w payload().
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*byte as char)
            }
            _ => {
                use std::fmt::Write as _;
                let _ = write!(&mut out, "%{:02X}", byte);
            }
        }
    }
    out
}

/// Odwrotnosc `percent_encode`: dekoduje %XX do bajtow UTF-8 (bledne ciagi
/// zostawia bez zmian).
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            // `get` zamiast indeksowania: nie panikuje na granicy znaku UTF-8.
            if let Some(v) = s
                .get(i + 1..i + 3)
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
            {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

impl EmailForm {
    /// Parsuje URI `mailto:` (np. odczytane z kodu QR) na pola formularza.
    /// None, gdy tekst nie jest URI mailto.
    pub fn parse_mailto(raw: &str) -> Option<EmailForm> {
        let raw = raw.trim();
        let rest = raw
            .get(..7)
            .filter(|head| head.eq_ignore_ascii_case("mailto:"))
            .map(|_| &raw[7..])?;
        let (to, query) = rest.split_once('?').unwrap_or((rest, ""));
        let mut form = EmailForm {
            to: percent_decode(to),
            ..Default::default()
        };
        for pair in query.split('&') {
            let Some((key, value)) = pair.split_once('=') else {
                continue;
            };
            match key.to_ascii_lowercase().as_str() {
                "subject" => form.subject = percent_decode(value),
                "body" => form.body = percent_decode(value).replace("\r\n", "\n"),
                _ => {}
            }
        }
        Some(form)
    }

    pub fn payload(&self) -> Result<String, String> {
        let to = self.to.trim();
        if to.is_empty() {
            return Err("wpisz adres e-mail odbiorcy".to_string());
        }

        let mut uri = format!("mailto:{}", percent_encode(to));
        let mut params = Vec::new();
        if !self.subject.trim().is_empty() {
            params.push(format!("subject={}", percent_encode(self.subject.trim())));
        }
        if !self.body.is_empty() {
            // RFC 6068 wymaga w tresci konca linii jako CRLF (%0D%0A).
            let normalized = self.body.replace("\r\n", "\n").replace('\n', "\r\n");
            params.push(format!("body={}", percent_encode(&normalized)));
        }
        if !params.is_empty() {
            uri.push('?');
            uri.push_str(&params.join("&"));
        }
        Ok(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mailto_roundtrip_through_parse() {
        let form = EmailForm {
            to: "jan@firma.pl".into(),
            subject: "Oferta: żółć & co".into(),
            body: "Dzień dobry,\nPozdrawiam".into(),
        };
        let parsed = EmailForm::parse_mailto(&form.payload().unwrap()).unwrap();
        assert_eq!(parsed.to, form.to);
        assert_eq!(parsed.subject, form.subject);
        assert_eq!(parsed.body, form.body);
        assert!(EmailForm::parse_mailto("https://firma.pl").is_none());
        assert_eq!(EmailForm::parse_mailto("MAILTO:a@b.pl").unwrap().to, "a@b.pl");
    }

    #[test]
    fn percent_decode_survives_garbage() {
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz%41"), "%zzA");
        assert_eq!(percent_decode("%ą"), "%ą");
    }

    #[test]
    fn plain_address_only() {
        let form = EmailForm {
            to: "jan@firma.pl".to_string(),
            ..Default::default()
        };
        assert_eq!(form.payload().unwrap(), "mailto:jan%40firma.pl");
    }

    #[test]
    fn subject_and_body_encoded() {
        let form = EmailForm {
            to: "a@b.pl".to_string(),
            subject: "Oferta 50% & więcej?".to_string(),
            body: "Dzień dobry,\nPozdrawiam".to_string(),
        };
        let uri = form.payload().unwrap();
        assert!(uri.contains("subject=Oferta%2050%25%20%26%20wi%C4%99cej%3F"));
        assert!(uri.contains("body=Dzie%C5%84%20dobry%2C%0D%0APozdrawiam"));
    }

    #[test]
    fn missing_to_errors() {
        assert!(EmailForm::default().payload().is_err());
    }
}
