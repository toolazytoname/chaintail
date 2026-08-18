from __future__ import annotations

FORBIDDEN_SUBSTR = ("private_key", "privkey", "mnemonic", "seed", "wif", "secret_key")


def forbidden_fields(obj) -> list[str]:
    hits = []

    def walk(o, prefix=""):
        if isinstance(o, dict):
            for k, v in o.items():
                path = f"{prefix}.{k}" if prefix else str(k)
                n = "".join(ch for ch in str(k).lower() if ch.isalnum() or ch == "_")
                if any(s in n for s in FORBIDDEN_SUBSTR):
                    hits.append(path)
                walk(v, path)

    walk(obj)
    return hits
