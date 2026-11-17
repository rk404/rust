use serde::{Deserialize, Serialize};

/// Adres strony WWW.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkForm {
    pub url: String,
}

/// Schematy URI, ktore zwykle wpisuje sie bez "//" (tel:, sms:, geo: ...).
const KNOWN_SCHEMES: [&str; 14] = [
    "mailto", "tel", "sms", "smsto", "mms", "geo", "wifi", "facetime", "skype", "market",
    "urn", "data", "bitcoin", "sip",
];

/// Czy wpis ma juz schemat URI (wtedy nie doklejamy https://).
/// Rozpoznajemy "schemat://...", znane schematy bez "//" (tel:, mailto:, ...)
/// oraz ogolny "schemat:reszta" - z wyjatkiem "host:port" (np. localhost:8080).
fn has_scheme(url: &str) -> bool {
    if url.contains("://") {
        return true;
    }
    let Some((scheme, rest)) = url.split_once(':') else {
        return false;
    };
    let valid_scheme = scheme.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'));
    if !valid_scheme {
        return false;
    }
    if KNOWN_SCHEMES.contains(&scheme.to_ascii_lowercase().as_str()) {
        return true;
    }
    // "host:8080" lub "host:8080/sciezka" to adres z portem, nie schemat.
    let port_len = rest.chars().take_while(|c| c.is_ascii_digit()).count();
    let looks_like_port = port_len > 0 && matches!(rest[port_len..].chars().next(), None | Some('/'));
    !looks_like_port && !rest.is_empty()
}

impl LinkForm {
    /// Czy odczytany tekst wyglada na pojedynczy adres z jawnym schematem
    /// (http://, https://, ftp:// ...). Takie dane trafiaja do formularza Link.
    pub fn looks_like_url(raw: &str) -> bool {
        let raw = raw.trim();
        !raw.is_empty() && !raw.contains(char::is_whitespace) && raw.contains("://")
    }

    /// Znormalizowany adres: gdy brak schematu, doklejamy https:// -
    /// takie adresy wpisuje sie najczesciej.
    pub fn normalized_url(&self) -> String {
        let url = self.url.trim();
        if url.is_empty() || has_scheme(url) {
            url.to_string()
        } else {
            format!("https://{url}")
        }
    }

    /// Czy payload() dokleji https:// (informacja dla interfejsu).
    pub fn will_add_scheme(&self) -> bool {
        let url = self.url.trim();
        !url.is_empty() && !has_scheme(url)
    }

    pub fn payload(&self) -> Result<String, String> {
        let url = self.normalized_url();
        if url.is_empty() {
            return Err("wpisz adres strony (URL)".to_string());
        }
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(url: &str) -> LinkForm {
        LinkForm {
            url: url.to_string(),
        }
    }

    #[test]
    fn adds_https_when_missing() {
        assert_eq!(link("example.com").payload().unwrap(), "https://example.com");
        assert!(link("example.com").will_add_scheme());
    }

    #[test]
    fn keeps_existing_scheme() {
        assert_eq!(
            link("http://example.com").payload().unwrap(),
            "http://example.com"
        );
        assert!(!link("http://example.com").will_add_scheme());
    }

    #[test]
    fn keeps_schemes_without_slashes() {
        for url in [
            "tel:+48123456789",
            "tel:123456789",
            "sms:+48123456789",
            "mailto:jan@firma.pl",
            "geo:54.35,18.65",
            "WIFI:T:WPA;S:siec;P:haslo;;",
            "javascript:alert(1)",
        ] {
            assert_eq!(link(url).payload().unwrap(), url, "{url}");
            assert!(!link(url).will_add_scheme(), "{url}");
        }
    }

    #[test]
    fn host_with_port_still_gets_https() {
        assert_eq!(
            link("localhost:8080").payload().unwrap(),
            "https://localhost:8080"
        );
        assert_eq!(
            link("firma.pl:8443/panel").payload().unwrap(),
            "https://firma.pl:8443/panel"
        );
    }

    #[test]
    fn detects_urls_in_decoded_text() {
        assert!(LinkForm::looks_like_url("https://firma.pl/oferta?a=1"));
        assert!(LinkForm::looks_like_url("  ftp://serwer/plik "));
        assert!(!LinkForm::looks_like_url("zwykly tekst https://firma.pl"));
        assert!(!LinkForm::looks_like_url("tel:+48123456789"));
        assert!(!LinkForm::looks_like_url(""));
    }

    #[test]
    fn empty_errors() {
        assert!(link("  ").payload().is_err());
    }
}
