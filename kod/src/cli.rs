use crate::formats::{FormatKind, FormsState};
use crate::input_json;
use crate::render::{self, Appearance};
use crate::settings::Settings;
use qrcode::EcLevel;
use std::path::PathBuf;

/// Argumenty wiersza polecen. Kazde pole jest Option - dzieki temu wiadomo,
/// co uzytkownik podal jawnie (i tylko to nadpisuje ustawienia/JSON).
#[derive(Debug, Default)]
pub struct CliArgs {
    pub help: bool,
    pub silent: bool,
    /// Okno deweloperskie do strojenia interfejsu (tylko GUI).
    pub debug_ui: bool,
    pub format: Option<FormatKind>,
    pub input_json: Option<PathBuf>,
    pub output: Option<String>,
    pub overwrite: Option<bool>,
    pub docx: Option<bool>,

    pub scale: Option<u32>,
    pub border: Option<u32>,
    /// Docelowy bok obrazu w px (nadpisuje --scale).
    pub size: Option<u32>,
    pub ecc: Option<EcLevel>,
    pub fill: Option<[u8; 3]>,
    pub back: Option<[u8; 3]>,

    /// Argument pozycyjny - dane w trybie tekstowym (kompatybilnosc wstecz).
    pub text_data: Option<String>,
    pub url: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,

    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub title_en: Option<String>,
    pub fax: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

/// Parsuje argumenty. Nieznana flaga lub bledna wartosc to twardy blad
/// (komunikat po polsku) - zadnego cichego ignorowania.
pub fn parse(args: &[String]) -> Result<CliArgs, String> {
    let mut cli = CliArgs::default();
    let mut i = 1; // args[0] = nazwa programu

    fn value<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
        *i += 1;
        args.get(*i)
            .map(|s| s.as_str())
            .ok_or_else(|| format!("flaga {flag} wymaga wartosci"))
    }

    let mut only_positional = false;
    while i < args.len() {
        let a = args[i].as_str();
        if only_positional || (!a.starts_with('-') && a != "--") {
            set_positional(&mut cli, a)?;
            i += 1;
            continue;
        }
        match a {
            "--" => only_positional = true,
            "-h" | "--help" => cli.help = true,
            "-s" | "--silent" => cli.silent = true,
            "--debug-ui" => cli.debug_ui = true,
            "--overwrite" => cli.overwrite = Some(true),
            "--no-docx" => cli.docx = Some(false),
            "--format" => {
                let v = value(args, &mut i, a)?;
                cli.format = Some(FormatKind::from_cli(v).ok_or_else(|| {
                    format!("bledna wartosc --format: '{v}' (dozwolone: text, link, email, vcard)")
                })?);
            }
            "--input" => cli.input_json = Some(PathBuf::from(value(args, &mut i, a)?)),
            "-o" | "--output" => cli.output = Some(value(args, &mut i, a)?.to_string()),
            "--scale" => {
                let v = value(args, &mut i, a)?;
                cli.scale = Some(v.parse().map_err(|_| {
                    format!("bledna wartosc --scale: '{v}' (oczekiwano liczby calkowitej)")
                })?);
            }
            "--border" => {
                let v = value(args, &mut i, a)?;
                cli.border = Some(v.parse().map_err(|_| {
                    format!("bledna wartosc --border: '{v}' (oczekiwano liczby calkowitej)")
                })?);
            }
            "--size" => {
                let v = value(args, &mut i, a)?;
                cli.size = Some(v.parse().map_err(|_| {
                    format!("bledna wartosc --size: '{v}' (oczekiwano liczby calkowitej)")
                })?);
            }
            "--error" => {
                let v = value(args, &mut i, a)?;
                cli.ecc = Some(render::ecc_from_letter(v).ok_or_else(|| {
                    format!("bledna wartosc --error: '{v}' (dozwolone: L, M, Q, H)")
                })?);
            }
            "--fill" => {
                let v = value(args, &mut i, a)?;
                cli.fill = Some(render::parse_hex_color(v).ok_or_else(|| {
                    format!("bledna wartosc --fill: '{v}' (format RRGGBB)")
                })?);
            }
            "--back" => {
                let v = value(args, &mut i, a)?;
                cli.back = Some(render::parse_hex_color(v).ok_or_else(|| {
                    format!("bledna wartosc --back: '{v}' (format RRGGBB)")
                })?);
            }
            "--url" => cli.url = Some(value(args, &mut i, a)?.to_string()),
            "--to" => cli.to = Some(value(args, &mut i, a)?.to_string()),
            "--subject" => cli.subject = Some(value(args, &mut i, a)?.to_string()),
            "--body" => cli.body = Some(value(args, &mut i, a)?.to_string()),
            "--first-name" => cli.first_name = Some(value(args, &mut i, a)?.to_string()),
            "--last-name" => cli.last_name = Some(value(args, &mut i, a)?.to_string()),
            "--phone" => cli.phone = Some(value(args, &mut i, a)?.to_string()),
            "--mobile" => cli.mobile = Some(value(args, &mut i, a)?.to_string()),
            "--email" => cli.email = Some(value(args, &mut i, a)?.to_string()),
            "--website" => cli.website = Some(value(args, &mut i, a)?.to_string()),
            "--company" => cli.company = Some(value(args, &mut i, a)?.to_string()),
            "--title" => cli.title = Some(value(args, &mut i, a)?.to_string()),
            "--title-en" => cli.title_en = Some(value(args, &mut i, a)?.to_string()),
            "--fax" => cli.fax = Some(value(args, &mut i, a)?.to_string()),
            "--street" => cli.street = Some(value(args, &mut i, a)?.to_string()),
            "--city" => cli.city = Some(value(args, &mut i, a)?.to_string()),
            "--postal-code" => cli.postal_code = Some(value(args, &mut i, a)?.to_string()),
            "--country" => cli.country = Some(value(args, &mut i, a)?.to_string()),
            _ => return Err(format!("nieznana flaga: '{a}' (uzyj --help)")),
        }
        i += 1;
    }

    Ok(cli)
}

/// Zapisuje argument pozycyjny (dane tekstowe). Drugi argument pozycyjny to
/// blad - wczesniej cicho nadpisywal pierwszy.
fn set_positional(cli: &mut CliArgs, value: &str) -> Result<(), String> {
    if cli.text_data.is_some() {
        return Err(format!(
            "podano wiecej niz jedne dane pozycyjne (nadmiarowy argument: '{value}'); \
             ujmij tekst w cudzyslow lub uzyj -- przed danymi"
        ));
    }
    cli.text_data = Some(value.to_string());
    Ok(())
}

pub fn usage() -> String {
    [
        "Użycie: qrgen [DANE] [opcje]",
        "",
        "Bez argumentów uruchamia interfejs graficzny (ekran wyboru formatu).",
        "Z danymi (bez --silent) uruchamia GUI z wypełnionym formularzem i od razu generuje plik.",
        "Z danymi i --silent generuje pliki bez okna (tryb dla ERP/skryptów).",
        "",
        "Ogólne:",
        "  --format FORMAT       Format danych: text | link | email | vcard",
        "  --input PLIK.json     Wczytaj pola z pliku JSON (UTF-8) - zalecane dla ERP",
        "  -o, --output ŚCIEŻKA  Plik wynikowy (.jpg/.png/.bmp) LUB katalog docelowy",
        "  -s, --silent          Tryb bez okna (konsolowy)",
        "  --overwrite           Nadpisuj istniejące pliki (domyślnie: dokleja _2, _3, ...)",
        "  --no-docx             (vcard) nie generuj dokumentu Word",
        "  --                    koniec flag; reszta to dane tekstowe (np. tekst zaczynający się od -)",
        "  -h, --help            Wyświetla tę pomoc",
        "  --debug-ui            Okno deweloperskie GUI (podgląd układu, edycja stylu)",
        "",
        "Wygląd kodu:",
        "  --scale N             Rozmiar modułu w pikselach (domyślnie: 10)",
        "  --size N              Docelowy bok obrazu w px (nadpisuje --scale)",
        "  --border N            Grubość ramki w modułach (domyślnie: 4)",
        "  --error L|M|Q|H       Poziom korekcji błędów (domyślnie: M)",
        "  --fill RRGGBB         Kolor kodu w hex (domyślnie: 000000)",
        "  --back RRGGBB         Kolor tła w hex (domyślnie: FFFFFF)",
        "",
        "Pola vCard:",
        "  --first-name --last-name --phone --mobile --email --website --company",
        "  --title --title-en --fax --street --city --postal-code --country",
        "",
        "Pola e-mail:  --to --subject --body      Pole link:  --url",
        "",
        "Przykłady:",
        "  qrgen \"dowolny tekst\" --silent -o kod.png",
        "  qrgen --format vcard --first-name Jan --last-name Kowalski --silent",
        "  qrgen --silent --format vcard --input pracownik.json -o C:\\Wizytowki",
        "",
        "Dla formatu vcard obok obrazu QR powstaje dokument Word (.docx) z danymi",
        "do wizytówki; nazwy plików zawierają imię i nazwisko (np. Jan_Kowalski.jpg).",
    ]
    .join("\n")
}

impl CliArgs {
    /// Czy podano jawnie jakiekolwiek dane tresci (poza --input).
    pub fn has_inline_content(&self) -> bool {
        self.text_data.is_some()
            || self.url.is_some()
            || self.to.is_some()
            || self.first_name.is_some()
            || self.last_name.is_some()
    }

    /// Nadpisuje pola formularzy wartosciami podanymi jawnie w CLI.
    pub fn apply_to_forms(&self, forms: &mut FormsState) {
        macro_rules! set {
            ($src:expr, $dst:expr) => {
                if let Some(v) = &$src {
                    $dst = v.clone();
                }
            };
        }
        set!(self.text_data, forms.text.data);
        set!(self.url, forms.link.url);
        set!(self.to, forms.email.to);
        set!(self.subject, forms.email.subject);
        set!(self.body, forms.email.body);
        set!(self.first_name, forms.vcard.first_name);
        set!(self.last_name, forms.vcard.last_name);
        set!(self.phone, forms.vcard.phone_work);
        set!(self.mobile, forms.vcard.phone_mobile);
        set!(self.email, forms.vcard.email);
        set!(self.website, forms.vcard.website);
        set!(self.company, forms.vcard.company);
        set!(self.title, forms.vcard.title_pl);
        set!(self.title_en, forms.vcard.title_en);
        set!(self.fax, forms.vcard.fax);
        set!(self.street, forms.vcard.street);
        set!(self.city, forms.vcard.city);
        set!(self.postal_code, forms.vcard.postal_code);
        set!(self.country, forms.vcard.country);
    }

    pub fn apply_appearance(&self, ap: &mut Appearance) {
        if let Some(v) = self.scale {
            ap.scale = v;
        }
        if let Some(v) = self.border {
            ap.border = v;
        }
        if let Some(v) = self.size {
            ap.target_px = v;
        }
        if let Some(v) = self.ecc {
            ap.ecc = v;
        }
        if let Some(v) = self.fill {
            ap.fill = v;
        }
        if let Some(v) = self.back {
            ap.back = v;
        }
    }
}

/// Scalone zlecenie po zastosowaniu precedencji:
/// domyślne <- settings.json <- --input JSON <- jawne flagi CLI.
pub struct Job {
    pub format: Option<FormatKind>,
    /// Pokaz okno deweloperskie w GUI (--debug-ui).
    pub debug_ui: bool,
    pub forms: FormsState,
    pub appearance: Appearance,
    /// Jawna sciezka wyjsciowa (-o lub "output" z JSON).
    pub output: Option<String>,
    pub overwrite: bool,
    pub docx: bool,
    /// Czy CLI/JSON dostarczyly dane tresci (GUI: generuj od razu na starcie).
    pub content_provided: bool,
}

/// Buduje zlecenie z argumentow CLI, ewentualnego pliku --input i ustawien.
/// `base_forms`: w GUI - formularze przywrocone z ustawien; w --silent - puste
/// (tresc moze pochodzic wylacznie z CLI/JSON, nie ze stanu GUI).
pub fn build_job(cli: &CliArgs, settings: &Settings, base_forms: FormsState) -> Result<Job, String> {
    let json = match &cli.input_json {
        Some(path) => Some(input_json::load(path)?),
        None => None,
    };

    let mut forms = base_forms;
    let mut appearance = settings.appearance();
    let mut content_provided = cli.has_inline_content();

    if let Some(json) = &json {
        json.apply_to_forms(&mut forms);
        json.apply_appearance(&mut appearance)?;
        content_provided = content_provided || json.has_content();
    }
    cli.apply_to_forms(&mut forms);
    cli.apply_appearance(&mut appearance);

    // Format: flaga CLI > JSON > (pozycyjne dane => text) > brak.
    let format = cli
        .format
        .or(json.as_ref().and_then(|j| j.format))
        .or(cli.text_data.as_ref().map(|_| FormatKind::Text));

    if let (Some(data), Some(kind)) = (&cli.text_data, format) {
        if kind != FormatKind::Text {
            return Err(format!(
                "dane pozycyjne ('{data}') dotycza tylko formatu text, a wybrano --format {}; \
                 uzyj flag pol tego formatu",
                kind.cli_name()
            ));
        }
    }

    let output = cli
        .output
        .clone()
        .or_else(|| json.as_ref().and_then(|j| j.output.clone()));
    let overwrite = cli
        .overwrite
        .or(json.as_ref().and_then(|j| j.overwrite))
        .unwrap_or(settings.overwrite);
    let docx = cli
        .docx
        .or(json.as_ref().and_then(|j| j.docx))
        .unwrap_or(true);

    Ok(Job {
        format,
        debug_ui: cli.debug_ui,
        forms,
        appearance,
        output,
        overwrite,
        docx,
        content_provided,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        std::iter::once("qrgen".to_string())
            .chain(list.iter().map(|s| s.to_string()))
            .collect()
    }

    #[test]
    fn positional_data_backward_compat() {
        let cli = parse(&args(&["moje dane", "-o", "plik.png", "--silent"])).unwrap();
        assert_eq!(cli.text_data.as_deref(), Some("moje dane"));
        assert_eq!(cli.output.as_deref(), Some("plik.png"));
        assert!(cli.silent);
    }

    #[test]
    fn second_positional_is_error() {
        let err = parse(&args(&["a", "b"])).unwrap_err();
        assert!(err.contains("wiecej niz jedne"));
    }

    #[test]
    fn double_dash_allows_dash_prefixed_text() {
        let cli = parse(&args(&["--silent", "--", "-tekst z minusem"])).unwrap();
        assert_eq!(cli.text_data.as_deref(), Some("-tekst z minusem"));
        // po -- nawet flagi sa danymi
        let cli = parse(&args(&["--", "--silent"])).unwrap();
        assert!(!cli.silent);
        assert_eq!(cli.text_data.as_deref(), Some("--silent"));
    }

    #[test]
    fn positional_with_non_text_format_is_error() {
        let cli = parse(&args(&["--format", "vcard", "dane"])).unwrap();
        let err = build_job(&cli, &Settings::default(), FormsState::default())
            .err()
            .unwrap();
        assert!(err.contains("vcard"));
        let cli = parse(&args(&["--format", "text", "dane"])).unwrap();
        assert!(build_job(&cli, &Settings::default(), FormsState::default()).is_ok());
    }

    #[test]
    fn unknown_flag_errors() {
        assert!(parse(&args(&["--nieznana"])).unwrap_err().contains("nieznana"));
    }

    #[test]
    fn bad_scale_errors() {
        assert!(parse(&args(&["--scale", "abc"])).unwrap_err().contains("--scale"));
    }

    #[test]
    fn missing_value_errors() {
        assert!(parse(&args(&["--output"])).unwrap_err().contains("wymaga wartosci"));
    }

    #[test]
    fn vcard_flags_applied() {
        let cli = parse(&args(&[
            "--format", "vcard", "--first-name", "Jan", "--last-name", "Kowalski",
        ]))
        .unwrap();
        assert_eq!(cli.format, Some(FormatKind::VCard));
        let mut forms = FormsState::default();
        cli.apply_to_forms(&mut forms);
        assert_eq!(forms.vcard.first_name, "Jan");
        assert!(cli.has_inline_content());
    }

    #[test]
    fn job_precedence_cli_over_settings() {
        let cli = parse(&args(&["--scale", "20", "dane"])).unwrap();
        let settings = Settings {
            scale: 5,
            border: 9,
            ..Default::default()
        };
        let job = build_job(&cli, &settings, FormsState::default()).unwrap();
        assert_eq!(job.appearance.scale, 20); // CLI wygrywa
        assert_eq!(job.appearance.border, 9); // z ustawien
        assert_eq!(job.format, Some(FormatKind::Text)); // pozycyjne dane => text
        assert!(job.content_provided);
    }

    #[test]
    fn job_json_between_settings_and_cli() {
        let dir = std::env::temp_dir().join(format!("qrgen_cli_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let json_path = dir.join("input.json");
        std::fs::write(
            &json_path,
            r#"{ "format": "vcard", "appearance": { "scale": 15 },
                 "vcard": { "first_name": "Anna", "last_name": "Nowak" } }"#,
        )
        .unwrap();

        let cli = parse(&args(&[
            "--input",
            json_path.to_str().unwrap(),
            "--first-name",
            "Ewa",
        ]))
        .unwrap();
        let job = build_job(&cli, &Settings::default(), FormsState::default()).unwrap();
        assert_eq!(job.format, Some(FormatKind::VCard));
        assert_eq!(job.appearance.scale, 15);
        assert_eq!(job.forms.vcard.first_name, "Ewa"); // flaga CLI nadpisuje JSON
        assert_eq!(job.forms.vcard.last_name, "Nowak");
        std::fs::remove_dir_all(&dir).ok();
    }
}
