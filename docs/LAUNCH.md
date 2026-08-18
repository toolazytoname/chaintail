# 自己用（真上线）

`fixtures/config.ok.json` 已指向 Base 公共 RPC + USDC。

```bash
cargo run -- follow --config fixtures/config.ok.json --rpc --lookback 50 --db data.sqlite
cargo run -- query --config fixtures/config.ok.json --db data.sqlite
```

第二次 follow 从上次游标的下一块开始，不会整段 lookback 重扫。
