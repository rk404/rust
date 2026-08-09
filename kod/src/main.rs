// Na Windows w trybie release ukrywamy konsole (brak "mrugniecia" czarnym
// oknem przy podwojnym kliknieciu). W trybie debug konsola zostaje odkryta,
// co ulatwia programowanie. Nie ma to wplywu na Linux/macOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use qrgen_rust::formats::{FormatKind, FormsState};
use qrgen_rust::output::{self, SaveRequest};
use qrgen_rust::settings::Settings;
use qrgen_rust::{cli, gui, platform, settings};

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

    let settings = settings::load();

    if cli_args.silent {
        platform::attach_console();
        run_silent(&cli_args, &settings);
        return;
    }

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

/// Tryb --silent: generowanie plikow bez okna (wywolania z ERP/skryptow).
/// Tresc pochodzi wylacznie z argumentow i --input (nie ze stanu GUI).
fn run_silent(cli_args: &cli::CliArgs, settings: &Settings) {
    let job = match cli::build_job(cli_args, settings, FormsState::default()) {
        Ok(job) => job,
        Err(e) => {
            eprintln!("Błąd: {e}");
            std::process::exit(2);
        }
    };

    let Some(format) = job.format else {
        eprintln!("Błąd: podaj dane do zakodowania (tekst pozycyjny, --format z polami lub --input; --help wyświetla pomoc)");
        std::process::exit(2);
    };

    let result = (|| -> Result<output::SaveResult, String> {
        let payload = job.forms.payload(format)?;
        let stem = match format {
            FormatKind::VCard => output::vcard_stem(&job.forms.vcard),
            _ => settings.file_stem.clone(),
        };
        let vcard_for_docx =
            (format == FormatKind::VCard && job.docx).then(|| job.forms.vcard.clone());
        let request = SaveRequest {
            payload: &payload,
            appearance: &job.appearance,
            output: job.output.as_deref(),
            // W trybie konsolowym domyslnym katalogiem pozostaje katalog
            // roboczy (jak dotychczas), nie folder Dokumenty.
            default_dir: std::env::current_dir().unwrap_or_else(|_| ".".into()),
            stem: &stem,
            extension: &settings.image_ext,
            docx: vcard_for_docx.as_ref(),
            overwrite: job.overwrite,
        };
        output::generate_and_save(&request)
    })();

    match result {
        Ok(saved) => {
            println!("Zapisano: {}", saved.image_path.display());
            if let Some(docx) = saved.docx_path {
                println!("Zapisano: {}", docx.display());
            }
        }
        Err(e) => {
            eprintln!("Błąd: {e}");
            std::process::exit(1);
        }
    }
}
