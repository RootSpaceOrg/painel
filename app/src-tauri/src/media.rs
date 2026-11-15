use base64::Engine;
use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session, GlobalSystemMediaTransportControlsSessionManager as Mgr,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use windows::Storage::Streams::DataReader;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

#[derive(Default)]
pub struct Media {
    selected: Mutex<Option<String>>,
    art: Mutex<String>,
}

#[derive(Serialize, Clone)]
struct MediaSession {
    id: String,
    app: String,
}

#[derive(Serialize, Clone, Default)]
struct MediaState {
    sessions: Vec<MediaSession>,
    active: Option<String>,
    title: String,
    artist: String,
    album: String,
    playing: bool,
    position: f64,
    duration: f64,
    art_key: String,
}

pub fn mta() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

fn friendly(id: &str) -> String {
    let l = id.to_lowercase();
    for (k, v) in [
        ("spotify", "Spotify"),
        ("msedge", "Edge"),
        ("chrome", "Chrome"),
        ("firefox", "Firefox"),
        ("brave", "Brave"),
        ("opera", "Opera"),
        ("zunemusic", "Media Player"),
        ("vlc", "VLC"),
        ("discord", "Discord"),
        ("deezer", "Deezer"),
        ("tidal", "Tidal"),
        ("youtube", "YouTube Music"),
    ] {
        if l.contains(k) {
            return v.into();
        }
    }
    let last = id.rsplit(['\\', '/', '!']).next().unwrap_or(id);
    let last = last.trim_end_matches(".exe").trim_end_matches(".EXE");
    let mut c = last.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

fn find(mgr: &Mgr, selected: &Option<String>) -> Option<Session> {
    if let Some(sel) = selected {
        if let Ok(list) = mgr.GetSessions() {
            for s in list {
                if s.SourceAppUserModelId().map(|h| h.to_string() == *sel).unwrap_or(false) {
                    return Some(s);
                }
            }
        }
    }
    mgr.GetCurrentSession().ok()
}

fn now_ticks() -> i64 {
    // 100ns desde 1601 (FILETIME), mesma base de LastUpdatedTime
    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    (d.as_nanos() / 100) as i64 + 116_444_736_000_000_000
}

fn read_art(s: &Session) -> windows::core::Result<String> {
    let props = s.TryGetMediaPropertiesAsync()?.join()?;
    let stream = props.Thumbnail()?.OpenReadAsync()?.join()?;
    let size = stream.Size()? as u32;
    let mime = stream.ContentType().map(|h| h.to_string()).unwrap_or_default();
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0)?)?;
    reader.LoadAsync(size)?.join()?;
    let mut buf = vec![0u8; size as usize];
    reader.ReadBytes(&mut buf)?;
    let mime = if mime.is_empty() { "image/jpeg".into() } else { mime };
    Ok(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(buf)))
}

fn snapshot(app: &AppHandle, mgr: &Mgr, last_key: &mut String) -> windows::core::Result<MediaState> {
    let media = app.state::<Media>();
    let selected = media.selected.lock().map(|g| g.clone()).unwrap_or(None);
    let mut st = MediaState::default();
    for s in mgr.GetSessions()? {
        let id = s.SourceAppUserModelId()?.to_string();
        st.sessions.push(MediaSession { app: friendly(&id), id });
    }
    let Some(s) = find(mgr, &selected) else {
        if !last_key.is_empty() {
            last_key.clear();
            if let Ok(mut a) = media.art.lock() {
                a.clear();
            }
        }
        return Ok(st);
    };
    st.active = Some(s.SourceAppUserModelId()?.to_string());
    let props = s.TryGetMediaPropertiesAsync()?.join()?;
    st.title = props.Title()?.to_string();
    st.artist = props.Artist()?.to_string();
    st.album = props.AlbumTitle()?.to_string();
    st.playing = s.GetPlaybackInfo()?.PlaybackStatus()? == Status::Playing;
    let tl = s.GetTimelineProperties()?;
    let (start, end, pos) = (tl.StartTime()?.Duration, tl.EndTime()?.Duration, tl.Position()?.Duration);
    let mut p = pos - start;
    if st.playing {
        p += now_ticks() - tl.LastUpdatedTime()?.UniversalTime;
    }
    st.duration = ((end - start).max(0)) as f64 / 1e7;
    st.position = (p.max(0) as f64 / 1e7).min(if st.duration > 0.0 { st.duration } else { f64::MAX });

    let mut h = std::collections::hash_map::DefaultHasher::new();
    (&st.active, &st.title, &st.artist, &st.album).hash(&mut h);
    let key = format!("{:x}", h.finish());
    if key != *last_key {
        let art = read_art(&s).unwrap_or_default();
        if !art.is_empty() || st.title.is_empty() {
            // só fixa a chave quando a capa chegou (players às vezes publicam a capa depois do título)
            *last_key = key;
        }
        if let Ok(mut a) = media.art.lock() {
            *a = art;
        }
    }
    st.art_key = if media.art.lock().map(|a| a.is_empty()).unwrap_or(true) { String::new() } else { last_key.clone() };
    Ok(st)
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        mta();
        let mut mgr: Option<Mgr> = None;
        let mut last_key = String::new();
        loop {
            if mgr.is_none() {
                mgr = Mgr::RequestAsync().and_then(|op| op.join()).ok();
            }
            if let Some(m) = &mgr {
                match snapshot(&app, m, &mut last_key) {
                    Ok(st) => {
                        let _ = app.emit("media", st);
                    }
                    Err(e) => eprintln!("media: {e}"),
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

#[tauri::command]
pub fn media_art(media: State<Media>) -> String {
    media.art.lock().map(|a| a.clone()).unwrap_or_default()
}

#[tauri::command]
pub async fn media_select(media: State<'_, Media>, id: Option<String>) -> Result<(), String> {
    *media.selected.lock().map_err(|e| e.to_string())? = id;
    Ok(())
}

#[tauri::command]
pub async fn media_control(media: State<'_, Media>, action: String, value: Option<f64>) -> Result<(), String> {
    let selected = media.selected.lock().map(|g| g.clone()).unwrap_or(None);
    tauri::async_runtime::spawn_blocking(move || -> windows::core::Result<()> {
        mta();
        let mgr = Mgr::RequestAsync()?.join()?;
        let Some(s) = find(&mgr, &selected) else { return Ok(()) };
        match action.as_str() {
            "toggle" => s.TryTogglePlayPauseAsync()?.join()?,
            "next" => s.TrySkipNextAsync()?.join()?,
            "prev" => s.TrySkipPreviousAsync()?.join()?,
            "seek" => {
                let start = s.GetTimelineProperties()?.StartTime()?.Duration;
                s.TryChangePlaybackPositionAsync(start + (value.unwrap_or(0.0) * 1e7) as i64)?.join()?
            }
            _ => false,
        };
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    // hardware real, só leitura: cargo test -- --ignored
    #[test]
    #[ignore]
    fn reads_session() {
        super::mta();
        let mgr = super::Mgr::RequestAsync().unwrap().join().unwrap();
        if let Some(s) = super::find(&mgr, &None) {
            let id = s.SourceAppUserModelId().unwrap().to_string();
            let art = super::read_art(&s).map(|a| a.len()).unwrap_or(0);
            println!("{id} -> {} | art {art} bytes", super::friendly(&id));
        }
        assert_eq!(super::friendly("Spotify.exe"), "Spotify");
        assert_eq!(super::friendly("C:/x/foobar.exe"), "Foobar");
    }
}
