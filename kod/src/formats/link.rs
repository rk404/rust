use serde::{Deserialize, Serialize};

/// Adres strony WWW.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkForm {
    pub url: String,
}

impl LinkForm {
    /// Znormalizowany adres: gdy brak schematu (http://, ftp://, ...),
    /// doklejamy https:// - takie adresy wpisuje sie najczesciej.
    pub fn normalized_url(&self) -> String {
        let url = self.url.trim();
        if url.is_empty() || url.contains("://") || url.starts_with("mailto:") {
            url.to_string()
        } else {
            format!("https://{url}")
        }
    }

    /// Czy payload() dokleji https:// (informacja dla interfejsu).
    pub fn will_add_scheme(&self) -> bool {
        let url = self.url.trim();
        !url.is_empty() && !url.contains("://") && !url.starts_with("mailto:")
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
    fn empty_errors() {
        assert!(link("  ").payload().is_err());
    }
}
