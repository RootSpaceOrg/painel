use crate::db::{self, Db};
use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, LogicalSize, Manager, Monitor, PhysicalPosition, WebviewWindow};

#[derive(Serialize, Deserialize, Clone)]
pub struct WindowCfg {
    monitor: Option<String>,
    fullscreen: bool,
    // fora da barra de tarefas (acesso pela bandeja); configs antigas sem o campo ficam com true
    #[serde(default = "yes")]
    hide_taskbar: bool,
}
fn yes() -> bool {
    true
}

#[derive(Serialize)]
pub struct MonitorInfo {
    name: String,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    primary: bool,
}

fn main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("main").ok_or_else(|| "janela não encontrada".to_string())
}

pub fn load_cfg(app: &AppHandle) -> WindowCfg {
    app.state::<Db>()
        .0
        .lock()
        .ok()
        .and_then(|c| db::kv_read(&c, "window"))
        .and_then(|s| serde_json::from_str(&s).ok())
        // primeira abertura: janela no primário, para escolher o monitor nas configurações
        .unwrap_or(WindowCfg { monitor: None, fullscreen: false, hide_taskbar: true })
}

fn pick_monitor(w: &WebviewWindow, name: &Option<String>) -> Option<Monitor> {
    let mons = w.available_monitors().unwrap_or_default();
    name.as_ref()
        .and_then(|n| mons.iter().find(|m| m.name() == Some(n)).cloned())
        .or_else(|| w.primary_monitor().ok().flatten())
        .or_else(|| mons.into_iter().next())
}

pub fn apply(app: &AppHandle, cfg: &WindowCfg) -> Result<(), String> {
    let w = main_window(app)?;
    let e = |x: tauri::Error| x.to_string();
    let Some(m) = pick_monitor(&w, &cfg.monitor) else { return Ok(()) };
    let (mp, ms, scale) = (*m.position(), *m.size(), m.scale_factor());
    w.set_skip_taskbar(cfg.hide_taskbar).map_err(e)?;
    w.set_fullscreen(false).map_err(e)?;
    if cfg.fullscreen {
        // fullscreen do Windows usa o monitor onde a janela está: move antes
        w.set_decorations(false).map_err(e)?;
        w.set_position(PhysicalPosition::new(mp.x + 40, mp.y + 40)).map_err(e)?;
        w.set_fullscreen(true).map_err(e)?;
    } else {
        w.set_decorations(true).map_err(e)?;
        w.set_size(LogicalSize::new(1366.0, 768.0)).map_err(e)?;
        let (pw, ph) = ((1366.0 * scale) as i32, (768.0 * scale) as i32);
        let x = mp.x + ((ms.width as i32 - pw) / 2).max(0);
        let y = mp.y + ((ms.height as i32 - ph) / 2).max(0);
        w.set_position(PhysicalPosition::new(x, y)).map_err(e)?;
    }
    Ok(())
}

fn save_apply(app: &AppHandle, cfg: WindowCfg) -> Result<(), String> {
    {
        let db = app.state::<Db>();
        let c = db.0.lock().map_err(|e| e.to_string())?;
        db::kv_write(&c, "window", &serde_json::to_string(&cfg).map_err(|e| e.to_string())?)?;
    }
    apply(app, &cfg)
}

#[tauri::command]
pub fn monitors_list(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let w = main_window(&app)?;
    let primary = w.primary_monitor().ok().flatten().and_then(|m| m.name().cloned());
    Ok(w.available_monitors()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|m| {
            let name = m.name().cloned().unwrap_or_default();
            MonitorInfo {
                primary: primary.as_ref() == Some(&name),
                width: m.size().width,
                height: m.size().height,
                x: m.position().x,
                y: m.position().y,
                name,
            }
        })
        .collect())
}

#[tauri::command]
pub fn window_get(app: AppHandle) -> WindowCfg {
    load_cfg(&app)
}

#[tauri::command]
pub fn window_apply(app: AppHandle, monitor: Option<String>, fullscreen: bool, hide_taskbar: bool) -> Result<(), String> {
    save_apply(&app, WindowCfg { monitor, fullscreen, hide_taskbar })
}

fn toggle_visible(app: &AppHandle) {
    if let Ok(w) = main_window(app) {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
            let _ = w.set_focus();
        }
    }
}

pub fn tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "toggle", "Mostrar/Ocultar", true, None::<&str>)?,
            &MenuItem::with_id(app, "fullscreen", "Alternar tela cheia", true, None::<&str>)?,
            &MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?,
        ],
    )?;
    let mut b = TrayIconBuilder::new().tooltip("Painel").menu(&menu).show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        b = b.icon(icon.clone());
    }
    b.on_menu_event(|app, ev| match ev.id.as_ref() {
        "toggle" => toggle_visible(app),
        "fullscreen" => {
            let mut cfg = load_cfg(app);
            cfg.fullscreen = !cfg.fullscreen;
            if let Err(e) = save_apply(app, cfg) {
                eprintln!("fullscreen: {e}");
            }
        }
        "quit" => app.exit(0),
        _ => {}
    })
    .on_tray_icon_event(|tray, ev| {
        if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
            toggle_visible(tray.app_handle());
        }
    })
    .build(app)?;
    Ok(())
}

// ---------- wallpaper ----------
fn wallpaper_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("wallpaper"))
}

#[tauri::command]
pub fn wallpaper_set(app: AppHandle, request: tauri::ipc::Request<'_>) -> Result<(), String> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else { return Err("esperava bytes da imagem".into()) };
    if bytes.is_empty() {
        return Err("imagem vazia".into());
    }
    std::fs::write(wallpaper_path(&app)?, bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn wallpaper_get(app: AppHandle) -> tauri::ipc::Response {
    tauri::ipc::Response::new(wallpaper_path(&app).ok().and_then(|p| std::fs::read(p).ok()).unwrap_or_default())
}

#[tauri::command]
pub fn wallpaper_clear(app: AppHandle) -> Result<(), String> {
    let p = wallpaper_path(&app)?;
    if p.exists() {
        std::fs::remove_file(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Abre um link no navegador padrão. Só https (fronteira: o texto vem de feeds de terceiros).
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") || url.chars().any(|c| c.is_whitespace() || c == '"') {
        return Err("Link inválido.".into());
    }
    std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", &url]).spawn().map_err(|e| e.to_string())?;
    Ok(())
}
