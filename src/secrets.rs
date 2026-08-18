const FORBIDDEN: &[&str] = &["private_key", "privkey", "mnemonic", "seed", "wif", "secret_key"];

fn norm(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub fn forbidden_fields(v: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    walk(v, "", &mut out);
    out.into_iter()
        .filter(|path| {
            let leaf = path.rsplit('.').next().unwrap_or(path);
            let n = norm(leaf);
            FORBIDDEN.iter().any(|needle| n.contains(needle))
        })
        .collect()
}

fn walk(v: &serde_json::Value, prefix: &str, out: &mut Vec<String>) {
    if let serde_json::Value::Object(map) = v {
        for (k, child) in map {
            let path = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{prefix}.{k}")
            };
            out.push(path.clone());
            walk(child, &path, out);
        }
    }
}
