// Wersja konsolowa: zwykla aplikacja konsolowa (bez podsystemu "windows"),
// wiec cmd/.bat/ERP czekaja na zakonczenie procesu i dostaja kod wyjscia
// oraz stdout/stderr. Zawsze dziala jak "qrgen_rust --silent".

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(qrgen_rust::silent::main_cli(&args));
}
