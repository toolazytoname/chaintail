use std::path::PathBuf;
use std::process::Command;

fn exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_chaintail"))
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(name)
}

#[test]
fn doctor_secret() {
    let out = Command::new(exe())
        .args(["doctor", "--config", fixture("config.secret.json").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let s = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(!s.contains("PLANT-SECRET-DO-NOT-LOG"));
}

#[test]
fn follow_query_alert() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = dir.path().join("config.json");
    std::fs::write(&cfg, std::fs::read(fixture("config.ok.json")).unwrap()).unwrap();
    let db = dir.path().join("db.sqlite");
    let follow = Command::new(exe())
        .args([
            "follow",
            "--config",
            cfg.to_str().unwrap(),
            "--fixture",
            fixture("events.json").to_str().unwrap(),
            "--db",
            db.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(follow.status.success(), "{}", String::from_utf8_lossy(&follow.stderr));
    let q = Command::new(exe())
        .args(["query", "--config", cfg.to_str().unwrap(), "--db", db.to_str().unwrap(), "--fail"])
        .output()
        .unwrap();
    assert!(q.status.success());
    let qs = String::from_utf8_lossy(&q.stdout);
    assert!(qs.contains("0xccc"));
    assert!(!qs.contains("0xaaa"));
    let a = Command::new(exe())
        .args([
            "alert",
            "--config",
            cfg.to_str().unwrap(),
            "--db",
            db.to_str().unwrap(),
            "--min-amount",
            "2000000",
            "--notify-file",
            dir.path().join("a.jsonl").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    let as_ = String::from_utf8_lossy(&a.stdout);
    assert!(as_.contains("0xbbb"));
    assert!(!as_.contains("0xaaa"));
}
