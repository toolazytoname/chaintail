# chaintail — 项目说明

> 推荐顺序第 3。底座手艺。hlsentry / oddsradar 的底层都是「把链上变化接到本机」。  
> 来源：`web3_explore/doc/next/WEB3-DIRECTIONS.md`

## 概述

给一个 program / 合约 / 地址，在本机把事件接下来，进 SQLite，能 tail、能查、能告警。不跑验证节点，不碰私钥。

第一版只选 **一条链**：Solana（更接近 grant 叙事）或一条 EVM（资料最多、AI 更熟）。选之前先定你自己要盯什么。

## 你自己怎么用

开发或盯一个真实合约时当显微镜。不是每天看仓的 App，是手艺课。

## 一开始

```text
chaintail init
chaintail follow
chaintail query
chaintail alert
```

一种数据源，一种告警。测试不依赖主网也能跑（fixture）。

## 后面

给另外两个哨兵当引擎；README 5 分钟能跑再申一次小额 grant。  
Superteam 微额常看地区，大陆资格别默认有。更稳：Solana Foundation / Gitcoin / Base 追溯（先有能跑的东西）。

## 上限

开源工具 + 小额资助 + 按件改一刀。成不了 Alchemy。不要写成「下一代索引器」。

## 挣钱

Grant 比订阅现实。有人要加链，按件收。

## 本周先做（有空再开）

1. 拍板第一链：Solana 或 Base/Arbitrum 二选一
2. 指定一个 **devnet / 测试网** 上的稳定 program 或合约当演示对象
3. 空壳 `--help` + 假数据 `follow` 能打出 JSON
