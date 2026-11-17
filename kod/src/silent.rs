// Tryb konsolowy (--silent): generowanie plikow bez okna, dla ERP i skryptow.
// Wspolny dla qrgen_rust --silent oraz osobnego pliku qrgen-cli.

use crate::cli::{self, CliArgs};
use crate::formats::{FormatKind, FormsState};
use crate::output::{self, SaveRequest};
use crate::settings::{self, Settings};

/// Kod wyjscia: bledne argumenty lub JSON.
pub const EXIT_BAD_INPUT: i32 = 2;
/// Kod wyjscia: blad generowania lub zapisu.
pub const EXIT_FAILED: i32 = 1;

/// Ustawienia dla trybu konsolowego: domyslne, chyba ze podano --use-settings.
/// Dzieki temu to samo wywolanie z ERP daje ten sam wynik na kazdym komputerze.
pub fn settings_for(cli_args: &CliArgs) -> Settings {
    if cli_args.use_settings {
        settings::load()
    } else {
        Settings::default()
    }
}

/// Wykonuje zlecenie i zwraca kod wyjscia procesu. Tresc pochodzi wylacznie
/// z argumentow i --input (nie ze stanu GUI).
pub fn run(cli_args: &CliArgs, settings: &Settings) -> i32 {
    let job = match cli::build_job(cli_args, settings, FormsState::default()) {
        Ok(job) => job,
        Err(e) => {
            eprintln!("Błąd: {e}");
            return EXIT_BAD_INPUT;
        }
    };

    let Some(format) = job.format else {
        eprintln!("Błąd: podaj dane do zakodowania (tekst pozycyjny, --format z polami lub --input; --help wyświetla pomoc)");
        return EXIT_BAD_INPUT;
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
            // roboczy, nie folder Dokumenty.
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
            0
        }
        Err(e) => {
            eprintln!("Błąd: {e}");
            EXIT_FAILED
        }
    }
}

/// Pelny przebieg dla pliku wykonywalnego konsolowego (qrgen-cli):
/// zawsze tryb --silent, bez GUI.
pub fn main_cli(args: &[String]) -> i32 {
    let cli_args = match cli::parse(args) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Błąd: {e}");
            return EXIT_BAD_INPUT;
        }
    };
    if cli_args.help {
        println!("{}", cli::usage());
        return 0;
    }
    if args.len() <= 1 {
        eprintln!("{}", cli::usage());
        return EXIT_BAD_INPUT;
    }
    let settings = settings_for(&cli_args);
    run(&cli_args, &settings)
}
