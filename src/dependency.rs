use colored::*;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::utils::get_file_name_and_extension;

/// Reads and parses a `package.json` file into a JSON value.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to the `package.json` file.
///
/// # Returns
///
/// Returns `Ok(Value)` containing the parsed JSON if successful.
/// Returns `Err(String)` with an error message if the file is not found or contains invalid JSON.
///
/// # Examples
///
/// ```
/// match read_package_json("package.json") {
///     Ok(json) => println!("Successfully parsed package.json: {:?}", json),
///     Err(e) => eprintln!("Failed to read package.json: {}", e),
/// }
/// ```
pub fn read_package_json(path: &str) -> Result<Value, String> {
    let file_name_and_extension = get_file_name_and_extension(path).unwrap_or_default();
    let content = fs::read_to_string(path)
        .map_err(|_| format!("Error: `{}` not found.", file_name_and_extension.0))?;
    serde_json::from_str(&content).map_err(|_| "Error: Invalid JSON in package.json.".to_string())
}

/// Collects all required dependencies from `package.json` and supported lockfiles.
///
/// This function checks for `package.json` and lockfiles (`package-lock.json`, `yarn.lock`,
/// `pnpm-lock.yaml`, `bun.lock`) to gather dependencies. If multiple lockfiles are detected,
/// it warns the user and returns an empty set to avoid ambiguity.
///
/// # Arguments
///
/// * `dir_path` - A string slice representing the path to the project directory.
///
/// # Returns
///
/// Returns a `HashSet<String>` containing the names of all required dependencies (from
/// `dependencies`, `devDependencies`, and lockfiles). Returns an empty set if multiple
/// lockfiles are detected or if no valid dependencies are found.
///
/// # Examples
///
/// ```
/// let deps = get_required_dependencies();
/// if !deps.is_empty() {
///     println!("Required dependencies: {:?}", deps);
/// } else {
///     println!("No dependencies found or multiple lockfiles detected.");
/// }
/// ```
pub fn get_required_dependencies(dir_path: &str) -> HashSet<String> {
    let mut required = HashSet::new();

    // Define paths for lockfiles
    let package_lock_json_path = Path::new(dir_path).join("package-lock.json");
    let yarn_lock_path = Path::new(dir_path).join("yarn.lock");
    let pnpm_lock_yaml_path = Path::new(dir_path).join("pnpm-lock.yaml");
    let bun_lock_path = Path::new(dir_path).join("bun.lock");

    // Check for the existence of lockfiles
    let lockfiles = [
        (
            "package-lock.json",
            Path::new(&package_lock_json_path).exists(),
        ),
        ("yarn.lock", Path::new(&yarn_lock_path).exists()),
        ("pnpm-lock.yaml", Path::new(&pnpm_lock_yaml_path).exists()),
        ("bun.lock", Path::new(&bun_lock_path).exists()),
    ];

    let existing_lockfiles: Vec<&str> = lockfiles
        .iter()
        .filter_map(|(name, exists)| if *exists { Some(*name) } else { None })
        .collect();

    if existing_lockfiles.len() > 1 {
        eprintln!(
            "{}: Multiple lockfiles detected ({}). Please use only one package manager.",
            "Warning".yellow().bold(),
            existing_lockfiles.join(", ")
        );
        return HashSet::new();
    }

    // Process package.json first to ensure top-level dependencies are included
    let package_json_path = Path::new(dir_path).join("package.json");
    if let Ok(package_json) = read_package_json(package_json_path.to_str().unwrap()) {
        if let Some(deps) = package_json.get("dependencies").and_then(Value::as_object) {
            required.extend(deps.keys().cloned());
        }

        // TODO: review the devDependencies logic (handle them in a different case)
        if let Some(dev_deps) = package_json
            .get("devDependencies")
            .and_then(Value::as_object)
        {
            required.extend(dev_deps.keys().cloned());
        }
    }

    // Process single lockfile
    if let Some(lockfile) = existing_lockfiles.first() {
        match *lockfile {
            // package-lock.json
            "package-lock.json" => {
                if let Ok(content) = fs::read_to_string(package_lock_json_path) {
                    if let Ok(lock) = serde_json::from_str::<Value>(&content) {
                        if let Some(packages) = lock.get("packages").and_then(Value::as_object) {
                            for key in packages.keys() {
                                let package_name = key
                                    .strip_prefix("node_modules/")
                                    .unwrap_or(key)
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
            // yarn.lock
            "yarn.lock" => {
                if let Ok(content) = fs::read_to_string(yarn_lock_path) {
                    for line in content.lines() {
                        if line.ends_with(':') && !line.starts_with('#') && !line.trim().is_empty()
                        {
                            let dep = line.trim_end_matches(':').trim();
                            let package_name = dep
                                .split(',')
                                .next()
                                .unwrap_or(dep)
                                .trim()
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
            // pnpm-lock.yaml
            "pnpm-lock.yaml" => {
                if let Ok(content) = fs::read_to_string(pnpm_lock_yaml_path) {
                    if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        if let Some(deps) = yaml
                            .get("dependencies")
                            .or_else(|| yaml.get("devDependencies")) // TODO: review the devDependencies logic
                            .and_then(|v| v.as_mapping())
                        {
                            for key in deps.keys() {
                                if let Some(key_str) = key.as_str() {
                                    required.insert(key_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
            // bun.lock
            "bun.lock" => {
                if let Ok(content) = fs::read_to_string(bun_lock_path) {
                    if let Ok(lock) = serde_json::from_str::<Value>(&content) {
                        if let Some(workspaces) = lock
                            .get("workspaces")
                            .and_then(|v| v.get(""))
                            .and_then(Value::as_object)
                        {
                            if let Some(deps) =
                                workspaces.get("dependencies").and_then(Value::as_object)
                            {
                                required.extend(deps.keys().cloned());
                            }

                            if let Some(dev_deps) =
                                workspaces.get("devDependencies").and_then(Value::as_object)
                            // TODO: review the devDependencies logic
                            {
                                required.extend(dev_deps.keys().cloned());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    required
}

/// Reads a `.cnpignore` file and returns its non-comment, non-empty lines as a set.
///
/// The function parses the `.cnpignore` file, ignoring empty lines, lines starting with `#`,
/// and inline comments (text after `#`). If the file is not found, an empty set is returned.
///
/// # Returns
///
/// Returns a `HashSet<String>` containing the trimmed, non-empty, non-comment lines from
/// the `.cnpignore` file. Returns an empty set if the file does not exist or cannot be read.
///
/// # Examples
///
/// ```
/// let ignore_patterns = read_cnpignore();
/// if !ignore_patterns.is_empty() {
///     println!("Ignore patterns: {:?}", ignore_patterns);
/// } else {
///     println!("No .cnpignore patterns found.");
/// }
/// ```
pub fn read_cnpignore() -> HashSet<String> {
    fs::read_to_string(".cnpignore")
        .map(|content| {
            content
                .lines()
                .map(|line| {
                    let trimmed = line.trim();
                    // Strip inline comments
                    trimmed
                        .split('#')
                        .next()
                        .unwrap_or(trimmed)
                        .trim()
                        .to_string()
                })
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .collect()
        })
        .unwrap_or_default()
}
