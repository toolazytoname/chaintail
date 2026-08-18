use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  chain TEXT NOT NULL,
  tx TEXT NOT NULL,
  log_index INTEGER NOT NULL,
  block INTEGER NOT NULL,
  address TEXT NOT NULL,
  kind TEXT NOT NULL,
  amount_raw TEXT NOT NULL,
  ok INTEGER NOT NULL,
  raw TEXT NOT NULL,
  UNIQUE(chain, tx, log_index)
);
"#;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventRow {
    pub chain: String,
    pub tx: String,
    pub log_index: i64,
    pub block: i64,
    pub address: String,
    pub kind: String,
    pub amount_raw: String,
    #[serde(default = "default_ok")]
    pub ok: bool,
}

fn default_ok() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct EventsFile {
    pub events: Vec<EventRow>,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("amount_raw must be integer string, got {0}")]
    Amount(String),
}

pub fn open(path: &Path) -> Result<Connection, StoreError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

pub fn ingest(conn: &Connection, rows: &[EventRow]) -> Result<usize, StoreError> {
    let mut n = 0;
    for row in rows {
        let raw = serde_json::to_string(row)?;
        let changed = conn.execute(
            "INSERT OR IGNORE INTO events(chain, tx, log_index, block, address, kind, amount_raw, ok, raw)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                row.chain,
                row.tx,
                row.log_index,
                row.block,
                row.address,
                row.kind,
                row.amount_raw,
                if row.ok { 1 } else { 0 },
                raw,
            ],
        )?;
        n += changed;
    }
    Ok(n)
}

pub fn parse_amount(text: &str) -> Result<i64, StoreError> {
    let s = text.trim();
    s.parse::<i64>().map_err(|_| StoreError::Amount(text.to_string()))
}

pub fn query(
    conn: &Connection,
    fail_only: bool,
    min_amount: Option<i64>,
) -> Result<Vec<EventRow>, StoreError> {
    let sql = if fail_only {
        "SELECT chain, tx, log_index, block, address, kind, amount_raw, ok FROM events WHERE ok = 0"
    } else {
        "SELECT chain, tx, log_index, block, address, kind, amount_raw, ok FROM events"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |r| {
        Ok(EventRow {
            chain: r.get(0)?,
            tx: r.get(1)?,
            log_index: r.get(2)?,
            block: r.get(3)?,
            address: r.get(4)?,
            kind: r.get(5)?,
            amount_raw: r.get(6)?,
            ok: r.get::<_, i64>(7)? != 0,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        let row = row?;
        if let Some(min) = min_amount {
            if parse_amount(&row.amount_raw)? < min {
                continue;
            }
        }
        out.push(row);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_idempotent_and_filters() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open(&dir.path().join("t.sqlite")).unwrap();
        let file: EventsFile = serde_json::from_str(
            &std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/events.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(ingest(&conn, &file.events).unwrap(), 3);
        assert_eq!(ingest(&conn, &file.events).unwrap(), 0);
        assert_eq!(query(&conn, false, None).unwrap().len(), 3);
        let fails = query(&conn, true, None).unwrap();
        assert_eq!(fails.len(), 1);
        assert_eq!(fails[0].tx, "0xccc");
        let big = query(&conn, false, Some(2_000_000)).unwrap();
        assert_eq!(big.iter().map(|r| r.tx.as_str()).collect::<Vec<_>>(), ["0xbbb"]);
    }
}
