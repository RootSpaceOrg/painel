// Iniciar com o Windows: valor "Painel" em HKCU\...\Run apontando para o exe atual.
use windows::core::{w, PCWSTR};
use windows::Win32::System::Registry::*;

const RUN: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
const NAME: PCWSTR = w!("Painel");

pub fn enabled() -> bool {
    unsafe { RegGetValueW(HKEY_CURRENT_USER, RUN, NAME, RRF_RT_REG_SZ, None, None, None).is_ok() }
}

pub fn set(on: bool) -> Result<(), String> {
    let r = unsafe {
        if on {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let v: Vec<u16> = format!("\"{}\"", exe.display()).encode_utf16().chain([0]).collect();
            RegSetKeyValueW(HKEY_CURRENT_USER, RUN, NAME, REG_SZ.0, Some(v.as_ptr().cast()), (v.len() * 2) as u32)
        } else if enabled() {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN, NAME)
        } else {
            return Ok(());
        }
    };
    r.ok().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn autostart_get() -> bool {
    enabled()
}

#[tauri::command]
pub fn autostart_set(on: bool) -> Result<(), String> {
    set(on)
}

/// Primeira execução do app instalado: liga sozinho (dá pra desligar nas configurações).
/// Também regrava o caminho se o exe mudou de lugar (reinstalação). Em dev não mexe no registro.
pub fn init(c: &rusqlite::Connection) {
    if cfg!(debug_assertions) {
        return;
    }
    let first = crate::db::kv_read(c, "autostart_init").is_none();
    if first || enabled() {
        if let Err(e) = set(true) {
            eprintln!("autostart: {e}");
        }
    }
    if first {
        let _ = crate::db::kv_write(c, "autostart_init", "1");
    }
}
