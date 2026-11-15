// Letras via LRCLIB (aberto, sem chave). Spotify não expõe letras na API pública.
use serde_json::Value;
use std::time::Duration;

fn fetch(artist: &str, title: &str, album: &str, duration: f64) -> Result<Option<Value>, String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).user_agent("Painel/0.1").build();
    let exact = agent
        .get("https://lrclib.net/api/get")
        .query("artist_name", artist)
        .query("track_name", title)
        .query("album_name", album)
        .query("duration", &(duration.round() as i64).to_string())
        .call();
    match exact {
        Ok(r) => return r.into_json().map(Some).map_err(|e| e.to_string()),
        Err(ureq::Error::Status(404, _)) => {}
        Err(e) => return Err(e.to_string()),
    }
    // sem match exato (título com "- Remastered", álbum diferente…): busca e prefere a sincronizada
    let list: Vec<Value> = agent
        .get("https://lrclib.net/api/search")
        .query("artist_name", artist)
        .query("track_name", title)
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;
    let synced = list.iter().find(|v| v["syncedLyrics"].is_string()).cloned();
    Ok(synced.or_else(|| list.into_iter().next()))
}

/// Retorna o objeto do LRCLIB ({ syncedLyrics, plainLyrics, instrumental, ... }) ou null.
#[tauri::command]
pub async fn lyrics(artist: String, title: String, album: String, duration: f64) -> Result<Option<Value>, String> {
    tauri::async_runtime::spawn_blocking(move || fetch(&artist, &title, &album, duration))
        .await
        .map_err(|e| e.to_string())?
}
