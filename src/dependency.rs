use serde_json::Value;
use std::collections::HashSet;
use std::fs;

pub fn read_package_json(path: &str) -> Result<Value, String> {
    let content = fs::read_to_string(path).map_err(|_| format!("Error: `{}` not found.", path))?;
    serde_json::from_str(&content).map_err(|_| "Error: Invalid JSON in package.json.".to_string())
}

pub fn get_required_dependencies() -> HashSet<String> {
    let mut required = HashSet::new();

    // package-lock.json
    if let Ok(content) = fs::read_to_string("package-lock.json") {
        if let Ok(lock) = serde_json::from_str::<Value>(&content) {
            if let Some(deps) = lock.get("dependencies").and_then(Value::as_object) {
                required.extend(deps.keys().filter(|k| !k.is_empty()).cloned());
            }
            if let Some(packages) = lock.get("packages").and_then(Value::as_object) {
                for key in packages.keys() {
                    if let Some(package_name) = key.strip_prefix("/").and_then(|k| {
                        if k.starts_with('@') {
                            k.splitn(2, '@').next()
                        } else {
                            k.split('@').next()
                        }
                    }) {
                        if !package_name.is_empty() {
                            required.insert(package_name.to_string());
                        }
                    }
                }
            }
        }
    }

    // yarn.lock
    if let Ok(content) = fs::read_to_string("yarn.lock") {
        for line in content.lines() {
            if line.ends_with(':') && !line.starts_with('#') && !line.trim().is_empty() {
                let dep = line.trim_end_matches(':').trim();
                if let Some(package_name) = dep.split('@').next() {
                    if !package_name.is_empty() {
                        required.insert(package_name.to_string());
                    }
                }
            }
        }
    }

    // pnpm-lock.yaml
    if let Ok(content) = fs::read_to_string("pnpm-lock.yaml") {
        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(packages) = yaml.get("packages").and_then(|v| v.as_mapping()) {
                for key in packages.keys() {
                    if let Some(key_str) = key.as_str() {
                        let package_name = key_str
                            .strip_prefix('/')
                            .unwrap_or(key_str)
                            .split('@')
                            .next()
                            .unwrap_or("")
                            .to_string();
                        if !package_name.is_empty() {
                            required.insert(package_name);
                        }
                    }
                }
            }
        }
    }

    // bun.lock
    if let Ok(content) = fs::read_to_string("bun.lock") {
        if let Ok(lock) = serde_json::from_str::<Value>(&content) {
            if let Some(packages) = lock.get("packages").and_then(Value::as_object) {
                required.extend(packages.keys().filter(|k| !k.is_empty()).cloned());
            }
        }
    }

    required
}

pub fn read_cnpignore() -> HashSet<String> {
    fs::read_to_string(".cnpignore")
        .map(|content| {
            content
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}
