from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any, Iterable


SCHEMA = """
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
"""


def connect(path: Path) -> sqlite3.Connection:
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(path))
    conn.row_factory = sqlite3.Row
    conn.execute(SCHEMA)
    return conn


def ingest(conn: sqlite3.Connection, rows: Iterable[dict[str, Any]]) -> int:
    n = 0
    for row in rows:
        cur = conn.execute(
            """
            INSERT OR IGNORE INTO events(chain, tx, log_index, block, address, kind, amount_raw, ok, raw)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                row["chain"],
                row["tx"],
                int(row["log_index"]),
                int(row["block"]),
                row["address"],
                row["kind"],
                str(row["amount_raw"]),
                1 if row.get("ok", True) else 0,
                json.dumps(row, sort_keys=True),
            ),
        )
        n += cur.rowcount
    conn.commit()
    return n


def query(
    conn: sqlite3.Connection,
    *,
    fail_only: bool = False,
    min_amount: int | None = None,
) -> list[dict]:
    sql = "SELECT * FROM events WHERE 1=1"
    args: list = []
    if fail_only:
        sql += " AND ok = 0"
    rows = [dict(r) for r in conn.execute(sql, args).fetchall()]
    if min_amount is not None:
        rows = [r for r in rows if _amount_int(r["amount_raw"]) >= min_amount]
    return rows


def _amount_int(text: str) -> int:
    s = str(text).strip()
    if not s or not s.lstrip("-").isdigit():
        raise ValueError(f"amount_raw must be integer string, got {text!r}")
    return int(s)
