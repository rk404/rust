pub mod email;
pub mod link;
pub mod text;
pub mod vcard;

pub use email::EmailForm;
pub use link::LinkForm;
pub use text::TextForm;
pub use vcard::VCardData;

use serde::{Deserialize, Serialize};

/// Rodzaj kodowanej tresci - jedna zakladka formularza na kazdy wariant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatKind {
    Text,
    Link,
    Email,
    #[serde(rename = "vcard")]
    VCard,
}

impl FormatKind {
    pub const ALL: [FormatKind; 4] = [
        FormatKind::Text,
        FormatKind::VCard,
        FormatKind::Link,
        FormatKind::Email,
    ];

    /// Etykieta wyswietlana w interfejsie.
    pub fn label(self) -> &'static str {
        match self {
            FormatKind::Text => "Tekst",
            FormatKind::Link => "Link",
            FormatKind::Email => "E-mail",
            FormatKind::VCard => "vCard",
        }
    }

    /// Ikona formatu (glify dostepne w fontach emoji egui).
    pub fn icon(self) -> &'static str {
        match self {
            FormatKind::Text => "📝",
            FormatKind::Link => "🔗",
            FormatKind::Email => "✉",
            FormatKind::VCard => "📇",
        }
    }

    /// Krotki opis pokazywany na kafelku ekranu startowego.
    pub fn subtitle(self) -> &'static str {
        match self {
            FormatKind::Text => "dowolny tekst",
            FormatKind::Link => "adres strony WWW",
            FormatKind::Email => "wiadomość e-mail",
            FormatKind::VCard => "wizytówka / dane kontaktowe",
        }
    }

    /// Wartosc przyjmowana przez --format oraz pole "format" w JSON.
    pub fn from_cli(s: &str) -> Option<FormatKind> {
        match s.trim().to_ascii_lowercase().as_str() {
            "text" | "tekst" => Some(FormatKind::Text),
            "link" | "url" => Some(FormatKind::Link),
            "email" | "e-mail" | "mail" => Some(FormatKind::Email),
            "vcard" | "v-card" => Some(FormatKind::VCard),
            _ => None,
        }
    }
}

/// Stan wszystkich formularzy naraz - przelaczanie zakladek nie gubi danych.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FormsState {
    pub text: TextForm,
    pub link: LinkForm,
    pub email: EmailForm,
    pub vcard: VCardData,
}

impl FormsState {
    /// Buduje tresc do zakodowania dla aktywnego formatu.
    /// Blad (po polsku), gdy brakuje wymaganych pol.
    pub fn payload(&self, kind: FormatKind) -> Result<String, String> {
        match kind {
            FormatKind::Text => self.text.payload(),
            FormatKind::Link => self.link.payload(),
            FormatKind::Email => self.email.payload(),
            FormatKind::VCard => self.vcard.payload(),
        }
    }

    /// Czy formularz danego formatu ma jakiekolwiek dane (np. przekazane z CLI).
    pub fn has_data(&self, kind: FormatKind) -> bool {
        match kind {
            FormatKind::Text => !self.text.data.trim().is_empty(),
            FormatKind::Link => !self.link.url.trim().is_empty(),
            FormatKind::Email => !self.email.to.trim().is_empty(),
            FormatKind::VCard => {
                !self.vcard.first_name.trim().is_empty()
                    || !self.vcard.last_name.trim().is_empty()
            }
        }
    }
}
