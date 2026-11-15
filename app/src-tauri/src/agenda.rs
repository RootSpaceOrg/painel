use crate::db::{self, Db};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use rrule::{RRule, Tz, Unvalidated};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Clone)]
pub struct Ev {
    uid: String,
    title: String,
    location: String,
    notes: String,
    start: i64,
    end: i64,
    all_day: bool,
}

#[derive(Default)]
pub struct FeedCache {
    events: Vec<Ev>,
    error: Option<String>,
    last_sync: Option<i64>,
}

pub struct Cache(pub Mutex<HashMap<i64, FeedCache>>);

#[derive(Serialize)]
pub struct CalEvent {
    id: String,
    local_id: Option<i64>,
    title: String,
    location: String,
    notes: String,
    start: i64,
    end: i64,
    all_day: bool,
    color: String,
    feed: Option<i64>,
}

#[derive(Serialize)]
pub struct Feed {
    id: i64,
    name: String,
    url: String,
    color: String,
    error: Option<String>,
    last_sync: Option<i64>,
}

// ---------------- parser ICS ----------------

struct Prop {
    name: String,
    params: Vec<(String, String)>,
    value: String,
}

impl Prop {
    fn param(&self, k: &str) -> Option<&str> {
        self.params.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str())
    }
}

fn parse_line(line: &str) -> Option<Prop> {
    // separa no primeiro ':' fora de aspas
    let mut in_q = false;
    let mut split = None;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' => in_q = !in_q,
            ':' if !in_q => {
                split = Some(i);
                break;
            }
            _ => {}
        }
    }
    let i = split?;
    let (head, value) = (&line[..i], &line[i + 1..]);
    let mut parts = head.split(';');
    let name = parts.next()?.to_ascii_uppercase();
    let params = parts
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (k.to_ascii_uppercase(), v.trim_matches('"').to_string()))
        .collect();
    Some(Prop { name, params, value: value.to_string() })
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') | Some('N') => out.push('\n'),
                Some(o) => out.push(o),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Interpreta um valor DATE / DATE-TIME. Retorna (instante no fuso certo, é data pura).
fn parse_dt(value: &str, tzid: Option<&str>) -> Option<(DateTime<Tz>, bool)> {
    let v = value.trim();
    if v.len() == 8 {
        let d = NaiveDate::parse_from_str(v, "%Y%m%d").ok()?;
        return Some((Tz::LOCAL.from_local_datetime(&d.and_hms_opt(0, 0, 0)?).earliest()?, true));
    }
    if let Some(u) = v.strip_suffix('Z') {
        let n = NaiveDateTime::parse_from_str(u, "%Y%m%dT%H%M%S").ok()?;
        return Some((Tz::UTC.from_utc_datetime(&n), false));
    }
    let n = NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S").ok()?;
    // TZID não-IANA (ex.: nomes do Windows no Outlook) cai para hora local
    let tz = tzid.and_then(|t| t.parse::<chrono_tz::Tz>().ok()).map(Tz::Tz).unwrap_or(Tz::LOCAL);
    let dt = tz.from_local_datetime(&n).earliest().or_else(|| Tz::LOCAL.from_local_datetime(&n).earliest())?;
    Some((dt, false))
}

/// ISO 8601 duration simples (P1D, PT1H30M, P1W, -PT15M ignorado o sinal).
fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim().trim_start_matches(['+', '-']).strip_prefix('P')?;
    let (mut total, mut num, mut in_time) = (Duration::zero(), String::new(), false);
    for c in s.chars() {
        match c {
            'T' => in_time = true,
            '0'..='9' => num.push(c),
            _ => {
                let n: i64 = num.parse().ok()?;
                num.clear();
                total += match (c, in_time) {
                    ('W', _) => Duration::weeks(n),
                    ('D', _) => Duration::days(n),
                    ('H', true) => Duration::hours(n),
                    ('M', true) => Duration::minutes(n),
                    ('S', true) => Duration::seconds(n),
                    _ => return None,
                };
            }
        }
    }
    Some(total)
}

#[derive(Default)]
struct Raw {
    uid: String,
    title: String,
    location: String,
    notes: String,
    start: Option<(DateTime<Tz>, bool)>,
    end: Option<(DateTime<Tz>, bool)>,
    duration: Option<Duration>,
    rrule: Option<String>,
    exdates: Vec<DateTime<Tz>>,
    recurrence_id: Option<DateTime<Tz>>,
    cancelled: bool,
}

fn add_days(dt: &DateTime<Tz>, days: i64) -> i64 {
    // dia inteiro: soma dias de calendário em hora local (seguro em horário de verão)
    let d = dt.with_timezone(&Tz::LOCAL).date_naive() + Duration::days(days);
    d.and_hms_opt(0, 0, 0)
        .and_then(|n| Tz::LOCAL.from_local_datetime(&n).earliest())
        .map(|x| x.timestamp_millis())
        .unwrap_or(dt.timestamp_millis() + days * 86_400_000)
}

/// Expande os VEVENTs do texto ICS dentro de [from, to) (epoch ms).
pub fn parse_ics(text: &str, from: i64, to: i64) -> Vec<Ev> {
    // unfold: linhas iniciadas por espaço/tab continuam a anterior
    let mut lines: Vec<String> = Vec::new();
    for l in text.split('\n') {
        let l = l.strip_suffix('\r').unwrap_or(l);
        if (l.starts_with(' ') || l.starts_with('\t')) && !lines.is_empty() {
            lines.last_mut().unwrap().push_str(&l[1..]);
        } else {
            lines.push(l.to_string());
        }
    }

    let mut raws: Vec<Raw> = Vec::new();
    let mut cur: Option<Raw> = None;
    let mut depth = 0; // ignora VALARM e afins dentro do VEVENT
    for l in &lines {
        let Some(p) = parse_line(l) else { continue };
        match (p.name.as_str(), p.value.as_str()) {
            ("BEGIN", "VEVENT") => {
                cur = Some(Raw::default());
                depth = 0;
            }
            ("END", "VEVENT") => raws.extend(cur.take()),
            ("BEGIN", _) if cur.is_some() => depth += 1,
            ("END", _) if cur.is_some() => depth -= 1,
            _ => {
                let Some(r) = cur.as_mut() else { continue };
                if depth > 0 {
                    continue;
                }
                let tz = p.param("TZID");
                match p.name.as_str() {
                    "UID" => r.uid = p.value.clone(),
                    "SUMMARY" => r.title = unescape(&p.value),
                    "LOCATION" => r.location = unescape(&p.value),
                    "DESCRIPTION" => r.notes = unescape(&p.value),
                    "DTSTART" => r.start = parse_dt(&p.value, tz),
                    "DTEND" => r.end = parse_dt(&p.value, tz),
                    "DURATION" => r.duration = parse_duration(&p.value),
                    "RRULE" => r.rrule = Some(p.value.clone()),
                    "EXDATE" => r.exdates.extend(p.value.split(',').filter_map(|v| parse_dt(v, tz)).map(|x| x.0)),
                    "RECURRENCE-ID" => r.recurrence_id = parse_dt(&p.value, tz).map(|x| x.0),
                    "STATUS" => r.cancelled = p.value.eq_ignore_ascii_case("CANCELLED"),
                    _ => {}
                }
            }
        }
    }

    // overrides (RECURRENCE-ID) removem a ocorrência original do mestre
    let mut overridden: HashMap<String, Vec<DateTime<Tz>>> = HashMap::new();
    for r in raws.iter().filter(|r| r.recurrence_id.is_some()) {
        overridden.entry(r.uid.clone()).or_default().push(r.recurrence_id.unwrap());
    }

    let mut out = Vec::new();
    for r in &raws {
        let Some((start, all_day)) = r.start else { continue };
        // duração: DTEND > DURATION > padrão (1 dia para data pura, 0 para horário)
        let (dur_ms, dur_days) = if all_day {
            let days = match (&r.end, r.duration) {
                (Some((e, _)), _) => (e.with_timezone(&Tz::LOCAL).date_naive() - start.with_timezone(&Tz::LOCAL).date_naive()).num_days(),
                (None, Some(d)) => d.num_days(),
                _ => 1,
            };
            (0, days.max(1))
        } else {
            let ms = match (&r.end, r.duration) {
                (Some((e, _)), _) => e.timestamp_millis() - start.timestamp_millis(),
                (None, Some(d)) => d.num_milliseconds(),
                _ => 0,
            };
            (ms.max(0), 0)
        };
        let end_of = |s: &DateTime<Tz>| if all_day { add_days(s, dur_days) } else { s.timestamp_millis() + dur_ms };

        let starts: Vec<DateTime<Tz>> = match (&r.rrule, r.recurrence_id) {
            (Some(rule), None) => {
                let set = rule.parse::<RRule<Unvalidated>>().and_then(|rr| rr.build(start));
                match set {
                    Ok(mut set) => {
                        for x in r.exdates.iter().chain(overridden.get(&r.uid).into_iter().flatten()) {
                            set = set.exdate(*x);
                        }
                        // começa antes de `from` o suficiente para pegar eventos longos em andamento
                        let lookback = (dur_ms + dur_days * 86_400_000).max(0);
                        let a = Tz::UTC.timestamp_millis_opt(from - lookback - 1).single();
                        let b = Tz::UTC.timestamp_millis_opt(to).single();
                        match (a, b) {
                            (Some(a), Some(b)) => set.after(a).before(b).all(2000).dates,
                            _ => vec![],
                        }
                    }
                    Err(err) => {
                        eprintln!("rrule inválida em '{}': {err}", r.title);
                        vec![start]
                    }
                }
            }
            _ => vec![start],
        };

        if r.cancelled {
            continue;
        }
        for s in starts {
            let (st, en) = (s.timestamp_millis(), end_of(&s));
            if en.max(st + 1) > from && st < to {
                out.push(Ev {
                    uid: r.uid.clone(),
                    title: r.title.clone(),
                    location: r.location.clone(),
                    notes: r.notes.clone(),
                    start: st,
                    end: en,
                    all_day,
                });
            }
        }
    }
    out
}

// ---------------- sync ----------------

fn window() -> (i64, i64) {
    let now = Utc::now().timestamp_millis();
    (now - 7 * 86_400_000, now + 90 * 86_400_000)
}

pub fn sync_all(app: &AppHandle) {
    let feeds = match app.state::<Db>().0.lock() {
        Ok(c) => db::feeds_raw(&c).unwrap_or_default(),
        Err(_) => return,
    };
    let (from, to) = window();
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(30)).build();
    for (id, _, url, _) in feeds {
        let res = agent.get(&url).call().map_err(|e| e.to_string()).and_then(|r| r.into_string().map_err(|e| e.to_string()));
        let now = Utc::now().timestamp_millis();
        let cache = app.state::<Cache>();
        let Ok(mut m) = cache.0.lock() else { return };
        let entry = m.entry(id).or_default();
        match res {
            Ok(text) if text.contains("BEGIN:VCALENDAR") => {
                entry.events = parse_ics(&text, from, to);
                entry.error = None;
                entry.last_sync = Some(now);
            }
            Ok(_) => entry.error = Some("Resposta não é um calendário ICS".into()),
            Err(e) => entry.error = Some(e), // mantém eventos antigos em cache
        }
    }
    let _ = app.emit("agenda-changed", ());
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || loop {
        sync_all(&app);
        std::thread::sleep(std::time::Duration::from_secs(15 * 60));
    });
}

// ---------------- comandos ----------------

#[tauri::command]
pub fn agenda(db: State<Db>, cache: State<Cache>, from: i64, to: i64) -> Result<Vec<CalEvent>, String> {
    let c = db.0.lock().map_err(|e| e.to_string())?;
    let mut out: Vec<CalEvent> = db::local_events(&c, from, to)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(id, title, location, notes, start, end, all_day)| CalEvent {
            id: format!("l:{id}"),
            local_id: Some(id),
            title,
            location,
            notes,
            start,
            end,
            all_day,
            color: String::new(),
            feed: None,
        })
        .collect();
    let colors: HashMap<i64, String> = db::feeds_raw(&c).unwrap_or_default().into_iter().map(|f| (f.0, f.3)).collect();
    drop(c);
    let m = cache.0.lock().map_err(|e| e.to_string())?;
    for (fid, fc) in m.iter() {
        let Some(color) = colors.get(fid) else { continue };
        for ev in fc.events.iter().filter(|e| e.end.max(e.start + 1) > from && e.start < to) {
            out.push(CalEvent {
                id: format!("f{fid}:{}:{}", ev.uid, ev.start),
                local_id: None,
                title: ev.title.clone(),
                location: ev.location.clone(),
                notes: ev.notes.clone(),
                start: ev.start,
                end: ev.end,
                all_day: ev.all_day,
                color: color.clone(),
                feed: Some(*fid),
            });
        }
    }
    out.sort_by_key(|e| (e.start, !e.all_day));
    Ok(out)
}

#[tauri::command]
pub fn feeds_list(db: State<Db>, cache: State<Cache>) -> Result<Vec<Feed>, String> {
    let feeds = db::feeds_raw(&*db.0.lock().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let m = cache.0.lock().map_err(|e| e.to_string())?;
    Ok(feeds
        .into_iter()
        .map(|(id, name, url, color)| {
            let fc = m.get(&id);
            Feed { id, name, url, color, error: fc.and_then(|f| f.error.clone()), last_sync: fc.and_then(|f| f.last_sync) }
        })
        .collect())
}

#[tauri::command]
pub async fn feeds_refresh(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || sync_all(&app)).await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ICS: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
BEGIN:VEVENT\r\nUID:simple\r\nSUMMARY:Reuni\\, ão\r\nDTSTART:20260310T130000Z\r\nDTEND:20260310T140000Z\r\n\
DESCRIPTION:linha1\\nlinha2 com texto que\r\n  continua\r\nBEGIN:VALARM\r\nSUMMARY:alarme\r\nEND:VALARM\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:allday\r\nSUMMARY:Feriado\r\nDTSTART;VALUE=DATE:20260312\r\nDTEND;VALUE=DATE:20260314\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:weekly\r\nSUMMARY:Daily\r\nDTSTART;TZID=America/Sao_Paulo:20260302T090000\r\nDURATION:PT30M\r\n\
RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=5\r\nEXDATE;TZID=America/Sao_Paulo:20260309T090000\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:weekly\r\nSUMMARY:Daily movida\r\nRECURRENCE-ID;TZID=America/Sao_Paulo:20260316T090000\r\n\
DTSTART;TZID=America/Sao_Paulo:20260316T150000\r\nDTEND;TZID=America/Sao_Paulo:20260316T153000\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:gone\r\nSUMMARY:Cancelado\r\nSTATUS:CANCELLED\r\nDTSTART:20260310T130000Z\r\nEND:VEVENT\r\n\
END:VCALENDAR\r\n";

    fn ms(s: &str) -> i64 {
        DateTime::parse_from_rfc3339(s).unwrap().timestamp_millis()
    }

    #[test]
    fn parses_and_expands() {
        let evs = parse_ics(ICS, ms("2026-03-01T00:00:00Z"), ms("2026-04-30T00:00:00Z"));
        let simple = evs.iter().find(|e| e.uid == "simple").unwrap();
        assert_eq!(simple.title, "Reuni, ão");
        assert_eq!(simple.notes, "linha1\nlinha2 com texto que continua");
        assert_eq!(simple.end - simple.start, 3_600_000);

        let allday = evs.iter().find(|e| e.uid == "allday").unwrap();
        assert!(allday.all_day);
        assert_eq!((allday.end - allday.start) / 3_600_000, 48);

        // 5 segundas: 02, 09 (EXDATE), 16 (override), 23, 30 → 3 da regra + 1 override
        let weekly: Vec<_> = evs.iter().filter(|e| e.uid == "weekly").collect();
        assert_eq!(weekly.len(), 4, "{:?}", weekly.iter().map(|e| (&e.title, e.start)).collect::<Vec<_>>());
        assert!(weekly.iter().any(|e| e.start == ms("2026-03-02T12:00:00Z") && e.end - e.start == 1_800_000));
        assert!(!weekly.iter().any(|e| e.start == ms("2026-03-09T12:00:00Z")));
        assert!(!weekly.iter().any(|e| e.start == ms("2026-03-16T12:00:00Z")));
        assert!(weekly.iter().any(|e| e.title == "Daily movida" && e.start == ms("2026-03-16T18:00:00Z")));

        assert!(!evs.iter().any(|e| e.uid == "gone"));
        assert!(!evs.iter().any(|e| e.title == "alarme"));
    }

    #[test]
    fn window_filters() {
        let evs = parse_ics(ICS, ms("2026-03-20T00:00:00Z"), ms("2026-03-24T00:00:00Z"));
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].start, ms("2026-03-23T12:00:00Z"));
    }
}
