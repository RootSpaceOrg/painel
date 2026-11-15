use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Nvml};
use serde::Serialize;
use std::time::{Duration, Instant};
use sysinfo::{Disks, Networks, System};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
struct Gpu {
    name: String,
    util: u32,
    temp: u32,
    vram_used: u64,
    vram_total: u64,
    power_w: Option<f64>,
}

#[derive(Serialize, Clone)]
struct Disk {
    name: String,
    mount: String,
    used: u64,
    total: u64,
}

#[derive(Serialize, Clone)]
struct Sample {
    cpu: f32,
    cores: Vec<f32>,
    ram_used: u64,
    ram_total: u64,
    gpu: Option<Gpu>,
    net_rx: u64,
    net_tx: u64,
    disks: Vec<Disk>,
}

fn gpu(nvml: &Nvml) -> Option<Gpu> {
    let d = nvml.device_by_index(0).ok()?;
    let mem = d.memory_info().ok()?;
    Some(Gpu {
        name: d.name().unwrap_or_default(),
        util: d.utilization_rates().map(|u| u.gpu).unwrap_or(0),
        temp: d.temperature(TemperatureSensor::Gpu).unwrap_or(0),
        vram_used: mem.used,
        vram_total: mem.total,
        power_w: d.power_usage().ok().map(|mw| mw as f64 / 1000.0),
    })
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let nvml = Nvml::init().map_err(|e| eprintln!("nvml indisponível: {e}")).ok();
        let Ok(mut sys) = System::new() else {
            eprintln!("sysinfo indisponível");
            return;
        };
        let mut nets = Networks::new_with_refreshed_list().ok();
        let mut disks = Disks::new_with_refreshed_list().ok();
        let mut last = Instant::now();
        let mut tick: u32 = 0;
        loop {
            std::thread::sleep(Duration::from_secs(2));
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            let secs = last.elapsed().as_secs_f64().max(0.001);
            last = Instant::now();
            let (mut rx, mut tx) = (0u64, 0u64);
            if let Some(n) = nets.as_mut() {
                n.refresh(true);
                for (_, d) in n.iter() {
                    rx += d.received();
                    tx += d.transmitted();
                }
            }
            // espaço em disco muda devagar: relê a cada ~30s
            if tick % 15 == 0 {
                if let Some(d) = disks.as_mut() {
                    d.refresh(true);
                }
            }
            tick = tick.wrapping_add(1);
            let disk_list = disks
                .as_ref()
                .map(|d| {
                    d.iter()
                        .filter(|k| k.total_space() > 0)
                        .map(|k| Disk {
                            name: k.name().to_string_lossy().into_owned(),
                            mount: k.mount_point().to_string_lossy().into_owned(),
                            used: k.total_space() - k.available_space(),
                            total: k.total_space(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            let s = Sample {
                cpu: sys.global_cpu_usage(),
                cores: sys.cpus().iter().map(|c| c.usage()).collect(),
                ram_used: sys.used_memory(),
                ram_total: sys.total_memory(),
                gpu: nvml.as_ref().and_then(gpu),
                net_rx: (rx as f64 / secs) as u64,
                net_tx: (tx as f64 / secs) as u64,
                disks: disk_list,
            };
            let _ = app.emit("sys", s);
        }
    });
}
