//! Local-first, read-only chain event tail.

pub mod secrets;
pub mod store;

pub fn cli_name() -> &'static str {
    "chaintail"
}
