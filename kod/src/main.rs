// Na Windows w trybie release ukrywamy konsole (brak "mrugniecia" czarnym
// oknem przy podwojnym kliknieciu). W trybie debug konsola zostaje odkryta,
// co ulatwia programowanie. Nie ma to wplywu na Linux/macOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use qrgen_rust::{cli, gui, platform, settings, silent};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cli_args = match cli::parse(&args) {
        Ok(parsed) => parsed,
        Err(e) => {
            platform::attach_console();
            eprintln!("Błąd: {e}");
            std::process::exit(2);
        }
    };

    if cli_args.help {
        platform::attach_console();
        println!("{}", cli::usage());
        return;
    }

    if cli_args.silent {
        platform::attach_console();
        let settings = silent::settings_for(&cli_args);
        std::process::exit(silent::run(&cli_args, &settings));
    }

    let settings = settings::load();

    // Tryb GUI: formularze startuja od stanu zapisanego w ustawieniach,
    // nadpisanego ewentualnymi danymi z CLI/JSON.
    match cli::build_job(&cli_args, &settings, settings.forms.clone()) {
        Ok(job) => gui::run_gui(job, settings),
        Err(e) => {
            platform::attach_console();
            eprintln!("Błąd: {e}");
            std::process::exit(2);
        }
    }
}
