use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Na Windows aplikacja jest budowana z podsystemem "windows" (bez konsoli).
/// Gdy jest uruchamiana z cmd.exe z flaga --silent/--help, doczepiamy sie do
/// konsoli procesu wywolujacego, zeby println!/eprintln! byly widoczne.
/// Na Linuksie/macOS konsola jest dostepna od razu - funkcja nic nie robi.
#[cfg(windows)]
pub fn attach_console() {
    use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
pub fn attach_console() {}

/// Odczyt motywu jasny/ciemny bezposrednio z rejestru przez API Windows
/// (RegGetValueW), bez odpalania procesu reg.exe.
#[cfg(windows)]
pub fn windows_uses_light_theme() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let value: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();

    let mut data: u32 = 1;
    let mut size: u32 = std::mem::size_of::<u32>() as u32;

    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut _),
            Some(&mut size),
        )
    };

    if status.is_ok() {
        data != 0
    } else {
        true
    }
}

#[cfg(not(windows))]
pub fn windows_uses_light_theme() -> bool {
    true
}

/// Folder Dokumenty uzytkownika - domyslna lokalizacja zapisu.
pub fn documents_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .map(|base| base.join("Documents"))
    }

    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|base| base.join("Documents"))
    }
}

/// Katalog na plik ustawien aplikacji.
pub fn config_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("LOCALAPPDATA"))
            .map(PathBuf::from)
            .map(|base| base.join("qrgen_rust"))
    }

    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .map(|base| base.join("qrgen_rust"))
    }
}

/// Otwiera plik lub katalog w domyslnej aplikacji systemowej.
pub fn open_path(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let mut command = {
        let mut c = Command::new("explorer");
        c.arg(path);
        c
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut c = Command::new("open");
        c.arg(path);
        c
    };

    #[cfg(all(not(windows), not(target_os = "macos")))]
    let mut command = {
        let mut c = Command::new("xdg-open");
        c.arg(path);
        c
    };

    command
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Nie udało się otworzyć '{}': {e}", path.display()))
}

/// Wysyla obraz do drukowania: na Windows przez okno wyboru drukarki
/// (PowerShell + System.Drawing.Printing), na macOS otwiera Podglad,
/// na innych systemach otwiera domyslna przegladarke obrazow.
pub fn print_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("Plik wyjściowy nie istnieje. Najpierw wygeneruj kod QR.".to_string());
    }

    #[cfg(windows)]
    {
        let escaped_path = path.display().to_string().replace('"', "''");
        let script = format!(
            r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
$path = [System.IO.Path]::GetFullPath('{escaped_path}')
if (-not (Test-Path -LiteralPath $path)) {{ throw 'Plik nie istnieje.' }}
$img = [System.Drawing.Image]::FromFile($path)
try {{
    $doc = New-Object System.Drawing.Printing.PrintDocument
    $doc.DocumentName = [System.IO.Path]::GetFileName($path)
    $doc.Add_PrintPage({{
        param($sender, $e)
        $e.Graphics.DrawImage($img, 0, 0, $e.PageBounds.Width, $e.PageBounds.Height)
    }})

    $printDialog = New-Object System.Windows.Forms.PrintDialog
    $printDialog.UseEXDialog = $true
    $printDialog.Document = $doc
    if ($printDialog.ShowDialog() -ne [System.Windows.Forms.DialogResult]::OK) {{
        exit 0
    }}

    $doc.Print()
}} finally {{
    if ($img) {{ $img.Dispose() }}
}}
"#
        );

        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-NoLogo",
                "-Sta",
                "-WindowStyle",
                "Hidden",
                "-Command",
                &script,
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Nie udało się uruchomić drukowania: {e}"))
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg("Preview")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Nie udało się uruchomić drukowania: {e}"))
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        open_path(path)
    }
}
