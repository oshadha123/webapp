# Contact Registry - Rust Web App

A personal data registry with server-side-only validation, built in **pure Rust** with **no web framework**, backed by **SQLite**, containerised with **Docker**.

---

## Quick Start

```bash
# Clone / extract the archive, then:
docker compose up --build -d

# Open in your browser
open http://localhost:8080
```

The SQLite database is stored in a named Docker volume (`db_data`) and survives container restarts.

---

## Architecture

```
Browser (TCP)
    │  raw HTTP/1.1
TcpListener (std::net)          <- no framework, no Hyper, no Axum
    │
    ├── parse_request()          <- manual header / body split
    ├── parse_form()             <- urlencoding crate (form body decode)
    ├── validate()               <- all rules in Rust, zero JS
    │
    ├── [POST valid]  insert_record() -> SQLite (rusqlite, WAL)
    │                 -> 303 Redirect (PRG pattern)
    │
    └── [GET / error] render HTML string -> write to TcpStream
```

### Files

```
webapp/
├── Cargo.toml          # 2 dependencies: rusqlite (bundled), urlencoding
├── Dockerfile          # Multi-stage: rust:slim → debian:slim (~12 MB image)
├── docker-compose.yml  # Port 8080, named volume db_data
└── src/
    └── main.rs         # ~780 lines, entire application
```

---

## Validation Rules (server-side only)

| Field        | Rule                                                                 |
|--------------|----------------------------------------------------------------------|
| Given Name   | Required · 2–50 characters · letters, hyphens, apostrophes only     |
| Family Name  | Required · 2–50 characters · letters, hyphens, apostrophes only     |
| Phone Number | Required · digits only · exactly 8 digits                           |
| Address      | Required · 5–200 characters                                         |
| Age          | Required · integer · 1–150                                          |

No `<script>` tags exist in the output HTML. No `required`, `pattern`, or `min`/`max` attributes are set on inputs. Validation is enforced exclusively in Rust. All user input is HTML-escaped before rendering to prevent XSS.

---

## Dependencies

| Crate       | Version | Purpose                                                              |
|-------------|---------|----------------------------------------------------------------------|
| rusqlite    | 0.31    | SQLite bindings (bundled feature compiles libsqlite3 into the binary)|
| urlencoding | 2.1     | Percent-decode form body strings                                     |

---

## Environment Variables

| Variable  | Default             | Description          |
|-----------|---------------------|----------------------|
| `PORT`    | `8080`              | TCP listen port      |
| `DB_PATH` | `/data/records.db`  | SQLite database file |
