use crate::db::{self, Db};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const PORT: u16 = 43821;
const REDIRECT: &str = "http://127.0.0.1:43821/callback";
const SCOPES: &str = "user-read-playback-state user-modify-playback-state user-read-currently-playing user-library-read \
user-library-modify playlist-read-private playlist-read-collaborative";

#[derive(Serialize, Deserialize)]
struct Tokens {
    access: String,
    refresh: String,
    expires_at: i64,
}

#[derive(Serialize)]
pub struct SpotifyStatus {
    client_id: String,
    connected: bool,
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn kv(app: &AppHandle, key: &str) -> Option<String> {
    app.state::<Db>().0.lock().ok().and_then(|c| db::kv_read(&c, key))
}
fn set_kv(app: &AppHandle, key: &str, value: &str) -> Result<(), String> {
    let db = app.state::<Db>();
    let c = db.0.lock().map_err(|e| e.to_string())?;
    db::kv_write(&c, key, value)
}

fn client_id(app: &AppHandle) -> String {
    kv(app, "spotify_client_id").unwrap_or_default()
}
fn tokens(app: &AppHandle) -> Option<Tokens> {
    kv(app, "spotify_tokens").and_then(|s| serde_json::from_str(&s).ok())
}

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn ureq_err(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, r) => {
            let body = r.into_string().unwrap_or_default();
            let msg = serde_json::from_str::<Value>(&body)
                .ok()
                .and_then(|v| {
                    v.pointer("/error/message")
                        .or_else(|| v.get("error_description"))
                        .or_else(|| v.get("error"))
                        .and_then(|m| m.as_str().map(String::from))
                })
                .unwrap_or(body);
            format!("{code}: {msg}")
        }
        other => other.to_string(),
    }
}

fn store_token_response(app: &AppHandle, v: Value, old_refresh: Option<String>) -> Result<Tokens, String> {
    let access = v["access_token"].as_str().ok_or("resposta sem access_token")?.to_string();
    let refresh = v["refresh_token"].as_str().map(String::from).or(old_refresh).ok_or("resposta sem refresh_token")?;
    let t = Tokens { access, refresh, expires_at: now_ms() + v["expires_in"].as_i64().unwrap_or(3600) * 1000 - 60_000 };
    set_kv(app, "spotify_tokens", &serde_json::to_string(&t).map_err(|e| e.to_string())?)?;
    Ok(t)
}

fn refresh(app: &AppHandle, t: &Tokens) -> Result<Tokens, String> {
    let cid = client_id(app);
    let v: Value = ureq::post("https://accounts.spotify.com/api/token")
        .timeout(Duration::from_secs(20))
        .send_form(&[("grant_type", "refresh_token"), ("refresh_token", &t.refresh), ("client_id", &cid)])
        .map_err(ureq_err)?
        .into_json()
        .map_err(|e| e.to_string())?;
    store_token_response(app, v, Some(t.refresh.clone()))
}

fn connect(app: &AppHandle) -> Result<(), String> {
    let cid = client_id(app);
    if cid.is_empty() || !cid.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Configure um Client ID válido do Spotify primeiro".into());
    }
    let mut rnd = [0u8; 76];
    getrandom::fill(&mut rnd).map_err(|e| e.to_string())?;
    let verifier = b64url(&rnd[..64]);
    let challenge = b64url(&Sha256::digest(verifier.as_bytes()));
    let state = b64url(&rnd[64..]);

    let server = tiny_http::Server::http(("127.0.0.1", PORT)).map_err(|e| format!("porta {PORT} ocupada: {e}"))?;
    let url = format!(
        "https://accounts.spotify.com/authorize?response_type=code&client_id={cid}&scope={}&redirect_uri={}&code_challenge_method=S256&code_challenge={challenge}&state={state}",
        enc(SCOPES),
        enc(REDIRECT)
    );
    std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", &url]).spawn().map_err(|e| e.to_string())?;

    let deadline = Instant::now() + Duration::from_secs(180);
    let code = loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err("Tempo esgotado esperando a autorização do Spotify".into());
        }
        let Some(req) = server.recv_timeout(left).map_err(|e| e.to_string())? else { continue };
        let Some(query) = req.url().strip_prefix("/callback?").map(String::from) else {
            let _ = req.respond(tiny_http::Response::empty(404));
            continue;
        };
        let get = |k: &str| query.split('&').find_map(|p| p.strip_prefix(&format!("{k}="))).map(String::from);
        let html = tiny_http::Header::from_bytes("Content-Type", "text/html; charset=utf-8").ok();
        let page = |msg: &str| {
            let mut r = tiny_http::Response::from_string(format!(
                "<html><body style=\"background:#07161a;color:#e8f3ea;font-family:Segoe UI;display:grid;place-items:center;height:90vh\"><h2>{msg}</h2></body></html>"
            ));
            if let Some(h) = html.clone() {
                r.add_header(h);
            }
            r
        };
        if get("state").as_deref() != Some(state.as_str()) {
            let _ = req.respond(page("Estado inválido. Tente conectar de novo pelo Painel."));
            return Err("state inválido no retorno do Spotify".into());
        }
        if let Some(err) = get("error") {
            let _ = req.respond(page("Autorização negada. Pode fechar esta aba."));
            return Err(format!("Spotify recusou: {err}"));
        }
        let _ = req.respond(page("Painel conectado ao Spotify. Pode fechar esta aba."));
        break get("code").ok_or("retorno sem code")?;
    };

    let v: Value = ureq::post("https://accounts.spotify.com/api/token")
        .timeout(Duration::from_secs(20))
        .send_form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", REDIRECT),
            ("client_id", &cid),
            ("code_verifier", &verifier),
        ])
        .map_err(ureq_err)?
        .into_json()
        .map_err(|e| e.to_string())?;
    store_token_response(app, v, None).map(|_| ())
}

fn call(app: &AppHandle, method: &str, path: &str, body: &Option<Value>) -> Result<Value, String> {
    let mut t = tokens(app).ok_or("Spotify não conectado")?;
    if now_ms() >= t.expires_at {
        t = refresh(app, &t)?;
    }
    for attempt in 0..2 {
        let req = ureq::request(method, &format!("https://api.spotify.com/v1{path}"))
            .timeout(Duration::from_secs(20))
            .set("Authorization", &format!("Bearer {}", t.access));
        let res = match body {
            Some(b) => req.send_json(b.clone()),
            None if method == "GET" || method == "DELETE" => req.call(),
            None => req.set("Content-Length", "0").send_string(""),
        };
        match res {
            Ok(r) => {
                let text = r.into_string().map_err(|e| e.to_string())?;
                return Ok(serde_json::from_str(&text).unwrap_or(Value::Null));
            }
            Err(ureq::Error::Status(401, _)) if attempt == 0 => t = refresh(app, &t)?,
            Err(e) => return Err(ureq_err(e)),
        }
    }
    Err("401: não autorizado".into())
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn spotify_status(app: AppHandle) -> SpotifyStatus {
    SpotifyStatus { client_id: client_id(&app), connected: tokens(&app).is_some() }
}
#[tauri::command]
pub fn spotify_set_client_id(app: AppHandle, id: String) -> Result<(), String> {
    set_kv(&app, "spotify_client_id", id.trim())
}
#[tauri::command]
pub fn spotify_disconnect(app: AppHandle) -> Result<(), String> {
    let db = app.state::<Db>();
    let c = db.0.lock().map_err(|e| e.to_string())?;
    c.execute("DELETE FROM kv WHERE key='spotify_tokens'", []).map(|_| ()).map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn spotify_connect(app: AppHandle) -> Result<(), String> {
    blocking(move || connect(&app)).await
}
#[tauri::command]
pub async fn spotify_api(app: AppHandle, method: String, path: String, body: Option<Value>) -> Result<Value, String> {
    if !path.starts_with('/') {
        return Err("path deve começar com /".into());
    }
    blocking(move || call(&app, &method, &path, &body)).await
}
