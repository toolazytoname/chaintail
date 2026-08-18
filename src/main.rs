use chaintail::secrets::forbidden_fields;
use chaintail::store::{self, EventsFile};
use clap::{Parser, Subcommand};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "chaintail", about = "Local-first read-only chain event tail")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Init {
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },
    Doctor {
        #[arg(long)]
        config: PathBuf,
    },
    Follow {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        fixture: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
    },
    Query {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        fail: bool,
        #[arg(long)]
        min_amount: Option<i64>,
    },
    Alert {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        fail: bool,
        #[arg(long)]
        min_amount: Option<i64>,
        #[arg(long)]
        notify_file: Option<PathBuf>,
    },
}

fn load_cfg(path: &std::path::Path) -> Result<serde_json::Value, ExitCode> {
    let raw = std::fs::read_to_string(path).map_err(|e| {
        eprintln!("{e}");
        ExitCode::from(1)
    })?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
        eprintln!("{e}");
        ExitCode::from(1)
    })?;
    let hits = forbidden_fields(&v);
    if !hits.is_empty() {
        eprintln!("doctor: forbidden secret field(s): {}", hits.join(", "));
        return Err(ExitCode::from(2));
    }
    Ok(v)
}

fn db_path(cfg: &serde_json::Value, override_db: Option<PathBuf>) -> PathBuf {
    override_db.unwrap_or_else(|| {
        PathBuf::from(
            cfg.get("db")
                .and_then(|v| v.as_str())
                .unwrap_or("chaintail.sqlite"),
        )
    })
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init { dir } => {
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("{e}");
                return ExitCode::from(1);
            }
            let cfg = dir.join("config.json");
            if !cfg.exists() {
                let _ = std::fs::write(
                    &cfg,
                    "{\n  \"chain\": \"evm-fixture\",\n  \"db\": \"chaintail.sqlite\",\n  \"notify\": {\"kind\": \"file\", \"path\": \"alerts.jsonl\"}\n}\n",
                );
            }
            println!("wrote {}", cfg.display());
            ExitCode::SUCCESS
        }
        Cmd::Doctor { config } => match load_cfg(&config) {
            Ok(v) => {
                println!("ok chain={}", v.get("chain").and_then(|x| x.as_str()).unwrap_or("?"));
                ExitCode::SUCCESS
            }
            Err(c) => c,
        },
        Cmd::Follow {
            config,
            fixture,
            db,
        } => {
            let cfg = match load_cfg(&config) {
                Ok(v) => v,
                Err(c) => return c,
            };
            let db = db_path(&cfg, db);
            let file: EventsFile = match serde_json::from_str(&std::fs::read_to_string(&fixture).unwrap_or_default()) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(1);
                }
            };
            let conn = match store::open(&db) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(1);
                }
            };
            match store::ingest(&conn, &file.events) {
                Ok(n) => {
                    println!("{}", serde_json::json!({"ingested": n, "db": db}));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::from(1)
                }
            }
        }
        Cmd::Query {
            config,
            db,
            fail,
            min_amount,
        } => {
            let cfg = match load_cfg(&config) {
                Ok(v) => v,
                Err(c) => return c,
            };
            let db = db_path(&cfg, db);
            let conn = match store::open(&db) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(1);
                }
            };
            match store::query(&conn, fail, min_amount) {
                Ok(rows) => {
                    for r in rows {
                        println!(
                            "{}",
                            serde_json::json!({
                                "chain": r.chain,
                                "tx": r.tx,
                                "log_index": r.log_index,
                                "kind": r.kind,
                                "amount_raw": r.amount_raw,
                                "ok": if r.ok { 1 } else { 0 },
                            })
                        );
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::from(1)
                }
            }
        }
        Cmd::Alert {
            config,
            db,
            fail,
            min_amount,
            notify_file,
        } => {
            let cfg = match load_cfg(&config) {
                Ok(v) => v,
                Err(c) => return c,
            };
            let db = db_path(&cfg, db);
            let dest = notify_file.unwrap_or_else(|| {
                PathBuf::from(
                    cfg.pointer("/notify/path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("alerts.jsonl"),
                )
            });
            let conn = match store::open(&db) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(1);
                }
            };
            let rows = match store::query(&conn, fail, min_amount) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(1);
                }
            };
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&dest) {
                for r in &rows {
                    let ev = serde_json::json!({
                        "kind": if r.ok { "amount" } else { "fail" },
                        "tx": r.tx,
                        "amount_raw": r.amount_raw,
                        "ok": if r.ok { 1 } else { 0 },
                    });
                    let _ = writeln!(f, "{ev}");
                    println!("{ev}");
                }
            }
            println!("notify file:{}", dest.display());
            ExitCode::SUCCESS
        }
    }
}
