#![allow(non_snake_case)] // vtable COM com nomes do Windows
use crate::media::mta;
use serde::Serialize;
use std::collections::BTreeMap;
use windows::core::{Interface, GUID, HRESULT, PCWSTR, PWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCommunications, eConsole, eMultimedia, eRender, AudioSessionStateExpired, ERole, IAudioSessionControl2,
    IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL, STGM_READ};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};

type WR<T> = windows::core::Result<T>;

#[derive(Serialize)]
pub struct AudioDevice {
    id: String,
    name: String,
    default: bool,
}
#[derive(Serialize)]
pub struct AudioSession {
    pid: u32,
    name: String,
    exe: String, // caminho do executável ('' = sons do sistema / sem acesso), usado para o ícone
    volume: f32,
    muted: bool,
}
#[derive(Serialize)]
pub struct AudioState {
    devices: Vec<AudioDevice>,
    master: f32,
    muted: bool,
    sessions: Vec<AudioSession>,
}

// Interface não documentada do Windows para trocar o dispositivo padrão.
// Só a ordem da vtable importa: os 10 primeiros métodos nunca são chamados.
#[windows::core::interface("f8679f50-850a-41cf-9c72-430f290290c8")]
unsafe trait IPolicyConfig: windows::core::IUnknown {
    fn GetMixFormat(&self) -> HRESULT;
    fn GetDeviceFormat(&self) -> HRESULT;
    fn ResetDeviceFormat(&self) -> HRESULT;
    fn SetDeviceFormat(&self) -> HRESULT;
    fn GetProcessingPeriod(&self) -> HRESULT;
    fn SetProcessingPeriod(&self) -> HRESULT;
    fn GetShareMode(&self) -> HRESULT;
    fn SetShareMode(&self) -> HRESULT;
    fn GetPropertyValue(&self) -> HRESULT;
    fn SetPropertyValue(&self) -> HRESULT;
    fn SetDefaultEndpoint(&self, id: PCWSTR, role: ERole) -> HRESULT;
}
const CLSID_POLICY_CONFIG: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

fn enumerator() -> WR<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
}
fn default_dev() -> WR<IMMDevice> {
    unsafe { enumerator()?.GetDefaultAudioEndpoint(eRender, eConsole) }
}
fn endpoint() -> WR<IAudioEndpointVolume> {
    unsafe { default_dev()?.Activate(CLSCTX_ALL, None) }
}

fn dev_id(d: &IMMDevice) -> WR<String> {
    unsafe {
        let p: PWSTR = d.GetId()?;
        let s = p.to_string().unwrap_or_default();
        CoTaskMemFree(Some(p.0 as _));
        Ok(s)
    }
}

fn dev_name(d: &IMMDevice) -> String {
    unsafe {
        d.OpenPropertyStore(STGM_READ)
            .and_then(|ps| ps.GetValue(&PKEY_Device_FriendlyName))
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "Dispositivo".into())
    }
}

fn proc_path(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
        let _ = CloseHandle(h);
        ok.then(|| String::from_utf16_lossy(&buf[..len as usize]))
    }
}

fn proc_name(pid: u32, path: Option<&str>) -> String {
    match (pid, path) {
        (0, _) => "Sons do sistema".into(),
        (_, Some(p)) => std::path::Path::new(p).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| p.into()),
        _ => format!("PID {pid}"),
    }
}

/// Sessões ativas do dispositivo padrão: (pid, controle de volume).
fn sessions() -> WR<Vec<(u32, ISimpleAudioVolume)>> {
    unsafe {
        let mgr: IAudioSessionManager2 = default_dev()?.Activate(CLSCTX_ALL, None)?;
        let en = mgr.GetSessionEnumerator()?;
        let mut out = Vec::new();
        for i in 0..en.GetCount()? {
            let ctl = en.GetSession(i)?;
            if ctl.GetState().map(|s| s == AudioSessionStateExpired).unwrap_or(true) {
                continue;
            }
            let ctl2: IAudioSessionControl2 = ctl.cast()?;
            let pid = if ctl2.IsSystemSoundsSession() == HRESULT(0) { 0 } else { ctl2.GetProcessId().unwrap_or(0) };
            out.push((pid, ctl.cast()?));
        }
        Ok(out)
    }
}

fn state() -> WR<AudioState> {
    unsafe {
        let en = enumerator()?;
        let def = en.GetDefaultAudioEndpoint(eRender, eConsole).ok().and_then(|d| dev_id(&d).ok());
        let col = en.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
        let mut devices = Vec::new();
        for i in 0..col.GetCount()? {
            let d = col.Item(i)?;
            let id = dev_id(&d)?;
            devices.push(AudioDevice { name: dev_name(&d), default: def.as_deref() == Some(id.as_str()), id });
        }
        let (master, muted) = match endpoint() {
            Ok(v) => (v.GetMasterVolumeLevelScalar()?, v.GetMute()?.as_bool()),
            Err(_) => (0.0, false),
        };
        // agrupa por pid (um app pode ter várias sessões)
        let mut by_pid: BTreeMap<u32, AudioSession> = BTreeMap::new();
        for (pid, v) in sessions().unwrap_or_default() {
            by_pid.entry(pid).or_insert_with(|| {
                let path = proc_path(pid);
                AudioSession {
                pid,
                name: proc_name(pid, path.as_deref()),
                exe: path.unwrap_or_default(),
                volume: v.GetMasterVolume().unwrap_or(0.0),
                muted: v.GetMute().map(|b| b.as_bool()).unwrap_or(false),
                }
            });
        }
        let mut sessions: Vec<_> = by_pid.into_values().collect();
        sessions.sort_by(|a, b| (a.pid == 0).cmp(&(b.pid == 0)).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        Ok(AudioState { devices, master, muted, sessions })
    }
}

// ---------- ícone do app ----------

#[derive(Serialize)]
pub struct Icon {
    w: u32,
    h: u32,
    rgba: String, // base64 dos pixels RGBA (a UI desenha num canvas)
}

/// Ícone grande (32x32) do executável, em RGBA.
fn icon(path: &str) -> WR<Option<Icon>> {
    use base64::Engine;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};
    unsafe {
        let wide: Vec<u16> = path.encode_utf16().chain([0]).collect();
        let mut info = SHFILEINFOW::default();
        let r = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if r == 0 || info.hIcon.is_invalid() {
            return Ok(None);
        }
        let mut ii = ICONINFO::default();
        let got = GetIconInfo(info.hIcon, &mut ii);
        let out = (|| {
            got.ok()?;
            let mut bm = BITMAP::default();
            if GetObjectW(ii.hbmColor.into(), std::mem::size_of::<BITMAP>() as i32, Some(&mut bm as *mut _ as _)) == 0 {
                return None;
            }
            let (w, h) = (bm.bmWidth, bm.bmHeight);
            let mut bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h, // de cima para baixo
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut buf = vec![0u8; (w * h * 4) as usize];
            let dc = CreateCompatibleDC(None);
            let lines = GetDIBits(dc, ii.hbmColor, 0, h as u32, Some(buf.as_mut_ptr() as _), &mut bi, DIB_RGB_COLORS);
            let _ = DeleteDC(dc);
            if lines == 0 {
                return None;
            }
            // BGRA → RGBA; ícones antigos sem canal alfa vêm com alfa 0 em tudo
            let no_alpha = buf.chunks(4).all(|p| p[3] == 0);
            for p in buf.chunks_mut(4) {
                p.swap(0, 2);
                if no_alpha {
                    p[3] = 255;
                }
            }
            Some(Icon { w: w as u32, h: h as u32, rgba: base64::engine::general_purpose::STANDARD.encode(&buf) })
        })();
        if !ii.hbmColor.is_invalid() {
            let _ = DeleteObject(ii.hbmColor.into());
        }
        if !ii.hbmMask.is_invalid() {
            let _ = DeleteObject(ii.hbmMask.into());
        }
        let _ = DestroyIcon(info.hIcon);
        Ok(out)
    }
}

#[tauri::command]
pub async fn app_icon(path: String) -> Result<Option<Icon>, String> {
    run(move || icon(&path)).await
}

async fn run<T: Send + 'static>(f: impl FnOnce() -> WR<T> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || {
        mta();
        f()
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn audio_state() -> Result<AudioState, String> {
    run(state).await
}
#[tauri::command]
pub async fn audio_set_master(volume: f32) -> Result<(), String> {
    run(move || unsafe { endpoint()?.SetMasterVolumeLevelScalar(volume.clamp(0.0, 1.0), std::ptr::null()) }).await
}
#[tauri::command]
pub async fn audio_set_master_mute(muted: bool) -> Result<(), String> {
    run(move || unsafe { endpoint()?.SetMute(muted, std::ptr::null()) }).await
}
#[tauri::command]
pub async fn audio_set_session(pid: u32, volume: f32) -> Result<(), String> {
    run(move || unsafe {
        for (p, v) in sessions()? {
            if p == pid {
                v.SetMasterVolume(volume.clamp(0.0, 1.0), std::ptr::null())?;
            }
        }
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn audio_set_session_mute(pid: u32, muted: bool) -> Result<(), String> {
    run(move || unsafe {
        for (p, v) in sessions()? {
            if p == pid {
                v.SetMute(muted, std::ptr::null())?;
            }
        }
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn audio_set_default(id: String) -> Result<(), String> {
    run(move || unsafe {
        let pc: IPolicyConfig = CoCreateInstance(&CLSID_POLICY_CONFIG, None, CLSCTX_ALL)?;
        let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
        for role in [eConsole, eMultimedia, eCommunications] {
            pc.SetDefaultEndpoint(PCWSTR(wide.as_ptr()), role).ok()?;
        }
        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
    // hardware real, só leitura: cargo test -- --ignored
    #[test]
    #[ignore]
    fn reads_state() {
        crate::media::mta();
        let s = super::state().unwrap();
        assert!(!s.devices.is_empty());
        println!("{}", serde_json::to_string(&s).unwrap());
    }
}

#[cfg(test)]
mod icon_tests {
    // Só leitura, no hardware real: cargo test icons_real -- --ignored --nocapture
    #[test]
    #[ignore]
    fn icons_real() {
        crate::media::mta();
        let st = super::state().expect("estado");
        for s in &st.sessions {
            let ic = if s.exe.is_empty() { None } else { super::icon(&s.exe).expect("ícone") };
            match ic {
                Some(i) => println!("{:<20} ícone {}x{} ({} bytes base64)", s.name, i.w, i.h, i.rgba.len()),
                None => println!("{:<20} sem ícone", s.name),
            }
        }
    }
}
