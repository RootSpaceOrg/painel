// Fones/caixas Bluetooth pareados: ver se estão conectados e conectar/desconectar sem abrir as Configurações.
// O Windows não tem API pública para isso; o caminho conhecido (o mesmo do ToothTray) é pedir ao filtro KS
// do driver de áudio Bluetooth a propriedade KSPROPSETID_BtAudio: ONESHOT_RECONNECT (0) / ONESHOT_DISCONNECT (1).
// Só vale para dispositivos de áudio já pareados.
use crate::media::mta;
use serde::Serialize;
use std::collections::BTreeMap;
use windows::core::{Interface, GUID, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_DeviceInterface_FriendlyName;
use windows::Win32::Media::Audio::{
    eAll, IConnector, IDeviceTopology, IMMDevice, IMMDeviceEnumerator, IPart, MMDeviceEnumerator, DEVICE_STATE, DEVICE_STATE_ACTIVE,
    DEVICE_STATE_UNPLUGGED,
};
use windows::Win32::Media::KernelStreaming::{IKsControl, KSIDENTIFIER, KSPROPERTY_TYPE_GET};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL, STGM_READ};

type WR<T> = windows::core::Result<T>;

const KSPROPSETID_BTAUDIO: GUID = GUID::from_u128(0x7fa06c40_b8f6_4c7e_8556_e8c33a12e54d);
const ONESHOT_RECONNECT: u32 = 0;
const ONESHOT_DISCONNECT: u32 = 1;

#[derive(Serialize)]
pub struct BtDevice {
    name: String,
    connected: bool,
}

struct Ep {
    name: String,
    active: bool,
    ks: IKsControl,
}

/// IKsControl do filtro KS por trás do endpoint, se o driver for Bluetooth.
unsafe fn bt_control(en: &IMMDeviceEnumerator, d: &IMMDevice) -> Option<IKsControl> {
    let topo: IDeviceTopology = d.Activate(CLSCTX_ALL, None).ok()?;
    let conn: IConnector = topo.GetConnector(0).ok()?;
    let to: IConnector = conn.GetConnectedTo().ok()?;
    let part: IPart = to.cast().ok()?;
    let ftopo: IDeviceTopology = part.GetTopologyObject().ok()?;
    let p = ftopo.GetDeviceId().ok()?;
    let id = p.to_string().unwrap_or_default();
    let dev = en.GetDevice(PCWSTR(p.0));
    CoTaskMemFree(Some(p.0 as _));
    // bthenum (A2DP), bthhfenum (chamada); outros drivers não são Bluetooth
    if !id.to_lowercase().contains("bth") {
        return None;
    }
    dev.ok()?.Activate(CLSCTX_ALL, None).ok()
}

fn endpoints() -> WR<Vec<Ep>> {
    unsafe {
        let en: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        // desconectado aparece como UNPLUGGED; conectado como ACTIVE
        let col = en.EnumAudioEndpoints(eAll, DEVICE_STATE(DEVICE_STATE_ACTIVE.0 | DEVICE_STATE_UNPLUGGED.0))?;
        let mut out = Vec::new();
        for i in 0..col.GetCount()? {
            let Ok(d) = col.Item(i) else { continue };
            let Some(ks) = bt_control(&en, &d) else { continue };
            // nome do aparelho (ex.: "Galaxy Buds2 Pro"), igual nos perfis de música e de chamada
            let name = d
                .OpenPropertyStore(STGM_READ)
                .and_then(|ps| ps.GetValue(&PKEY_DeviceInterface_FriendlyName))
                .map(|v| v.to_string())
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let active = d.GetState().map(|s| s == DEVICE_STATE_ACTIVE).unwrap_or(false);
            out.push(Ep { name, active, ks });
        }
        Ok(out)
    }
}

fn oneshot(ks: &IKsControl, id: u32) -> WR<()> {
    let mut prop = KSIDENTIFIER::default();
    prop.Anonymous.Anonymous.Set = KSPROPSETID_BTAUDIO;
    prop.Anonymous.Anonymous.Id = id;
    prop.Anonymous.Anonymous.Flags = KSPROPERTY_TYPE_GET;
    let mut ret = 0u32;
    unsafe { ks.KsProperty(&prop, std::mem::size_of::<KSIDENTIFIER>() as u32, std::ptr::null_mut(), 0, &mut ret) }
}

fn list() -> WR<Vec<BtDevice>> {
    let mut by_name: BTreeMap<String, bool> = BTreeMap::new();
    for e in endpoints()? {
        *by_name.entry(e.name).or_default() |= e.active;
    }
    Ok(by_name.into_iter().map(|(name, connected)| BtDevice { name, connected }).collect())
}

fn set(name: &str, connect: bool) -> WR<()> {
    let id = if connect { ONESHOT_RECONNECT } else { ONESHOT_DISCONNECT };
    let mut last = Ok(());
    let mut any = false;
    for e in endpoints()?.iter().filter(|e| e.name == name) {
        any = true;
        // basta um perfil aceitar; o Windows conecta os demais
        match oneshot(&e.ks, id) {
            Ok(()) => return Ok(()),
            Err(err) => last = Err(err),
        }
    }
    if !any {
        return Err(windows::core::Error::new(windows::Win32::Foundation::E_INVALIDARG, "Dispositivo não encontrado"));
    }
    last
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
pub async fn bt_list() -> Result<Vec<BtDevice>, String> {
    run(list).await
}

#[tauri::command]
pub async fn bt_set(name: String, connect: bool) -> Result<(), String> {
    run(move || set(&name, connect)).await
}

#[cfg(test)]
mod tests {
    // Só leitura, no hardware real: cargo test bt_list_real -- --ignored --nocapture
    #[test]
    #[ignore]
    fn bt_list_real() {
        crate::media::mta();
        for d in super::list().expect("listar") {
            println!("{} — {}", d.name, if d.connected { "conectado" } else { "desconectado" });
        }
    }
}
