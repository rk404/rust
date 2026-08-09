use serde::{Deserialize, Serialize};

/// Dowolny tekst - kodowany bez zmian.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TextForm {
    pub data: String,
}

impl TextForm {
    pub fn payload(&self) -> Result<String, String> {
        if self.data.trim().is_empty() {
            return Err("wpisz tekst do zakodowania".to_string());
        }
        Ok(self.data.clone())
    }
}
