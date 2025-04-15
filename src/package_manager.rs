use std::path::Path;

/// Detects the package manager used in the current project based on lockfile presence.
///
/// This function checks for specific lockfiles (`pnpm-lock.yaml`, `yarn.lock`, `bun.lock`) to
/// determine the package manager. If none are found, it defaults to `npm`.
///
/// # Returns
///
/// Returns a `String` representing the detected package manager:
/// - `"pnpm"` if `pnpm-lock.yaml` exists.
/// - `"yarn"` if `yarn.lock` exists.
/// - `"bun"` if `bun.lock` exists.
/// - `"npm"` if no recognized lockfile is found.
///
/// # Examples
///
/// ```
/// let package_manager = detect_package_manager();
/// println!("Detected package manager: {}", package_manager);
/// // If `yarn.lock` exists, prints: "Detected package manager: yarn"
/// ```
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
