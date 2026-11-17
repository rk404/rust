# qrgen (Rust + egui/eframe)

Generator kodów QR napisany w Rust. Aplikacja obsługuje kilka formatów
kodowanych danych (tekst, wizytówka vCard, link, e-mail), potrafi odczytywać
istniejące kody QR z obrazów i jest przystosowana do wywołań z innych
systemów (np. ERP) przez argumenty CLI lub plik JSON.

## Co potrafi aplikacja

- pokazywać **podgląd na żywo** (w pamięci, bez zapisu) i zapisywać plik dopiero przyciskiem „Zapisz”,
- generować kod QR dla formatów: **Tekst**, **vCard** (wizytówka), **Link**, **E-mail**,
- dla vCard generować dodatkowo **dokument Word (.docx)** z danymi do wizytówki
  (blok polski oraz — gdy podano stanowisko po angielsku — blok angielski),
- nazywać pliki wizytówki imieniem i nazwiskiem (np. `Jan_Kowalski.jpg` + `Jan_Kowalski.docx`,
  polskie znaki transliterowane do ASCII),
- **odczytywać kody QR** z obrazów (wybór pliku, przeciągnięcie na okno, wklejenie
  ze schowka); rozpoznane wizytówki vCard są pokazywane jako pola i mogą trafić
  do formularza,
- zapisywać wynik do JPG (domyślnie, jakość 95), PNG lub BMP,
- chronić przed nadpisaniem: przy kolizji nazw dokleja `_2`, `_3`, ...
  (wyłączane opcją `--overwrite` / w Ustawieniach),
- dostosowywać rozdzielczość (docelowy bok obrazu w px lub skalę), ramkę,
  korekcję błędów i kolory (okno „Ustawienia”),
- kopiować dane lub wygenerowany obraz do schowka, otwierać i drukować pliki,
- zapamiętywać ustawienia i zawartość formularzy między uruchomieniami (Windows: `%APPDATA%\qrgen_rust`, macOS: `~/Library/Application Support/qrgen_rust`, Linux: `~/.config/qrgen_rust`;
  dawne ustawienia z `~/.config` na macOS są wczytywane automatycznie)
  (`settings.json`; stary `settings.txt` jest migrowany automatycznie).

## Tryby działania

1. **Brak argumentów** → GUI. U góry przełącznik trybu `Generuj | Odczyt` i przycisk
   „Ustawienia”. W trybie Generuj po lewej formularz z przełącznikiem formatu
   (`Tekst | vCard | Link | E-mail`), po prawej podgląd na żywo, wygląd kodu i
   przycisk „Zapisz” (Ctrl+S). Na dole pasek stanu z odnośnikami do plików.
2. **Argumenty bez `--silent`** → GUI otwiera się od razu na wskazanym
   formularzu z wypełnionymi polami i generuje pliki.
3. **Argumenty z `--silent`** → tryb konsolowy bez okna (dla ERP/skryptów);
   ścieżki zapisanych plików trafiają na stdout, błędy na stderr
   (kod wyjścia: `2` błędne argumenty/JSON, `1` błąd generowania).
   Lokalny `settings.json` jest tu **pomijany** (wynik nie zależy od komputera);
   `--use-settings` go włącza.

### qrgen-cli (dla ERP i plików .bat)

Na Windows `qrgen_rust.exe` to aplikacja okienkowa: `cmd` i pliki `.bat` nie czekają
na jej zakończenie, więc nie widać kodu wyjścia ani stdout. Do integracji służy
osobny **`qrgen-cli.exe`** (zwykła aplikacja konsolowa, zawsze tryb `--silent`):

```bat
qrgen-cli.exe --format vcard --input pracownik.json -o "C:\Wizytowki"
if errorlevel 1 echo Blad generowania
```

Zapis plików jest atomowy (`create_new`), więc równoległe wywołania nigdy nie
dostaną tej samej nazwy.

## Integracja z ERP

Zalecana ścieżka to plik JSON (omija problemy z cudzysłowami i polskimi
znakami w cmd/batch):

```bash
qrgen_rust --silent --format vcard --input pracownik.json -o "C:\Wizytowki"
```

Przykładowy `pracownik.json` (wszystkie pola opcjonalne, klucze angielskie,
kodowanie UTF-8, BOM tolerowany):

```json
{
  "format": "vcard",
  "output": "C:/Wizytowki",
  "overwrite": false,
  "docx": true,
  "appearance": { "scale": 10, "size": 0, "border": 4, "ecc": "M", "fill": "000000", "back": "FFFFFF" },
  "vcard": {
    "first_name": "Jan", "last_name": "Kowalski",
    "phone_work": "58 307 00 00", "phone_mobile": "600 000 000",
    "email": "jan.kowalski@firma.pl", "website": "https://firma.pl",
    "company": "Firma Sp. z o.o.",
    "title_pl": "Kierownik Logistyki Dostaw", "title_en": "Supply Chain Manager",
    "fax": "", "street": "ul. Portowa 1", "city": "Gdańsk",
    "postal_code": "80-001", "country": "Polska"
  },
  "text":  { "data": "..." },
  "link":  { "url": "..." },
  "email": { "to": "", "subject": "", "body": "" }
}
```

Precedencja wartości: domyślne ← `settings.json` ← `--input` JSON ← jawne flagi CLI.
Te same pola można podać bezpośrednio flagami (patrz niżej).

## Opcje wiersza poleceń

```text
qrgen [DANE] [opcje]              DANE (pozycyjne) = tryb tekstowy, jak dotychczas

Ogólne:
  --format text|link|email|vcard  wybór formatu / formularza startowego
  --input PLIK.json               wczytaj pola z pliku JSON (UTF-8)
  -o, --output ŚCIEŻKA            plik (.jpg/.png/.bmp) LUB katalog docelowy
  -s, --silent                    tryb konsolowy bez okna
  --overwrite                     nadpisuj istniejące pliki
  --no-docx                       (vcard) nie generuj dokumentu Word
  -h, --help                      pomoc

Wygląd:  --scale N  --size N (docelowy bok obrazu w px, nadpisuje --scale)
         --border N  --error L|M|Q|H  --fill RRGGBB  --back RRGGBB

Pola vCard:  --first-name --last-name --phone --mobile --email --website
             --company --title --title-en --fax --street --city
             --postal-code --country
Pola e-mail: --to --subject --body        Pole link: --url
```

Nieznane flagi, błędne wartości i nadmiarowe dane pozycyjne kończą się błędem
(kod 2) — nic nie jest ignorowane po cichu. Tekst zaczynający się od `-` podaj po `--`.
Maksymalny bok obrazu to 8000 px.

## Przykłady

```bash
qrgen_rust "https://example.com"
qrgen_rust "Tekst do zakodowania" --silent -o kod.png
qrgen_rust --format vcard --first-name Jan --last-name Kowalski \
  --phone "58 307 00 00" --title "Kierownik Logistyki Dostaw" --silent
qrgen_rust --silent --format link --url firma.pl -o /tmp/kody/
```

## Struktura projektu

```text
src/
├── main.rs        – punkt wejścia (GUI): CLI → --help | --silent | GUI
├── bin/qrgen-cli.rs – wersja konsolowa (zawsze --silent) dla ERP
├── silent.rs      – tryb konsolowy współdzielony przez oba pliki wykonywalne
├── lib.rs         – moduły jako biblioteka (testy integracyjne w tests/)
├── cli.rs         – parser argumentów + scalanie z JSON/ustawieniami
├── input_json.rs  – schemat pliku --input (serde)
├── formats/       – Tekst / Link / E-mail / vCard (buildery + parser vCard)
├── render.rs      – renderowanie obrazu QR
├── output.rs      – nazwy plików (transliteracja, sufiksy), zapis JPG/PNG/BMP
├── docx.rs        – dokument Word z wizytówką (minimalny OOXML)
├── decode.rs      – odczyt kodów QR z obrazów (rqrr)
├── settings.rs    – settings.json + migracja starego settings.txt
├── platform.rs    – integracje systemowe (konsola, motyw, otwieranie, druk)
└── gui/           – interfejs (układ okna, widżety, motyw, formularze,
                     podgląd z wyglądem kodu, widok odczytu, okno Ustawień)
```

## Budowanie i testy

Wymagany jest zainstalowany Rust (`rustup`). W katalogu projektu:

```bash
cargo build --release   # target/release/qrgen_rust(.exe) oraz qrgen-cli(.exe)
cargo test              # jednostkowe + round-trip + testy CLI (kody wyjścia, pliki)
```

## Zależności

- `qrcode` – kodowanie QR,
- `image` – zapis/odczyt JPG, PNG, BMP,
- `rqrr` – dekodowanie kodów QR z obrazów (czysty Rust),
- `serde` / `serde_json` – plik `--input` z ERP oraz `settings.json`,
- `zip` – paczka .docx,
- `eframe`/`egui` – interfejs graficzny,
- `rfd` – systemowe okna dialogowe wyboru plików,
- `arboard` – schowek (kopiowanie obrazu, wklejanie do odczytu),
- `windows` – tylko na Windows: konsola i motyw systemowy.

Wszystkie zależności są pobierane automatycznie przez `cargo build`.

## Uwagi

- Interfejs używa czcionki systemowej (Windows: Segoe UI, macOS: San Francisco lub Arial,
  Linux: Noto Sans, Cantarell, DejaVu Sans lub Liberation Sans), wczytywanej z dysku.
  Gdy żadnej nie znajdzie, zostaje wbudowana czcionka egui. Własną czcionkę (TTF/OTF)
  można wskazać zmienną środowiskową `QRGEN_FONT`.

- Dane w kodzie QR są zapisywane jako UTF-8 (tryb bajtowy, bez nagłówka ECI) —
  poprawność polskich znaków potwierdza test round-trip.
- Odczyt kodów QR działa najlepiej na zrzutach ekranu i skanach; zdjęcia
  o niskim kontraście lub nietypowych kolorach mogą nie zostać rozpoznane.
- W bloku angielskim wizytówki numery telefonów dostają prefiks `+48 `,
  chyba że już zaczynają się od `+`.
