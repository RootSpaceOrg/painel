// Atualização pelos releases públicos de RootSpaceOrg/painel (sem login), igual à Central:
// consulta o último release, baixa o instalador NSIS e roda em silêncio; o app fecha e reabre.
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use tauri::AppHandle;

const REPO: &str = "RootSpaceOrg/painel";

#[derive(Serialize)]
pub struct UpdateInfo {
    current: String,
    latest: Option<String>,
    available: bool,
    notes: String,
    url: Option<String>,
    error: Option<String>,
}

fn parts(v: &str) -> Vec<u64> {
    v.trim_start_matches('v').split('.').map(|p| p.parse().unwrap_or(0)).collect()
}
fn newer(latest: &str, current: &str) -> bool {
    parts(latest) > parts(current)
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(60)).user_agent("Painel-updater").build()
}

fn latest_release() -> Result<Option<Value>, String> {
    match agent()
        .get(&format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .set("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(r) => r.into_json().map(Some).map_err(|e| e.to_string()),
        Err(ureq::Error::Status(404, _)) => Ok(None), // ainda sem release
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_check(app: AppHandle) -> UpdateInfo {
    let current = app.package_info().version.to_string();
    let res = tauri::async_runtime::spawn_blocking(latest_release).await.map_err(|e| e.to_string()).and_then(|r| r);
    match res {
        Ok(Some(rel)) => {
            let latest = rel["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
            let url = rel["assets"].as_array().and_then(|a| {
                a.iter()
                    .find(|x| x["name"].as_str().is_some_and(|n| n.ends_with("-setup.exe")))
                    .and_then(|x| x["browser_download_url"].as_str().map(String::from))
            });
            UpdateInfo {
                available: !latest.is_empty() && newer(&latest, &current) && url.is_some(),
                notes: rel["body"].as_str().unwrap_or("").to_string(),
                latest: Some(latest),
                current,
                url,
                error: None,
            }
        }
        Ok(None) => UpdateInfo { current, latest: None, available: false, notes: String::new(), url: None, error: None },
        Err(e) => UpdateInfo {
            current, latest: None, available: false, notes: String::new(), url: None,
            error: Some(format!("Não deu para consultar o GitHub (sem internet?): {e}")),
        },
    }
}

#[tauri::command]
pub async fn update_install(app: AppHandle, url: String) -> Result<(), String> {
    // fronteira de confiança: só instala o que veio dos releases deste repositório
    let prefix = format!("https://github.com/{REPO}/releases/download/");
    if !url.starts_with(&prefix) || !url.ends_with("-setup.exe") {
        return Err("URL de atualização inválida.".into());
    }
    let installer = std::env::temp_dir().join("painel-update-setup.exe");
    let dest = installer.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let r = agent().get(&url).call().map_err(|e| e.to_string())?;
        let mut f = std::fs::File::create(&dest).map_err(|e| e.to_string())?;
        std::io::copy(&mut r.into_reader(), &mut f).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let q = |p: &std::path::Path| p.display().to_string().replace('\'', "''");
    // espera o app fechar, instala em silêncio (/S) e reabre
    let script = format!(
        "Start-Sleep -Seconds 2; Start-Process -Wait -FilePath '{}' -ArgumentList '/S'; Start-Process -FilePath '{}'",
        q(&installer),
        q(&exe)
    );
    use std::os::windows::process::CommandExt;
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| e.to_string())?;
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::newer;
    #[test]
    fn versions() {
        assert!(newer("0.2.0", "0.1.9"));
        assert!(newer("v0.10.0", "0.9.0"));
        assert!(!newer("0.1.0", "0.1.0"));
        assert!(!newer("0.1.0", "0.2.0"));
    }
}
