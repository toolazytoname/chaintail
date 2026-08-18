from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from chaintail.store import connect, ingest, query  # noqa: E402

BIN = [sys.executable, "-m", "chaintail"]


def run(args, cwd=None):
    return subprocess.run(
        BIN + args,
        cwd=cwd or ROOT,
        capture_output=True,
        text=True,
        env={**os.environ, "PYTHONPATH": str(ROOT)},
    )


class TestStore(unittest.TestCase):
    def test_ingest_query_filters(self):
        with tempfile.TemporaryDirectory() as td:
            conn = connect(Path(td) / "t.sqlite")
            events = json.loads((ROOT / "fixtures/events.json").read_text())["events"]
            self.assertEqual(ingest(conn, events), 3)
            self.assertEqual(ingest(conn, events), 0)  # idempotent
            self.assertEqual(len(query(conn)), 3)
            fails = query(conn, fail_only=True)
            self.assertEqual(len(fails), 1)
            self.assertEqual(fails[0]["tx"], "0xccc")
            big = query(conn, min_amount=2_000_000)
            self.assertEqual([r["tx"] for r in big], ["0xbbb"])


class TestCLI(unittest.TestCase):
    def test_follow_query_alert(self):
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cfg = td / "config.json"
            cfg.write_text((ROOT / "fixtures/config.ok.json").read_text())
            db = td / "db.sqlite"
            r = run(
                ["follow", "--config", str(cfg), "--fixture", str(ROOT / "fixtures/events.json"), "--db", str(db)],
                cwd=td,
            )
            self.assertEqual(r.returncode, 0, r.stderr)
            q = run(["query", "--config", str(cfg), "--db", str(db), "--fail"], cwd=td)
            self.assertEqual(q.returncode, 0, q.stderr)
            self.assertIn("0xccc", q.stdout)
            self.assertNotIn("0xaaa", q.stdout)
            a = run(
                ["alert", "--config", str(cfg), "--db", str(db), "--min-amount", "2000000", "--notify-file", str(td / "a.jsonl")],
                cwd=td,
            )
            self.assertEqual(a.returncode, 0, a.stderr)
            self.assertIn("0xbbb", a.stdout)
            self.assertNotIn("0xaaa", a.stdout)

    def test_doctor_secret(self):
        r = run(["doctor", "--config", str(ROOT / "fixtures/config.secret.json")])
        self.assertNotEqual(r.returncode, 0)
        self.assertNotIn("PLANT-SECRET-DO-NOT-LOG", r.stdout + r.stderr)


if __name__ == "__main__":
    unittest.main()
