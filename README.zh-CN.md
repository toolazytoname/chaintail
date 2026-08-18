# chaintail

[English](README.md) · **中文** — 计划见 [docs/PROJECT-PLAN.md](docs/PROJECT-PLAN.md)

本地优先、**只读**的链上事件尾巴。

给一个 program / 合约 / 地址列表，把日志或账户变化接到本机 SQLite。能 `tail`、能查、能告警。不跑验证节点，不碰私钥，不广播。

像给区块链装一个本机版 `tail -f`。第一版只做一条链（Solana **或** 一条 EVM）。

## 状态

**v0.1 可运行。** fixture 日志进 SQLite，可 query / alert。

## 明确不做

- 自建全节点，或写成「开源 Helius」
- 私钥、签名、广播
- 第一版就做全链历史索引
- 第一天抽象十条链

后续工作在这个文件夹里展开。先读 `docs/PROJECT-PLAN.md`。
