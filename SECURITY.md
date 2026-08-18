# Security

chaintail is a **read-only** local tail. It must never see, store, or log private keys, mnemonics, or signing material.

## Rules

- Config may contain public addresses, program ids, contract addresses, RPC URLs, and notify tokens.
- Forbidden in config, env, logs, and the SQLite db: `private_key`, `privkey`, `mnemonic`, `seed`, `wif`, `secret_key`.
- The local address list stays on disk. v0.1 does not upload it.
- No transaction broadcast.
- Amounts are integer smallest units. Floating-point money math is a defect.

## Reporting

Open a private GitHub security advisory, or contact the maintainer on the GitHub profile.
