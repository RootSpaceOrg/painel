use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct Db(pub Mutex<Connection>);

type R<T> = Result<T, String>;
fn e<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

pub fn open(path: &std::path::Path) -> rusqlite::Result<Connection> {
    let c = Connection::open(path)?;
    c.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS kv(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS todos(id INTEGER PRIMARY KEY, title TEXT NOT NULL, days INTEGER NOT NULL DEFAULT 0,
            done_on TEXT, created TEXT NOT NULL DEFAULT (datetime('now')));
         CREATE TABLE IF NOT EXISTS habits(id INTEGER PRIMARY KEY, name TEXT NOT NULL, icon TEXT NOT NULL DEFAULT '',
            days INTEGER NOT NULL DEFAULT 127, position INTEGER NOT NULL DEFAULT 0);
         CREATE TABLE IF NOT EXISTS habit_log(habit_id INTEGER NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
            day TEXT NOT NULL, PRIMARY KEY(habit_id, day));
         CREATE TABLE IF NOT EXISTS events(id INTEGER PRIMARY KEY, title TEXT NOT NULL, start INTEGER NOT NULL, end INTEGER NOT NULL,
            all_day INTEGER NOT NULL DEFAULT 0, notes TEXT NOT NULL DEFAULT '', location TEXT NOT NULL DEFAULT '');
         CREATE TABLE IF NOT EXISTS feeds(id INTEGER PRIMARY KEY, name TEXT NOT NULL, url TEXT NOT NULL, color TEXT NOT NULL DEFAULT '');",
    )?;
    Ok(c)
}

// ---------- kv ----------
pub fn kv_read(c: &Connection, key: &str) -> Option<String> {
    c.query_row("SELECT value FROM kv WHERE key=?1", [key], |r| r.get(0)).optional().ok().flatten()
}
pub fn kv_write(c: &Connection, key: &str, value: &str) -> R<()> {
    c.execute("INSERT INTO kv(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [key, value])
        .map(|_| ())
        .map_err(e)
}

#[tauri::command]
pub fn kv_get(db: State<Db>, key: String) -> R<Option<String>> {
    Ok(kv_read(&*db.0.lock().map_err(e)?, &key))
}
#[tauri::command]
pub fn kv_set(db: State<Db>, key: String, value: String) -> R<()> {
    kv_write(&*db.0.lock().map_err(e)?, &key, &value)
}

// ---------- todos ----------
#[derive(Serialize, Deserialize)]
pub struct Todo {
    pub id: Option<i64>,
    pub title: String,
    pub days: i64,
    pub done_on: Option<String>,
}

#[tauri::command]
pub fn todo_list(db: State<Db>) -> R<Vec<Todo>> {
    let c = db.0.lock().map_err(e)?;
    let mut s = c.prepare("SELECT id,title,days,done_on FROM todos ORDER BY id").map_err(e)?;
    let rows = s
        .query_map([], |r| Ok(Todo { id: r.get(0)?, title: r.get(1)?, days: r.get(2)?, done_on: r.get(3)? }))
        .map_err(e)?;
    rows.collect::<Result<_, _>>().map_err(e)
}
#[tauri::command]
pub fn todo_save(db: State<Db>, todo: Todo) -> R<i64> {
    let c = db.0.lock().map_err(e)?;
    match todo.id {
        Some(id) => {
            c.execute("UPDATE todos SET title=?2, days=?3, done_on=?4 WHERE id=?1", params![id, todo.title, todo.days, todo.done_on])
                .map_err(e)?;
            Ok(id)
        }
        None => {
            c.execute("INSERT INTO todos(title,days,done_on) VALUES(?1,?2,?3)", params![todo.title, todo.days, todo.done_on])
                .map_err(e)?;
            Ok(c.last_insert_rowid())
        }
    }
}
#[tauri::command]
pub fn todo_delete(db: State<Db>, id: i64) -> R<()> {
    db.0.lock().map_err(e)?.execute("DELETE FROM todos WHERE id=?1", [id]).map(|_| ()).map_err(e)
}

// ---------- habits ----------
#[derive(Serialize)]
pub struct Habit {
    id: i64,
    name: String,
    icon: String,
    days: i64,
    log: Vec<String>,
}
#[derive(Deserialize)]
pub struct HabitIn {
    id: Option<i64>,
    name: String,
    icon: String,
    days: i64,
}

#[tauri::command]
pub fn habit_list(db: State<Db>) -> R<Vec<Habit>> {
    let c = db.0.lock().map_err(e)?;
    let mut s = c.prepare("SELECT id,name,icon,days FROM habits ORDER BY position,id").map_err(e)?;
    let mut habits: Vec<Habit> = s
        .query_map([], |r| Ok(Habit { id: r.get(0)?, name: r.get(1)?, icon: r.get(2)?, days: r.get(3)?, log: vec![] }))
        .map_err(e)?
        .collect::<Result<_, _>>()
        .map_err(e)?;
    let since = (chrono::Local::now().date_naive() - chrono::Duration::days(400)).format("%Y-%m-%d").to_string();
    let mut s = c.prepare("SELECT day FROM habit_log WHERE habit_id=?1 AND day>=?2 ORDER BY day").map_err(e)?;
    for h in &mut habits {
        h.log = s.query_map(params![h.id, since], |r| r.get(0)).map_err(e)?.collect::<Result<_, _>>().map_err(e)?;
    }
    Ok(habits)
}
#[tauri::command]
pub fn habit_save(db: State<Db>, habit: HabitIn) -> R<i64> {
    let c = db.0.lock().map_err(e)?;
    match habit.id {
        Some(id) => {
            c.execute("UPDATE habits SET name=?2, icon=?3, days=?4 WHERE id=?1", params![id, habit.name, habit.icon, habit.days])
                .map_err(e)?;
            Ok(id)
        }
        None => {
            c.execute(
                "INSERT INTO habits(name,icon,days,position) VALUES(?1,?2,?3,(SELECT COALESCE(MAX(position),0)+1 FROM habits))",
                params![habit.name, habit.icon, habit.days],
            )
            .map_err(e)?;
            Ok(c.last_insert_rowid())
        }
    }
}
#[tauri::command]
pub fn habit_delete(db: State<Db>, id: i64) -> R<()> {
    db.0.lock().map_err(e)?.execute("DELETE FROM habits WHERE id=?1", [id]).map(|_| ()).map_err(e)
}
#[tauri::command]
pub fn habit_mark(db: State<Db>, id: i64, day: String, done: bool) -> R<()> {
    let c = db.0.lock().map_err(e)?;
    let sql = if done {
        "INSERT OR IGNORE INTO habit_log(habit_id,day) VALUES(?1,?2)"
    } else {
        "DELETE FROM habit_log WHERE habit_id=?1 AND day=?2"
    };
    c.execute(sql, params![id, day]).map(|_| ()).map_err(e)
}

// ---------- events (locais) ----------
#[derive(Deserialize)]
pub struct EventIn {
    id: Option<i64>,
    title: String,
    start: i64,
    end: i64,
    all_day: bool,
    notes: String,
    location: String,
}

#[tauri::command]
pub fn event_save(db: State<Db>, event: EventIn) -> R<i64> {
    let c = db.0.lock().map_err(e)?;
    let ev = &event;
    match ev.id {
        Some(id) => {
            c.execute(
                "UPDATE events SET title=?2,start=?3,end=?4,all_day=?5,notes=?6,location=?7 WHERE id=?1",
                params![id, ev.title, ev.start, ev.end, ev.all_day, ev.notes, ev.location],
            )
            .map_err(e)?;
            Ok(id)
        }
        None => {
            c.execute(
                "INSERT INTO events(title,start,end,all_day,notes,location) VALUES(?1,?2,?3,?4,?5,?6)",
                params![ev.title, ev.start, ev.end, ev.all_day, ev.notes, ev.location],
            )
            .map_err(e)?;
            Ok(c.last_insert_rowid())
        }
    }
}
#[tauri::command]
pub fn event_delete(db: State<Db>, id: i64) -> R<()> {
    db.0.lock().map_err(e)?.execute("DELETE FROM events WHERE id=?1", [id]).map(|_| ()).map_err(e)
}

/// (id, title, location, notes, start, end, all_day) dos eventos locais que cruzam [from, to).
pub fn local_events(c: &Connection, from: i64, to: i64) -> rusqlite::Result<Vec<(i64, String, String, String, i64, i64, bool)>> {
    let mut s = c.prepare("SELECT id,title,location,notes,start,end,all_day FROM events WHERE end>?1 AND start<?2")?;
    let rows = s.query_map([from, to], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)))?;
    rows.collect()
}

// ---------- feeds ----------
#[derive(Deserialize)]
pub struct FeedIn {
    id: Option<i64>,
    name: String,
    url: String,
    color: String,
}

pub fn feeds_raw(c: &Connection) -> rusqlite::Result<Vec<(i64, String, String, String)>> {
    let mut s = c.prepare("SELECT id,name,url,color FROM feeds ORDER BY id")?;
    let rows = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
    rows.collect()
}

#[tauri::command]
pub fn feed_save(db: State<Db>, feed: FeedIn) -> R<i64> {
    let url = feed.url.trim().replacen("webcal://", "https://", 1);
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("URL precisa começar com https:// (ou webcal://)".into());
    }
    let c = db.0.lock().map_err(e)?;
    match feed.id {
        Some(id) => {
            c.execute("UPDATE feeds SET name=?2,url=?3,color=?4 WHERE id=?1", params![id, feed.name, url, feed.color]).map_err(e)?;
            Ok(id)
        }
        None => {
            c.execute("INSERT INTO feeds(name,url,color) VALUES(?1,?2,?3)", params![feed.name, url, feed.color]).map_err(e)?;
            Ok(c.last_insert_rowid())
        }
    }
}
#[tauri::command]
pub fn feed_delete(db: State<Db>, cache: State<crate::agenda::Cache>, id: i64) -> R<()> {
    db.0.lock().map_err(e)?.execute("DELETE FROM feeds WHERE id=?1", [id]).map_err(e)?;
    if let Ok(mut m) = cache.0.lock() {
        m.remove(&id);
    }
    Ok(())
}
