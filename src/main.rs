use rusqlite::{Connection, Result as SqlResult, params};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

// ─────────────────────────────────────────────
//  CSS (single source of truth, embedded)
// ─────────────────────────────────────────────
const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Syne:wght@400;500;600;700;800&family=Literata:ital,wght@0,300;0,400;1,300;1,400&family=IBM+Plex+Mono:wght@400;500&display=swap');

*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

:root {
  --bg:          #0d0d0f;
  --bg-raised:   #131316;
  --bg-card:     #18181c;
  --bg-input:    #1e1e23;
  --line:        #2a2a30;
  --line-bright: #3a3a42;
  --fg:          #e8e6f0;
  --fg-muted:    #7a7888;
  --fg-dim:      #3e3d48;
  --lime:        #b8f55a;
  --lime-dim:    rgba(184,245,90,0.12);
  --lime-glow:   rgba(184,245,90,0.06);
  --red:         #ff5c5c;
  --red-dim:     rgba(255,92,92,0.1);
  --green:       #52d98a;
  --green-dim:   rgba(82,217,138,0.1);
  --radius:      3px;
  --font-head:   'Syne', system-ui, sans-serif;
  --font-body:   'Literata', Georgia, serif;
  --font-mono:   'IBM Plex Mono', 'Courier New', monospace;
}

html { font-size: 16px; }

body {
  background-color: var(--bg);
  background-image:
    radial-gradient(ellipse 70% 50% at 10% 0%, rgba(184,245,90,0.03) 0%, transparent 60%),
    radial-gradient(ellipse 50% 60% at 90% 100%, rgba(184,245,90,0.025) 0%, transparent 60%);
  color: var(--fg);
  font-family: var(--font-body);
  font-weight: 300;
  line-height: 1.65;
  min-height: 100vh;
  padding: 0 2rem 6rem;
}

/* ── Top bar ── */
.topbar {
  max-width: 860px;
  margin: 0 auto;
  padding: 1.5rem 0;
  border-bottom: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.topbar-id {
  font-family: var(--font-mono);
  font-size: 0.6rem;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: #9e9cb0;
}

.topbar-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--lime);
  box-shadow: 0 0 8px var(--lime);
}

/* ── Header ── */
header {
  max-width: 860px;
  margin: 0 auto;
  padding: 4rem 0 3.5rem;
  display: grid;
  grid-template-columns: 1fr auto;
  align-items: end;
  gap: 2rem;
  border-bottom: 1px solid var(--line);
}

header h1 {
  font-family: var(--font-head);
  font-size: clamp(2.8rem, 6vw, 5rem);
  font-weight: 800;
  line-height: 0.92;
  letter-spacing: -0.04em;
  color: var(--fg);
  text-transform: uppercase;
}

header h1 span {
  display: block;
  color: var(--lime);
  font-style: italic;
  font-weight: 400;
  font-family: var(--font-body);
  font-size: 0.45em;
  letter-spacing: 0.02em;
  text-transform: none;
  margin-bottom: 0.3rem;
}

.header-meta {
  text-align: right;
  font-family: var(--font-mono);
  font-size: 0.6rem;
  letter-spacing: 0.12em;
  color: #9e9cb0;
  text-transform: uppercase;
  line-height: 2;
}

/* ── Main layout ── */
main {
  max-width: 860px;
  margin: 0 auto;
}

/* ── Section titles ── */
.section-head {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin: 3rem 0 1.25rem;
}

.section-num {
  font-family: var(--font-mono);
  font-size: 0.6rem;
  color: var(--lime);
  letter-spacing: 0.1em;
  flex-shrink: 0;
}

.section-title {
  font-family: var(--font-head);
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--fg-muted);
  white-space: nowrap;
}

.section-rule {
  flex: 1;
  height: 1px;
  background: var(--line);
}

/* ── Card ── */
.card {
  background: var(--bg-card);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  overflow: hidden;
}

/* ── Form ── */
.form-inner { padding: 2.25rem 2.5rem 2.5rem; }

.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.25rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
  margin-bottom: 1.25rem;
}

.field:last-child { margin-bottom: 0; }

label {
  font-family: var(--font-mono);
  font-size: 0.6rem;
  font-weight: 500;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: #9e9cb0;
  display: flex;
  align-items: center;
  gap: 0.3rem;
}

.required-mark { color: var(--lime); opacity: 0.8; }

input[type="text"] {
  width: 100%;
  padding: 0.75rem 1rem;
  background: #242429;
  border: 1px solid #38383f;
  border-radius: var(--radius);
  font-family: var(--font-body);
  font-size: 0.95rem;
  font-weight: 300;
  color: var(--fg);
  outline: none;
  transition: border-color 0.18s, box-shadow 0.18s;
}

input[type="text"]:focus {
  border-color: var(--lime);
  box-shadow: 0 0 0 3px var(--lime-dim), inset 0 0 0 1px transparent;
}

input[type="text"]::placeholder {
  color: #8a8898;
  font-style: italic;
}

.field-hint {
  font-family: var(--font-mono);
  font-size: 0.6rem;
  color: #8a8898;
  letter-spacing: 0.05em;
}

/* ── Error/Success banners ── */
.banner {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  padding: 1rem 1.25rem;
  border-radius: var(--radius);
  margin-bottom: 1.75rem;
  font-size: 0.85rem;
  border-left: 2px solid;
}

.banner-error {
  background: var(--red-dim);
  border-color: var(--red);
  color: var(--red);
}

.banner-success {
  background: var(--green-dim);
  border-color: var(--green);
  color: var(--green);
}

.banner-icon {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  font-weight: 500;
  flex-shrink: 0;
  margin-top: 0.15rem;
  letter-spacing: 0;
}

.banner ul { padding-left: 1rem; margin-top: 0.3rem; }
.banner li { margin-bottom: 0.18rem; opacity: 0.85; font-family: var(--font-mono); font-size: 0.78rem; }
.banner strong { font-weight: 600; font-family: var(--font-head); letter-spacing: 0.04em; }

/* ── Submit button ── */
.btn-row { margin-top: 2rem; }

button[type="submit"] {
  padding: 0.8rem 2rem;
  background: var(--lime);
  color: #0d0d0f;
  border: none;
  border-radius: var(--radius);
  font-family: var(--font-head);
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  cursor: pointer;
  transition: opacity 0.15s, transform 0.1s, box-shadow 0.2s;
  box-shadow: 0 0 24px var(--lime-dim);
}

button[type="submit"]:hover {
  opacity: 0.88;
  box-shadow: 0 0 40px rgba(184,245,90,0.2);
  transform: translateY(-1px);
}

button[type="submit"]:active {
  transform: translateY(0);
  opacity: 1;
}

/* ── Records table ── */
.table-wrap { overflow-x: auto; }

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

thead {
  border-bottom: 1px solid var(--line-bright);
}

thead th {
  padding: 0.9rem 1.25rem;
  text-align: left;
  font-family: var(--font-mono);
  font-size: 0.58rem;
  font-weight: 500;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: #9e9cb0;
  white-space: nowrap;
  background: var(--bg-raised);
}

thead th:first-child { padding-left: 2rem; }
thead th:last-child  { padding-right: 2rem; }

tbody tr {
  border-bottom: 1px solid var(--line);
  transition: background 0.12s;
}

tbody tr:hover { background: rgba(255,255,255,0.02); }
tbody tr:last-child { border-bottom: none; }

tbody td {
  padding: 0.9rem 1.25rem;
  color: var(--fg);
  vertical-align: middle;
  font-weight: 300;
}

tbody td:first-child { padding-left: 2rem; }
tbody td:last-child  { padding-right: 2rem; }

.td-mono {
  font-family: var(--font-mono);
  font-size: 0.78rem;
  color: #c8c6d8;
}

.td-id {
  font-family: var(--font-mono);
  font-size: 0.62rem;
  color: #9e9cb0;
}

.td-name {
  font-family: var(--font-head);
  font-weight: 600;
  font-size: 0.875rem;
  letter-spacing: 0.01em;
}

.empty-state {
  text-align: center;
  padding: 4.5rem 1rem;
  color: var(--fg-dim);
  font-family: var(--font-mono);
  font-size: 0.68rem;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.count-badge {
  font-family: var(--font-mono);
  font-size: 0.58rem;
  background: var(--lime-dim);
  color: var(--lime);
  border: 1px solid rgba(184,245,90,0.25);
  padding: 0.1rem 0.5rem;
  border-radius: 2px;
  margin-left: 0.6rem;
  vertical-align: middle;
  letter-spacing: 0.08em;
}

/* ── Footer ── */
footer {
  max-width: 860px;
  margin: 4rem auto 0;
  padding-top: 1.5rem;
  border-top: 1px solid var(--line);
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-family: var(--font-mono);
  font-size: 0.58rem;
  letter-spacing: 0.12em;
  color: var(--fg-dim);
  text-transform: uppercase;
}

/* ── Toast ── */
.toast {
  position: fixed;
  bottom: 2.5rem;
  left: 50%;
  transform: translateX(-50%) translateY(0);
  background: var(--lime);
  color: #0d0d0f;
  padding: 0.7rem 1.6rem;
  border-radius: 2px;
  font-family: var(--font-mono);
  font-size: 0.72rem;
  font-weight: 500;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  white-space: nowrap;
  box-shadow: 0 0 40px rgba(184,245,90,0.25);
  animation: toast-in 0.35s cubic-bezier(0.34,1.56,0.64,1) forwards,
             toast-out 0.35s ease 2.8s forwards;
  pointer-events: none;
  z-index: 9999;
}

@keyframes toast-in {
  from { opacity: 0; transform: translateX(-50%) translateY(1.5rem); }
  to   { opacity: 1; transform: translateX(-50%) translateY(0); }
}

@keyframes toast-out {
  from { opacity: 1; transform: translateX(-50%) translateY(0); }
  to   { opacity: 0; transform: translateX(-50%) translateY(1.5rem); }
}

/* ── Responsive ── */
@media (max-width: 600px) {
  body { padding: 0 1.25rem 5rem; }
  .field-row { grid-template-columns: 1fr; }
  .form-inner { padding: 1.5rem; }
  header { grid-template-columns: 1fr; gap: 1.25rem; padding: 2.5rem 0 2rem; }
  .header-meta { text-align: left; }
  header h1 { font-size: clamp(2.4rem, 10vw, 3.5rem); }
}
"#;

// ─────────────────────────────────────────────
//  Database
// ─────────────────────────────────────────────

fn open_db(path: &str) -> SqlResult<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         CREATE TABLE IF NOT EXISTS records (
             id        INTEGER PRIMARY KEY AUTOINCREMENT,
             first_name TEXT NOT NULL,
             last_name  TEXT NOT NULL,
             phone      TEXT NOT NULL,
             address    TEXT NOT NULL,
             age        INTEGER NOT NULL,
             created_at TEXT NOT NULL DEFAULT (datetime('now'))
         );",
    )?;
    Ok(conn)
}

#[derive(Debug)]
struct Record {
    id: i64,
    first_name: String,
    last_name: String,
    phone: String,
    address: String,
    age: i64,
    created_at: String,
}

fn insert_record(conn: &Connection, r: &FormData) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO records (first_name, last_name, phone, address, age) VALUES (?1,?2,?3,?4,?5)",
        params![r.first_name, r.last_name, r.phone, r.address, r.age],
    )?;
    Ok(())
}

fn fetch_records(conn: &Connection) -> SqlResult<Vec<Record>> {
    let mut stmt = conn.prepare(
        "SELECT id, first_name, last_name, phone, address, age, created_at FROM records ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Record {
            id: row.get(0)?,
            first_name: row.get(1)?,
            last_name: row.get(2)?,
            phone: row.get(3)?,
            address: row.get(4)?,
            age: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    rows.collect()
}

// ─────────────────────────────────────────────
//  Form data & validation
// ─────────────────────────────────────────────

#[derive(Default, Debug)]
struct FormData {
    first_name: String,
    last_name: String,
    phone: String,
    address: String,
    age: i64,
    age_raw: String,
}

// Letters (including all Unicode / Latvian accented chars), hyphens, apostrophes.
fn is_valid_name_char(c: char) -> bool {
    c.is_alphabetic() || c == '-' || c == '\''
}

fn validate(data: &FormData) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    // ── First Name ──────────────────────────────────────────────────────────
    let fn_trimmed = data.first_name.trim();
    if fn_trimmed.is_empty() {
        errors.push("First Name is required.".into());
    } else if fn_trimmed.chars().count() < 2 {
        errors.push("First Name must be at least 2 characters.".into());
    } else if fn_trimmed.chars().count() > 50 {
        errors.push("First Name must not exceed 50 characters.".into());
    } else if !fn_trimmed.chars().all(is_valid_name_char) {
        errors.push("First Name must contain only letters, hyphens, or apostrophes (no digits or symbols).".into());
    }

    // ── Last Name ───────────────────────────────────────────────────────────
    let ln_trimmed = data.last_name.trim();
    if ln_trimmed.is_empty() {
        errors.push("Last Name is required.".into());
    } else if ln_trimmed.chars().count() < 2 {
        errors.push("Last Name must be at least 2 characters.".into());
    } else if ln_trimmed.chars().count() > 50 {
        errors.push("Last Name must not exceed 50 characters.".into());
    } else if !ln_trimmed.chars().all(is_valid_name_char) {
        errors.push("Last Name must contain only letters, hyphens, or apostrophes (no digits or symbols).".into());
    }

    // ── Phone ───────────────────────────────────────────────────────────────
    let phone = data.phone.trim();
    if phone.is_empty() {
        errors.push("Phone Number is required.".into());
    } else if !phone.chars().all(|c| c.is_ascii_digit()) {
        errors.push("Phone Number must contain only numeric digits (no spaces, dashes, or symbols).".into());
    } else if phone.len() != 8 {
        errors.push(format!(
            "Phone Number must be exactly 8 digits (got {}).",
            phone.len()
        ));
    }

    // ── Address ─────────────────────────────────────────────────────────────
    let addr = data.address.trim();
    if addr.is_empty() {
        errors.push("Address is required.".into());
    } else if addr.chars().count() < 5 {
        errors.push("Address must be at least 5 characters.".into());
    } else if addr.chars().count() > 200 {
        errors.push("Address must not exceed 200 characters.".into());
    } else {
        // A real address must contain both a street name (letter) and a number (digit).
        if !addr.chars().any(|c| c.is_alphabetic()) {
            errors.push("Address must include a street name (at least one letter).".into());
        }
        if !addr.chars().any(|c| c.is_ascii_digit()) {
            errors.push("Address must include a street number (at least one digit).".into());
        }
    }

    // ── Age ─────────────────────────────────────────────────────────────────
    let age_trimmed = data.age_raw.trim();
    if age_trimmed.is_empty() {
        errors.push("Age is required.".into());
    } else if !age_trimmed.chars().all(|c| c.is_ascii_digit()) {
        errors.push("Age must be a whole number with no letters or symbols.".into());
    } else if data.age < 1 || data.age > 150 {
        errors.push(format!(
            "Age must be between 1 and 150 (got {}).",
            data.age
        ));
    }

    errors
}

fn parse_form(body: &str) -> FormData {
    let mut map: HashMap<String, String> = HashMap::new();
    for part in body.split('&') {
        let mut kv = part.splitn(2, '=');
        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
            let key = urlencoding::decode(k).unwrap_or_default().replace('+', " ").to_string();
            let val = urlencoding::decode(v).unwrap_or_default().replace('+', " ").to_string();
            map.insert(key, val);
        }
    }
    let age_raw = map.get("age").cloned().unwrap_or_default();
    let age: i64 = age_raw.trim().parse().unwrap_or(0);
    FormData {
        first_name: map.get("first_name").cloned().unwrap_or_default(),
        last_name:  map.get("last_name").cloned().unwrap_or_default(),
        phone:      map.get("phone").cloned().unwrap_or_default(),
        address:    map.get("address").cloned().unwrap_or_default(),
        age,
        age_raw,
    }
}

// ─────────────────────────────────────────────
//  HTML rendering
// ─────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        return String::new();
    }
    let items: String = errors
        .iter()
        .map(|e| format!("<li>{}</li>", html_escape(e)))
        .collect();
    format!(
        r#"<div class="banner banner-error">
          <span class="banner-icon">ERR</span>
          <div><strong>Validation failed &mdash;</strong><ul>{}</ul></div>
        </div>"#,
        items
    )
}

fn render_success() -> String {
    r#"<div class="banner banner-success">
      <span class="banner-icon">OK</span>
      <div><strong>Entry committed.</strong> The record has been written to the database.</div>
    </div>"#
    .to_string()
}

fn render_form(data: &FormData, errors: &[String], show_success: bool) -> String {
    let error_html   = render_errors(errors);
    let success_html = if show_success { render_success() } else { String::new() };
    let v = |s: &str| html_escape(s);

    format!(
        r#"{error_html}{success_html}
        <form method="POST" action="/" autocomplete="off">
          <div class="field-row">
            <div class="field">
              <label for="first_name">Given Name<span class="required-mark">*</span></label>
              <input type="text" id="first_name" name="first_name" value="{fn}" placeholder="e.g. Jānis">
              <span class="field-hint">2–50 letters, hyphens or apostrophes only</span>
            </div>
            <div class="field">
              <label for="last_name">Family Name<span class="required-mark">*</span></label>
              <input type="text" id="last_name" name="last_name" value="{ln}" placeholder="e.g. Bērziņš">
              <span class="field-hint">2–50 letters, hyphens or apostrophes only</span>
            </div>
          </div>
          <div class="field">
            <label for="phone">Contact Number<span class="required-mark">*</span></label>
            <input type="text" id="phone" name="phone" value="{ph}" placeholder="12345678">
            <span class="field-hint">Exactly 8 numeric digits &mdash; no spaces or symbols</span>
          </div>
          <div class="field">
            <label for="address">Residential Address<span class="required-mark">*</span></label>
            <input type="text" id="address" name="address" value="{addr}" placeholder="e.g. Brīvības iela 19, Rīga">
            <span class="field-hint">5–200 characters, must include a street name and number</span>
          </div>
          <div class="field">
            <label for="age">Age<span class="required-mark">*</span></label>
            <input type="text" id="age" name="age" value="{age}" placeholder="25">
            <span class="field-hint">Whole number between 1 and 150</span>
          </div>
          <div class="btn-row">
            <button type="submit">Commit Entry &rarr;</button>
          </div>
        </form>"#,
        fn   = v(&data.first_name),
        ln   = v(&data.last_name),
        ph   = v(&data.phone),
        addr = v(&data.address),
        age  = if data.age_raw.is_empty() { String::new() } else { v(&data.age_raw) },
    )
}

fn render_table(records: &[Record]) -> String {
    if records.is_empty() {
        return r#"<div class="empty-state">No entries yet &mdash; submit the form above</div>"#.to_string();
    }

    let rows: String = records.iter().map(|r| {
        format!(
            r#"<tr>
              <td class="td-id">{id}</td>
              <td class="td-name">{fn} {ln}</td>
              <td class="td-mono">{ph}</td>
              <td>{addr}</td>
              <td class="td-mono">{age}</td>
              <td class="td-mono">{ts}</td>
            </tr>"#,
            id   = r.id,
            fn   = html_escape(&r.first_name),
            ln   = html_escape(&r.last_name),
            ph   = html_escape(&r.phone),
            addr = html_escape(&r.address),
            age  = r.age,
            ts   = &r.created_at[..16],
        )
    }).collect();

    format!(
        r#"<div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>Full Name</th>
                <th>Contact</th>
                <th>Address</th>
                <th>Age</th>
                <th>Recorded At</th>
              </tr>
            </thead>
            <tbody>{}</tbody>
          </table>
        </div>"#,
        rows
    )
}

fn full_page(form_html: &str, table_html: &str, count: usize, show_toast: bool) -> String {
    let badge = if count > 0 {
        format!("<span class=\"count-badge\">{}</span>", count)
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Personal Records Registry</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <style>{css}</style>
</head>
<body>
  {toast}
  <div class="topbar">
    <span class="topbar-id">PRSN-REG &nbsp;/&nbsp; v1.0</span>
    <span class="topbar-dot"></span>
  </div>
  <header>
    <h1><span>Contact</span>Registry</h1>
    <div class="header-meta">
      <div>Personal Data</div>
      <div>Management System</div>
    </div>
  </header>
  <main>
    <div class="section-head">
      <span class="section-num">01</span>
      <span class="section-title">New Entry</span>
      <div class="section-rule"></div>
    </div>
    <div class="card">
      <div class="form-inner">
        {form}
      </div>
    </div>

    <div class="section-head">
      <span class="section-num">02</span>
      <span class="section-title">All Entries{badge}</span>
      <div class="section-rule"></div>
    </div>
    <div class="card">
      {table}
    </div>
  </main>
  <footer></footer>
</body>
</html>"#,
        css   = CSS,
        form  = form_html,
        table = table_html,
        badge = badge,
        toast = if show_toast {
            r#"<div class="toast">Entry committed</div>"#
        } else {
            ""
        },
    )
}

// ─────────────────────────────────────────────
//  HTTP layer (raw TCP)
// ─────────────────────────────────────────────

fn read_request(stream: &mut TcpStream) -> Option<(String, String, String)> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).ok()?;
    let raw = String::from_utf8_lossy(&buf[..n]).to_string();

    let mut lines = raw.splitn(2, "\r\n");
    let request_line = lines.next()?.to_string();
    let rest = lines.next().unwrap_or("");

    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path   = parts.next()?.to_string();

    let body = if let Some(pos) = rest.find("\r\n\r\n") {
        rest[pos + 4..].to_string()
    } else {
        String::new()
    };

    Some((method, path, body))
}

fn send_response(stream: &mut TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn send_redirect(stream: &mut TcpStream, location: &str) {
    let response = format!(
        "HTTP/1.1 303 See Other\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        location
    );
    let _ = stream.write_all(response.as_bytes());
}

// ─────────────────────────────────────────────
//  Request handler
// ─────────────────────────────────────────────

fn handle(stream: &mut TcpStream, db: &Arc<Mutex<Connection>>) {
    let Some((method, path, body)) = read_request(stream) else { return };

    let (route, query) = match path.find('?') {
        Some(pos) => (&path[..pos], &path[pos + 1..]),
        None      => (path.as_str(), ""),
    };

    if route != "/" {
        send_response(stream, "404 Not Found", "<h1>404</h1>");
        return;
    }

    let show_toast = query.contains("ok=1");
    let conn = db.lock().unwrap();

    match method.as_str() {
        "GET" => {
            let records    = fetch_records(&conn).unwrap_or_default();
            let count      = records.len();
            let form_html  = render_form(&FormData::default(), &[], false);
            let table_html = render_table(&records);
            let page = full_page(&form_html, &table_html, count, show_toast);
            send_response(stream, "200 OK", &page);
        }

        "POST" => {
            let data   = parse_form(&body);
            let errors = validate(&data);

            if errors.is_empty() {
                if let Err(e) = insert_record(&conn, &data) {
                    eprintln!("DB error: {e}");
                }
                send_redirect(stream, "/?ok=1");
            } else {
                let records    = fetch_records(&conn).unwrap_or_default();
                let count      = records.len();
                let form_html  = render_form(&data, &errors, false);
                let table_html = render_table(&records);
                let page = full_page(&form_html, &table_html, count, show_toast);
                send_response(stream, "400 Bad Request", &page);
            }
        }

        _ => send_response(stream, "405 Method Not Allowed", ""),
    }
}

// ─────────────────────────────────────────────
//  Main
// ─────────────────────────────────────────────

fn main() {
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "/data/records.db".to_string());
    let port    = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr    = format!("0.0.0.0:{}", port);

    let conn = open_db(&db_path).expect("Failed to open database");
    let db   = Arc::new(Mutex::new(conn));

    let listener = TcpListener::bind(&addr).expect("Failed to bind");
    println!("► Listening on http://{addr}");
    println!("► DB at {db_path}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let db = Arc::clone(&db);
                std::thread::spawn(move || handle(&mut s, &db));
            }
            Err(e) => eprintln!("Connection error: {e}"),
        }
    }
}