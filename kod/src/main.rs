// Na Windows w trybie release ukrywamy konsole (brak "mrugniecia" czarnym
// oknem przy podwojnym kliknieciu). W trybie debug konsola zostaje odkryta,
// co ulatwia programowanie. Nie ma to wplywu na Linux/macOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gui;
mod qr_logic;

use qr_logic::{ecc_from_letter, generate_and_save, parse_hex_color, Options};

pub struct ParsedArgs {
    pub opts: Options,
    pub silent: bool,
    pub help: bool,
    pub has_data: bool,
}

fn parse_args(args: &[String]) -> ParsedArgs {
    let mut opts = Options::default();
    let mut silent = false;
    let mut help = false;
    let mut has_data = false;

    let mut i = 1; // args[0] = nazwa programu
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "-h" | "--help" => help = true,
            "--silent" | "-s" => silent = true,
            "-o" | "--output" if i + 1 < args.len() => {
                i += 1;
                opts.output = args[i].clone();
            }
            "--scale" if i + 1 < args.len() => {
                i += 1;
                opts.scale = args[i].parse().unwrap_or(opts.scale);
            }
            "--border" if i + 1 < args.len() => {
                i += 1;
                opts.border = args[i].parse().unwrap_or(opts.border);
            }
            "--error" if i + 1 < args.len() => {
                i += 1;
                if let Some(e) = ecc_from_letter(&args[i]) {
                    opts.ecc = e;
                }
            }
            "--fill" if i + 1 < args.len() => {
                i += 1;
                if let Some(c) = parse_hex_color(&args[i]) {
                    opts.fill = c;
                }
            }
            "--back" if i + 1 < args.len() => {
                i += 1;
                if let Some(c) = parse_hex_color(&args[i]) {
                    opts.back = c;
                }
            }
            _ if !a.starts_with('-') => {
                opts.data = a.to_string();
                has_data = true;
            }
            _ => { /* nieznana flaga - ignorujemy */ }
        }
        i += 1;
    }

    ParsedArgs { opts, silent, help, has_data }
}

fn print_usage() {
    println!("Uzycie: qrgen \"dane\" [opcje]\n");
    println!("Bez argumentow uruchamia interfejs graficzny.");
    println!("Z argumentami (bez --silent) uruchamia GUI z uzupelnionymi polami.");
    println!("Z argumentami i --silent generuje plik bez okna.\n");
    println!("Opcje:");
    println!("  -o, --output PLIK     Nazwa pliku wynikowego (domyslnie: qrcode.png)");
    println!("  --scale N             Rozmiar modulu w pikselach (domyslnie: 10)");
    println!("  --border N            Grubosc bialej ramki w modulach (domyslnie: 4)");
    println!("  --error L|M|Q|H       Poziom korekcji bledow (domyslnie: M)");
    println!("  --fill RRGGBB         Kolor kodu w hex (domyslnie: 000000)");
    println!("  --back RRGGBB         Kolor tla w hex (domyslnie: FFFFFF)");
    println!("  --silent              Dziala bez GUI (tryb konsolowy)");
    println!("  -h, --help            Wyswietla te pomoc");
    println!("\nRozszerzenie pliku wyjsciowego decyduje o formacie (.png, .bmp).");
}

/// Na Windows aplikacja jest budowana z podsystemem "windows" (patrz atrybut
/// na gorze pliku), co oznacza, ze nie ma wlasnej konsoli. Gdy jest
/// uruchamiana z cmd.exe z flaga --silent/--help, doczepiamy sie do konsoli
/// procesu wywolujacego, zeby println!/eprintln! byly widoczne.
/// Na Linuksie/macOS to zwykla, dostepna od razu konsola - funkcja nic nie robi.
#[cfg(windows)]
fn attach_console() {
    use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
fn attach_console() {}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let parsed = parse_args(&args);

    if parsed.help {
        attach_console();
        print_usage();
        return;
    }

    if parsed.silent {
        attach_console();
        match generate_and_save(&parsed.opts) {
            Ok(msg) => println!("{}", msg),
            Err(e) => {
                eprintln!("Blad: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Bez --silent: zawsze uruchamiamy GUI (puste lub uzupelnione danymi).
    gui::run_gui(parsed.opts, parsed.has_data);
}
