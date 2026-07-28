# qrgen (Rust + egui/eframe)

Generator kodów QR napisany w Rust. Aplikacja łączy prosty interfejs
tekstowy z natywnym oknem GUI, pozwalając wygenerować kod QR do pliku
PNG lub BMP oraz zobaczyć jego podgląd przed zapisaniem.

## Co potrafi aplikacja

- generować kod QR z tekstu lub adresu URL,
- zapisywać wynik do pliku PNG/BMP,
- dostosowywać skalę modułu, grubość ramki i poziom korekcji błędów,
- zmieniać kolor kodu i tła,
- podglądać wynik w oknie aplikacji,
- kopiować dane lub wygenerowany obraz do schowka,
- zapamiętywać ustawienia użytkownika między uruchomieniami.

## Tryby działania

1. **Brak argumentów** → otwiera się puste okno GUI.
2. **Argumenty bez `--silent`** → otwiera się GUI z uzupełnionymi polami
   (dane, plik wyjściowy, skala, ramka, korekcja błędów, kolory).
3. **Argumenty z `--silent`** → aplikacja działa bez okna i zapisuje plik
   w trybie konsolowym.

## Struktura projektu

```text
qrgen_rust/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs      – parsowanie argumentów, tryb --silent, punkt wejścia
    ├── qr_logic.rs  – kodowanie danych i renderowanie obrazu QR
    └── gui.rs       – interfejs graficzny (eframe/egui)
```

Logika QR jest wspólna dla GUI i trybu konsolowego, więc ten sam kod jest
używany zarówno przy przycisku „Generuj”, jak i przy uruchomieniu z CLI.

## Budowanie

Wymagany jest zainstalowany Rust z aktualnym `rustup`.
W katalogu projektu uruchom:

```bash
cargo build --release
```

Gotowy plik binarny pojawi się w:

- Linux/macOS: `target/release/qrgen_rust`
- Windows: `target/release/qrgen_rust.exe`

## Uruchamianie

### GUI

```bash
cargo run
```

### Z danymi z wiersza poleceń

```bash
cargo run -- "https://example.com"
```

### Tryb konsolowy

```bash
cargo run -- "https://example.com" --silent -o qr.png
```

## Opcje wiersza poleceń

| Opcja | Opis | Domyślnie |
|---|---|---|
| `-o`, `--output PLIK` | Plik wynikowy (rozszerzenie decyduje o formacie: `.png`, `.bmp`) | `qrcode.png` |
| `--scale N` | Rozmiar modułu w pikselach | `10` |
| `--border N` | Grubość ramki w modułach | `4` |
| `--error L|M|Q|H` | Poziom korekcji błędów | `M` |
| `--fill RRGGBB` | Kolor kodu w formacie hex | `000000` |
| `--back RRGGBB` | Kolor tła w formacie hex | `FFFFFF` |
| `--silent` | Bez GUI, tryb konsolowy | – |
| `-h`, `--help` | Wyświetla pomoc | – |

## Przykłady

```bash
qrgen_rust "https://example.com"
qrgen_rust "Tekst do zakodowania" --silent -o kod.bmp
qrgen_rust "https://example.com" --scale 12 --error H --fill 0033AA --silent
```

## Zależności

- `qrcode` – kodowanie QR,
- `image` – zapis plików PNG/BMP,
- `eframe`/`egui` – interfejs graficzny,
- `rfd` – okno dialogowe „Zapisz jako”,
- `arboard` – kopiowanie danych i obrazu do schowka,
- `windows` – tylko na Windows, do obsługi konsoli i motywu systemowego.

Wszystkie zależności są pobierane automatycznie przez `cargo build`.
