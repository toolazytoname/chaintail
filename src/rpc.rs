//! EVM JSON-RPC follow. Read-only: eth_blockNumber + eth_getLogs. No broadcast.

use crate::store::EventRow;
use serde_json::{json, Value};
use thiserror::Error;

/// keccak256("Transfer(address,address,uint256)") — ERC-20 event id, not a secret.
pub const TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("rpc http: {0}")]
    Http(String),
    #[error("rpc: {0}")]
    Msg(String),
}

pub struct RpcClient {
    url: String,
}

impl RpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }

    pub fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let body = json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
        let resp = ureq::post(&self.url)
            .set("Content-Type", "application/json")
            .set("User-Agent", "chaintail/0.1")
            .send_json(body)
            .map_err(|e| RpcError::Http(e.to_string()))?;
        let v: Value = resp.into_json().map_err(|e| RpcError::Http(e.to_string()))?;
        if let Some(err) = v.get("error") {
            return Err(RpcError::Msg(err.to_string()));
        }
        v.get("result")
            .cloned()
            .ok_or_else(|| RpcError::Msg("missing result".into()))
    }

    pub fn block_number(&self) -> Result<u64, RpcError> {
        let v = self.call("eth_blockNumber", json!([]))?;
        parse_hex_u64(v.as_str().unwrap_or("0x0"))
    }

    pub fn get_logs(
        &self,
        address: &str,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<EventRow>, RpcError> {
        let v = self.call(
            "eth_getLogs",
            json!([{
                "address": address,
                "fromBlock": format!("0x{from_block:x}"),
                "toBlock": format!("0x{to_block:x}"),
            }]),
        )?;
        parse_logs("evm", address, &v)
    }
}

pub fn parse_hex_u64(s: &str) -> Result<u64, RpcError> {
    let h = s.trim_start_matches("0x");
    u64::from_str_radix(h, 16).map_err(|e| RpcError::Msg(e.to_string()))
}

pub fn data_amount_dec(data: &str) -> String {
    let h = data.trim_start_matches("0x").trim_start_matches('0');
    if h.is_empty() {
        return "0".into();
    }
    if h.len() > 32 {
        return format!("0x{h}");
    }
    u128::from_str_radix(h, 16)
        .map(|n| n.to_string())
        .unwrap_or_else(|_| format!("0x{h}"))
}

pub fn parse_logs(chain: &str, fallback_addr: &str, logs: &Value) -> Result<Vec<EventRow>, RpcError> {
    let arr = logs
        .as_array()
        .ok_or_else(|| RpcError::Msg("logs not array".into()))?;
    let mut out = Vec::new();
    for log in arr {
        let tx = log
            .get("transactionHash")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let log_index = log
            .get("logIndex")
            .and_then(|t| t.as_str())
            .map(parse_hex_u64)
            .transpose()?
            .unwrap_or(0) as i64;
        let block = log
            .get("blockNumber")
            .and_then(|t| t.as_str())
            .map(parse_hex_u64)
            .transpose()?
            .unwrap_or(0) as i64;
        let address = log
            .get("address")
            .and_then(|t| t.as_str())
            .unwrap_or(fallback_addr)
            .to_string();
        let topic0 = log
            .get("topics")
            .and_then(|t| t.get(0))
            .and_then(|t| t.as_str())
            .unwrap_or("");
        let kind = if topic0.eq_ignore_ascii_case(TRANSFER_TOPIC) {
            "Transfer"
        } else {
            "Log"
        };
        let data = log.get("data").and_then(|d| d.as_str()).unwrap_or("0x");
        let amount_raw = if kind == "Transfer" {
            data_amount_dec(data)
        } else {
            "0".into()
        };
        out.push(EventRow {
            chain: chain.into(),
            tx,
            log_index,
            block,
            address,
            kind: kind.into(),
            amount_raw,
            ok: true,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_transfer_amount() {
        let logs: Value = serde_json::from_str(include_str!("../fixtures/eth_logs.json")).unwrap();
        let rows = parse_logs("evm", "0xusdc", &logs).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, "Transfer");
        assert_eq!(rows[0].amount_raw, "1000000");
        assert_eq!(rows[0].tx, "0xabc");
    }

    #[test]
    fn hex_block() {
        assert_eq!(parse_hex_u64("0x10").unwrap(), 16);
    }
}
