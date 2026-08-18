# chaintail

**English** · [中文](README.zh-CN.md) — plan: [docs/PROJECT-PLAN.md](docs/PROJECT-PLAN.md)

Local-first, **read-only** chain event tail.

Point it at a program id, contract, or address list. It follows logs / account changes into SQLite on your machine. You can `tail`, query, and alert. No validator, no keys, no broadcast.

Think `tail -f` for a chain. First implementation may be one chain only (Solana *or* one EVM).

## Status

Scaffold. Spec is in `docs/`. No runtime yet.

## v0.1 commands (target)

```text
chaintail init
chaintail follow
chaintail query
chaintail alert
```

## What we will not do

- Run a full node or “open-source Helius”
- Private keys, signing, broadcast
- Historical full-chain index in v0.1
- Ten-chain abstraction on day one

## License

MIT.

## Security

Read [`SECURITY.md`](SECURITY.md).
