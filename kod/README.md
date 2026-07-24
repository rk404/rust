# qrgen (Rust + egui/eframe)

Generator kodów QR — ta sama funkcjonalność co wersja C/Win32, przepisana
w Rust. GUI zbudowane na `egui`/`eframe` (immediate-mode, natywne okno,
działa tak samo na Windows/macOS/Linux).

## Zachowanie (identyczne jak w wersji C)

1. **Brak argumentów** → otwiera się puste okno GUI.
2. **Argumenty z cmd, bez `--silent`** → otwiera się GUI z uzupełnionymi
   polami (dane, plik wyjściowy, skala, ramka, korekcja błędów, kolory) —
   podgląd generuje się automatycznie.
3. **Argumenty + `--silent`** → działa bez okna: generuje plik i wypisuje
   wynik w konsoli.

## Struktura projektu

```
qrgen_rust/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs      – parsowanie argumentów, tryb --silent, punkt wejścia
    ├── qr_logic.rs  – rdzeń: kodowanie danych + renderowanie obrazu (bez GUI)
    └── gui.rs       – interfejs graficzny (eframe/egui)
```

`qr_logic.rs` jest całkowicie niezależny od GUI — to ten sam kod jest
używany przez tryb `--silent` i przez okno GUI (przycisk „Generuj”).

## Budowanie

Wymagany jest zainstalowany Rust (https://rustup.rs — jedno polecenie,
instalacja zajmuje dosłownie chwilę). Następnie, w folderze projektu:

```bash
cargo build --release
```

Gotowy plik pojawi się w `target/release/qrgen_rust` (Linux/macOS) albo
`target\release\qrgen_rust.exe` (Windows).

### Budowanie na Windows

Po prostu `cargo build --release` na maszynie z Windows — nie trzeba nic
dodatkowo konfigurować, `eframe`/`rfd` same dociągną odpowiednie zależności
dla tej platformy.

### Uwaga dot. tego środowiska (cross-kompilacja)

Ten projekt **nie mógł zostać cross-skompilowany do gotowego `.exe` w tym
sandboxie** (Linux, offline poza wybraną listą domen). W wersji C
wystarczył pakiet `mingw-w64` z Ubuntu, ale w Rust cross-kompilacja na
Windows wymaga pobrania dodatkowego komponentu Rusta (`rust-std` dla celu
`x86_64-pc-windows-gnu` lub `-msvc`) z serwerów `static.rust-lang.org`,
do których to środowisko nie ma dostępu sieciowego. Na zwykłym komputerze
z `rustup` to pojedyncze polecenie:

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## Co zostało zweryfikowane w tym środowisku

- **Rdzeń logiki (`qr_logic.rs`) w pełni skompilowany, przetestowany i
  zweryfikowany** — testy jednostkowe (`cargo test`) przechodzą, a
  wygenerowane pliki PNG/BMP (w tym z polskimi znakami, niestandardowymi
  kolorami i poziomami korekcji błędów) zostały zdekodowane z powrotem
  (biblioteką `pyzbar`), potwierdzając poprawność kodowania.
- **Parsowanie argumentów i tryb `--silent`** — przetestowane end-to-end
  na Linuksie, działa poprawnie, łącznie z danymi zawierającymi polskie
  znaki (Rust obsługuje Unicode/UTF-8 natywnie w `std::env::args()`, bez
  problemów, które pojawiły się w wersji C+Wine).
- **Warstwa GUI (`gui.rs`, `eframe`)** — napisana starannie w oparciu o
  standardowe, sprawdzone wzorce API `egui`/`eframe`, ale **nie mogła
  zostać skompilowana ani przetestowana wizualnie w tym środowisku** —
  dostępny tu Rust (z systemowego apt Ubuntu) jest zbyt stary (1.75 z
  końca 2023), a aktualne wersje `eframe`/`egui`/`rfd` na crates.io
  wymagają dziś dużo nowszego kompilatora (edycja Rust 2024). To ograniczenie
  wyłącznie tego sandboxa — na zwykłym komputerze z aktualnym `rustup`
  (co jest standardem) nie ma to żadnego znaczenia.

Krótko mówiąc: **to, co najważniejsze — poprawność generowania kodu QR —
jest pewne**. Sam wygląd/działanie okna warto sprawdzić po zbudowaniu na
docelowej maszynie; jeśli coś będzie wymagało poprawki, daj znać.

## Opcje wiersza poleceń

| Opcja | Opis | Domyślnie |
|---|---|---|
| `-o`, `--output PLIK` | Plik wynikowy (rozszerzenie decyduje o formacie: `.png`, `.bmp`) | `qrcode.png` |
| `--scale N` | Rozmiar modułu w pikselach | `10` |
| `--border N` | Grubość białej ramki w modułach | `4` |
| `--error L\|M\|Q\|H` | Poziom korekcji błędów | `M` |
| `--fill RRGGBB` | Kolor kodu (hex) | `000000` |
| `--back RRGGBB` | Kolor tła (hex) | `FFFFFF` |
| `--silent` | Bez GUI, tryb konsolowy | – |
| `-h`, `--help` | Pomoc | – |

## Przykłady

```bash
qrgen_rust "https://example.com"                        # GUI, puste pole
qrgen_rust "https://example.com" --silent               # bez GUI
qrgen_rust "Tekst" -o kod.bmp --scale 12 --error H       # GUI, wypełnione
qrgen_rust "https://example.com" --fill 0033AA --silent # bez GUI, kolor
```

## Zależności (crates.io)

- [`qrcode`](https://crates.io/crates/qrcode) — kodowanie QR (MIT/Apache-2.0)
- [`image`](https://crates.io/crates/image) — zapis PNG/BMP (MIT)
- [`eframe`/`egui`](https://crates.io/crates/eframe) — GUI (MIT/Apache-2.0)
- [`rfd`](https://crates.io/crates/rfd) — natywne okno „Zapisz jako” (MIT)
- [`windows`](https://crates.io/crates/windows) — tylko na Windows, do
  doczepienia konsoli w trybie `--silent` (MIT/Apache-2.0)

Wszystkie są pobierane automatycznie przez `cargo build` — nie trzeba
niczego ręcznie dołączać.
