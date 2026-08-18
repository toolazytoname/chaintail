# chaintail

**English** · [中文](README.zh-CN.md) — plan: [docs/PROJECT-PLAN.md](docs/PROJECT-PLAN.md)

Local-first, **read-only** chain event tail.

Point it at a program id, contract, or address list. It follows logs / account changes into SQLite on your machine. You can `tail`, query, and alert. No validator, no keys, no broadcast.

Think `tail -f` for a chain. First implementation may be one chain only (Solana *or* one EVM).

## Status

**v0.1 runtime (Rust 1.85 + bundled rusqlite).** One chain (`evm-fixture`): canned logs into local SQLite, query, alert. Amounts are integer strings.

```bash
cd chaintail
cargo test
cargo run -- follow --config fixtures/config.ok.json --fixture fixtures/events.json --db /tmp/chaintail-demo.sqlite
cargo run -- query --config fixtures/config.ok.json --db /tmp/chaintail-demo.sqlite --fail
cargo run -- alert --config fixtures/config.ok.json --db /tmp/chaintail-demo.sqlite --min-amount 2000000
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
