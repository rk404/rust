// Testy end-to-end pliku konsolowego qrgen-cli: kody wyjscia, stdout i pliki.

use std::path::PathBuf;
use std::process::{Command, Output};

fn run(args: &[&str], cwd: &PathBuf) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qrgen-cli"))
        .args(args)
        .current_dir(cwd)
        // Lokalne ustawienia nie moga wplywac na wynik.
        .env("XDG_CONFIG_HOME", cwd.join("cfg"))
        .env("APPDATA", cwd.join("cfg"))
        .output()
        .unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("qrgen_e2e_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn text_is_saved_and_path_printed() {
    let dir = temp_dir("text");
    let out = run(&["Zażółć gęślą", "-o", "kod.png"], &dir);
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("kod.png"));
    let decoded = qrgen_rust::decode::decode_file(&dir.join("kod.png")).unwrap();
    assert_eq!(decoded.raw, "Zażółć gęślą");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn second_run_does_not_overwrite_without_flag() {
    let dir = temp_dir("collision");
    assert_eq!(run(&["a", "-o", "kod.png"], &dir).status.code(), Some(0));
    assert_eq!(run(&["b", "-o", "kod.png"], &dir).status.code(), Some(0));
    assert!(dir.join("kod.png").exists() && dir.join("kod_2.png").exists());
    assert_eq!(
        run(&["c", "-o", "kod.png", "--overwrite"], &dir).status.code(),
        Some(0)
    );
    assert!(!dir.join("kod_3.png").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn vcard_creates_image_and_docx_pair() {
    let dir = temp_dir("vcard");
    let out = run(
        &["--format", "vcard", "--first-name", "Jan", "--last-name", "Kowalski"],
        &dir,
    );
    assert_eq!(out.status.code(), Some(0));
    assert!(dir.join("Jan_Kowalski.jpg").exists());
    assert!(dir.join("Jan_Kowalski.docx").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn bad_arguments_exit_with_code_2() {
    let dir = temp_dir("bad");
    for args in [
        vec!["--nieznana"],
        vec!["a", "b"],
        vec!["--scale", "abc", "x"],
        vec!["--format", "vcard", "dane"],
        vec![],
    ] {
        let out = run(&args, &dir);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn generation_failure_exits_with_code_1() {
    let dir = temp_dir("fail");
    // rozszerzenie nieobslugiwane => blad generowania, nie argumentow
    let out = run(&["x", "-o", "kod.gif"], &dir);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Błąd"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn local_settings_are_ignored_unless_requested() {
    let dir = temp_dir("settings");
    let cfg = dir.join("cfg").join("qrgen_rust");
    std::fs::create_dir_all(&cfg).unwrap();
    std::fs::write(cfg.join("settings.json"), r#"{ "image_ext": "bmp", "scale": 3 }"#).unwrap();

    assert_eq!(run(&["x"], &dir).status.code(), Some(0));
    assert!(dir.join("qrcode.jpg").exists(), "ustawienia lokalne wplynely na wynik");

    assert_eq!(run(&["x", "--use-settings"], &dir).status.code(), Some(0));
    assert!(dir.join("qrcode.bmp").exists());
    std::fs::remove_dir_all(&dir).ok();
}
