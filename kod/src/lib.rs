// Logika aplikacji qrgen jako biblioteka - dzieki temu moduly sa dostepne
// w testach integracyjnych (tests/), a main.rs pozostaje cienkim wejsciem.

pub mod cli;
pub mod decode;
pub mod docx;
pub mod formats;
pub mod gui;
pub mod input_json;
pub mod output;
pub mod platform;
pub mod render;
pub mod settings;
