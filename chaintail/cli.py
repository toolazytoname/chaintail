from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from chaintail.secrets import forbidden_fields
from chaintail.store import connect, ingest, query


def _safe_cfg(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    hits = forbidden_fields(data)
    if hits:
        print(f"doctor: forbidden secret field(s): {', '.join(hits)}", file=sys.stderr)
        raise SystemExit(2)
    return data


def cmd_init(args) -> int:
    d = Path(args.dir)
    d.mkdir(parents=True, exist_ok=True)
    cfg = d / "config.json"
    if not cfg.exists():
        cfg.write_text(
            json.dumps(
                {"chain": "evm-fixture", "db": "chaintail.sqlite", "notify": {"kind": "file", "path": "alerts.jsonl"}},
                indent=2,
            )
            + "\n"
        )
    print(f"wrote {cfg}")
    return 0


def cmd_doctor(args) -> int:
    cfg = _safe_cfg(Path(args.config))
    print(f"ok chain={cfg.get('chain')} db={cfg.get('db')}")
    return 0


def cmd_follow(args) -> int:
    cfg = _safe_cfg(Path(args.config))
    db = Path(args.db or cfg.get("db") or "chaintail.sqlite")
    payload = json.loads(Path(args.fixture).read_text(encoding="utf-8"))
    conn = connect(db)
    n = ingest(conn, payload["events"])
    print(json.dumps({"ingested": n, "db": str(db)}))
    conn.close()
    return 0


def cmd_query(args) -> int:
    cfg = _safe_cfg(Path(args.config))
    db = Path(args.db or cfg.get("db") or "chaintail.sqlite")
    conn = connect(db)
    rows = query(conn, fail_only=args.fail, min_amount=args.min_amount)
    for r in rows:
        print(json.dumps({k: r[k] for k in ("chain", "tx", "log_index", "kind", "amount_raw", "ok")}, sort_keys=True))
    conn.close()
    return 0


def cmd_alert(args) -> int:
    cfg = _safe_cfg(Path(args.config))
    db = Path(args.db or cfg.get("db") or "chaintail.sqlite")
    conn = connect(db)
    rows = query(conn, fail_only=args.fail, min_amount=args.min_amount)
    dest = Path(args.notify_file or (cfg.get("notify") or {}).get("path") or "alerts.jsonl")
    with dest.open("a", encoding="utf-8") as fh:
        for r in rows:
            ev = {
                "kind": "fail" if r["ok"] == 0 else "amount",
                "tx": r["tx"],
                "amount_raw": r["amount_raw"],
                "ok": r["ok"],
            }
            fh.write(json.dumps(ev, sort_keys=True) + "\n")
            print(json.dumps(ev, sort_keys=True))
    conn.close()
    print(f"notify file:{dest}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="chaintail")
    sub = p.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("init")
    s.add_argument("--dir", default=".")
    s.set_defaults(func=cmd_init)
    s = sub.add_parser("doctor")
    s.add_argument("--config", required=True)
    s.set_defaults(func=cmd_doctor)
    s = sub.add_parser("follow")
    s.add_argument("--config", required=True)
    s.add_argument("--fixture", required=True)
    s.add_argument("--db", default=None)
    s.set_defaults(func=cmd_follow)
    s = sub.add_parser("query")
    s.add_argument("--config", required=True)
    s.add_argument("--db", default=None)
    s.add_argument("--fail", action="store_true")
    s.add_argument("--min-amount", type=int, default=None)
    s.set_defaults(func=cmd_query)
    s = sub.add_parser("alert")
    s.add_argument("--config", required=True)
    s.add_argument("--db", default=None)
    s.add_argument("--fail", action="store_true")
    s.add_argument("--min-amount", type=int, default=None)
    s.add_argument("--notify-file", default=None)
    s.set_defaults(func=cmd_alert)
    return p


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    return args.func(args)
