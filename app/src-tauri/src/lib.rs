mod agenda;
mod audio;
mod autostart;
mod db;
mod lyrics;
mod media;
mod spotify;
mod sys;
mod updater;
mod window;

use tauri::{Manager, WindowEvent};

pub fn run() {
    tauri::Builder::default()
        .manage(media::Media::default())
        .manage(agenda::Cache(Default::default()))
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = db::open(&dir.join("painel.db"))?;
            autostart::init(&conn);
            app.manage(db::Db(std::sync::Mutex::new(conn)));
            let h = app.handle().clone();
            window::tray(&h)?;
            if let Err(e) = window::apply(&h, &window::load_cfg(&h)) {
                eprintln!("janela: {e}");
            }
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
            }
            media::start(h.clone());
            sys::start(h.clone());
            agenda::start(h);
            Ok(())
        })
        .on_window_event(|w, ev| {
            // fechar só esconde; sair é pela bandeja
            if let WindowEvent::CloseRequested { api, .. } = ev {
                api.prevent_close();
                let _ = w.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            db::kv_get, db::kv_set,
            db::todo_list, db::todo_save, db::todo_delete,
            db::habit_list, db::habit_save, db::habit_delete, db::habit_mark,
            db::event_save, db::event_delete, db::feed_save, db::feed_delete,
            agenda::agenda, agenda::feeds_list, agenda::feeds_refresh,
            media::media_art, media::media_control, media::media_select,
            audio::audio_state, audio::audio_set_master, audio::audio_set_master_mute,
            audio::audio_set_session, audio::audio_set_session_mute, audio::audio_set_default,
            window::monitors_list, window::window_get, window::window_apply,
            window::open_url,
            window::wallpaper_set, window::wallpaper_get, window::wallpaper_clear,
            spotify::spotify_status, spotify::spotify_set_client_id, spotify::spotify_connect,
            spotify::spotify_disconnect, spotify::spotify_api,
            autostart::autostart_get, autostart::autostart_set,
            lyrics::lyrics,
            updater::update_check, updater::update_install,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Painel");
}
