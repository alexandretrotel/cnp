use std::path::Path;

pub fn detect_package_manager() -> String {
    if Path::new("pnpm-lock.yaml").exists() {
        "pnpm".to_string()
    } else if Path::new("yarn.lock").exists() {
        "yarn".to_string()
    } else if Path::new("bun.lock").exists() {
        "bun".to_string()
    } else {
        "npm".to_string()
    }
}
