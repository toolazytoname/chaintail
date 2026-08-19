<p align="center">
  <img src="learn/assets/cover.jpg" alt="chaintail：链上事件流进本机账本" width="880">
</p>

<h1 align="center">chaintail</h1>

<p align="center">
  <strong>本地优先、只读的链上事件尾巴。</strong><br>
  给一条合约做本机版 <code>tail -f</code>，结果进 SQLite。
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="README.zh-CN.md"><strong>中文</strong></a> ·
  <a href="learn/README.md">学习</a> ·
  <a href="docs/PROJECT-PLAN.md">计划</a> ·
  <a href="SECURITY.md">安全</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-1F6FEB" alt="version 0.1.0">
  <img src="https://img.shields.io/badge/rust-1.85-DEA584" alt="Rust 1.85">
  <img src="https://img.shields.io/badge/license-MIT-0B6E4F" alt="MIT license">
  <img src="https://img.shields.io/badge/mode-read--only-111827" alt="只读">
</p>

---

给一个合约地址，把 `eth_getLogs` 接到本机 SQLite。能 `query`、能 `alert`。不跑验证节点，不碰私钥，不广播。

> 像给区块链装一个本机 `tail -f`。v0.1 只做一条 EVM（fixture 或公共 RPC）。这不是索引公司。

## 为什么做这个

区块浏览器很好用，直到你想把**自己的**观察名单放在**自己的**磁盘上，并且能离线查询。全链索引是另一种产品：运维、回填、账单。chaintail 只 index 你点名的地址。

以太坊系里，ERC-20 `Transfer` 的 topic0 是：

```text
keccak256("Transfer(address,address,uint256)")
= 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
```

这是全网公开的函数签名哈希，不是密钥。金额在 `data` 里是 32 字节大端整数，入库仍是整数字符串。

## 能力

| | |
|---|---|
| **本机 SQLite** | 捆绑 `rusqlite`。地址列表不必离开磁盘。 |
| **幂等入库** | `UNIQUE (chain, tx, log_index)` + `INSERT OR IGNORE`。 |
| **游标** | 第一次用 `--lookback`；之后从 `last_block + 1` 接着扫。 |
| **整数金额** | 十六进制 `data` → 十进制字符串。不用 `parseFloat`。 |
| **fixture 与 RPC 在 ingest 汇合** | 录下来的 `eth_getLogs` JSON 就能测解码，不必烧节点额度。 |

## 怎么工作

<p align="center">
  <img src="learn/assets/architecture.svg" alt="chaintail 架构：fixture 或 eth_getLogs 进入 SQLite，再 query / alert" width="880">
</p>

用到的 JSON-RPC：`eth_blockNumber`、`eth_getLogs`。永远不调用 `eth_sendRawTransaction`。

## 环境

- [Rust](https://www.rust-lang.org/tools/install) **1.85**
- 真跟链需要一个公共 EVM RPC（fixture 路径不需要）

```bash
git clone https://github.com/toolazytoname/chaintail.git
cd chaintail
cargo test
```

## 快速开始

**Fixture（离线）：**

```bash
cargo run -- follow \
  --config fixtures/config.ok.json \
  --fixture fixtures/events.json \
  --db /tmp/ct.sqlite

cargo run -- query \
  --config fixtures/config.ok.json \
  --db /tmp/ct.sqlite \
  --fail
```

`--fail` 只留下 `ok=false` 的行（fixture 里那笔 `0xccc`）。

**Live（Base 公共 RPC + USDC，只读）：**

```bash
cargo run -- follow \
  --config fixtures/config.ok.json \
  --rpc --lookback 50 \
  --db /tmp/ct.sqlite

cargo run -- query \
  --config fixtures/config.ok.json \
  --db /tmp/ct.sqlite
```

连续跑两次 `--rpc --lookback 3`，第二次的 `rows` 应远小于第一次：游标已经往前走了。

## 配置

`fixtures/config.ok.json`（可以复制，里面没有密钥）：

```json
{
  "chain": "base",
  "db": "chaintail.sqlite",
  "rpc_url": "https://mainnet.base.org",
  "address": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
  "notify": { "kind": "file", "path": "alerts.jsonl" }
}
```

示例 `address` 是 Base 上的原生 USDC。换成别的 ERC-20 也可以——大多数 `Transfer` 的 topic0 相同。

## 命令

| 命令 | 作用 |
|---|---|
| `init --dir .` | 写一份起始 `config.json`。 |
| `doctor --config FILE` | 拒绝密钥字段名。 |
| `follow --config FILE --fixture FILE --db PATH` | 摄入罐头日志。 |
| `follow --config FILE --rpc [--lookback N] --db PATH` | 从 `rpc_url` 拉 `eth_getLogs`。 |
| `query --config FILE --db PATH [--fail] [--min-amount N]` | 打印匹配行。 |
| `alert --config FILE --db PATH [--notify-file PATH]` | 把匹配写成 JSONL。 |

省略 `--fixture` 时也走 live RPC（和 `--rpc` 一样），前提是配置了 `rpc_url` 和 `address`。

## 测试

```bash
cargo test
```

解码钉在 `fixtures/eth_logs.json`。对同一 SQLite 再 ingest 一次，不得重复行。

## 安全

请读 **[`SECURITY.md`](SECURITY.md)**。无私钥、不签名、不广播。金额是整数最小单位；浮点算钱算缺陷。SQLite 文件在你这边——v0.1 不会上传。

## 明确不做

- 自建全节点，或写成「开源 Helius」
- 私钥、签名、广播
- 第一版就做全链历史回填
- 第一天抽象十条链
- 解码任意 ABI（那是下一步「带一份 ABI 文件」，不是先上通用索引器）

## 学习

[`learn/`](learn/) 讲 LOG 和账户模型、topic0，以及为什么 `0xf4240` 在 6 位小数下是 1 USDC。封面动画：[`learn/assets/cover.mp4`](learn/assets/cover.mp4)。

## 相关

- [hlsentry](https://github.com/toolazytoname/hlsentry) — Hyperliquid 清算哨兵
- [oddsradar](https://github.com/toolazytoname/oddsradar) — 预测市场跨所价差雷达
- [slotbench](https://github.com/toolazytoname/slotbench) — Solana RPC 相对到达秒表

## 许可

[MIT](LICENSE) © 2026 toolazytoname
