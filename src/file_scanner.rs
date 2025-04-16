use crate::config::{is_typescript_project, EXTENSIONS, IGNORE_FOLDERS, TYPESCRIPT_EXTENSIONS};
use glob::glob;
use indicatif::ProgressBar;
use regex::Regex;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Normalizes a file path for consistent handling across platforms, especially macOS.
///
/// On macOS, this function removes the `/private` prefix from paths if present, which can appear
/// due to temporary filesystem mounts. It also attempts to canonicalize the path to its absolute form.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` to normalize.
///
/// # Returns
///
/// Returns a `String` representing the normalized path. If canonicalization fails, returns the
/// original path as a string.
///
/// # Examples
///
/// ```
/// let path = Path::new("/private/tmp/file.txt");
/// let normalized = normalize_path(path);
/// // On macOS, might return "/tmp/file.txt"
/// println!("Normalized path: {}", normalized);
/// ```
fn normalize_path(path: &Path) -> String {
    let path_str = fs::canonicalize(path)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string());
    // On macOS, strip /private prefix if present
    if cfg!(target_os = "macos") && path_str.starts_with("/private") {
        path_str.replacen("/private", "", 1)
    } else {
        path_str
    }
}

/// Runs the TypeScript compiler (`tsc`) to detect unused imports (TS6133 errors).
///
/// This function executes `tsc --noEmit --pretty false` to collect diagnostics for unused imports
/// in a TypeScript project. If `tsc` fails or no TypeScript project is detected, it returns an empty set.
///
/// # Returns
///
/// Returns a `HashSet<String>` containing the names of unused imports identified by TS6133 errors.
/// Returns an empty set if the project is not TypeScript, `tsc` fails, or no unused imports are found.
///
/// # Examples
///
/// ```
/// let unused = get_typescript_unused_imports();
/// if !unused.is_empty() {
///     println!("Unused imports: {:?}", unused);
/// } else {
///     println!("No unused imports detected.");
/// }
/// ```
fn get_typescript_unused_imports() -> HashSet<String> {
    let mut unused_imports = HashSet::new();
    if !is_typescript_project() {
        return unused_imports;
    }

    // Run tsc with --noEmit to get diagnostics
    let output = Command::new("tsc")
        .args(["--noEmit", "--pretty", "false"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            // No errors, so no unused imports
            return unused_imports;
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            for line in stderr.lines() {
                if line.contains("TS6133") {
                    // Example: "file.ts(1,8): error TS6133: 'analytics' is declared but its value is never read."
                    if let Some(import_name) = extract_import_name(line) {
                        unused_imports.insert(import_name);
                    }
                }
            }
        }
        Err(_) => {
            // tsc failed (e.g., not installed), fall back to regex
        }
    }
    unused_imports
}

/// Extracts the import name from a TypeScript TS6133 diagnostic message.
///
/// Parses a diagnostic message to retrieve the name of an unused import. Special cases are handled,
/// such as mapping known imports (e.g., "analytics" to "@vercel/analytics").
///
/// # Arguments
///
/// * `diagnostic` - A string slice containing the TS6133 diagnostic message.
///
/// # Returns
///
/// Returns `Some(String)` with the extracted import name if parsing succeeds, or `None` if the
/// diagnostic format is invalid or no name is found.
///
/// # Examples
///
/// ```
/// let diagnostic = "file.ts(1,8): error TS6133: 'analytics' is declared but its value is never read.";
/// if let Some(name) = extract_import_name(diagnostic) {
///     println!("Unused import: {}", name); // Prints "Unused import: @vercel/analytics"
/// }
/// ```
fn extract_import_name(diagnostic: &str) -> Option<String> {
    let parts: Vec<&str> = diagnostic.split("'").collect();
    if parts.len() >= 2 {
        let name = parts[1].to_string();
        // Map to package name (e.g., "analytics" -> "@vercel/analytics")
        match name.as_str() {
            "analytics" => Some("@vercel/analytics".to_string()),
            _ => Some(name),
        }
    } else {
        None
    }
}

/// Scans project files to identify used dependencies, explored files, and ignored files.
///
/// This function searches for files matching configured extensions (e.g., `.js`, `.ts`) using glob
/// patterns, processes their content to find dependency usage, and respects ignore rules (e.g., for
/// folders like `node_modules`). For TypeScript files, it integrates with `tsc` to exclude unused imports.
///
/// # Arguments
///
/// * `dependencies` - A reference to a `HashSet<String>` containing the project's dependencies.
/// * `pb` - A reference to a `ProgressBar` for tracking scanning progress.
///
/// # Returns
///
/// Returns a tuple `(HashSet<String>, Vec<String>, Vec<String>)` containing:
/// - A `HashSet<String>` of used dependency names.
/// - A `Vec<String>` of explored file paths (normalized).
/// - A `Vec<String>` of ignored file or directory paths (normalized).
///
/// # Examples
///
/// ```
/// let dependencies = HashSet::new();
/// let pb = ProgressBar::new(100);
/// let (used, explored, ignored) = scan_files(&dependencies, &pb);
/// println!("Used dependencies: {:?}", used);
/// println!("Explored files: {:?}", explored);
/// println!("Ignored files: {:?}", ignored);
/// ```
pub fn scan_files(
    dependencies: &HashSet<String>,
    pb: &ProgressBar,
) -> (HashSet<String>, Vec<String>, Vec<String>) {
    let patterns: Vec<String> = EXTENSIONS
        .iter()
        .map(|ext| format!("**/*.{}", ext))
        .collect();
    let mut used_packages = HashSet::new();
    let mut ignored_files = Vec::new();
    let mut explored_files = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut typescript_files = Vec::new();

    for pattern in patterns {
        for entry in glob(&pattern).expect("Failed to read glob pattern") {
            pb.inc(1);

            match entry {
                Ok(path) if !path.is_dir() && !path.is_symlink() => {
                    let abs_path = normalize_path(&path);
                    if seen_paths.contains(&abs_path) {
                        continue;
                    }
                    seen_paths.insert(abs_path.clone());

                    if should_ignore(&path) {
                        ignored_files.push(abs_path);
                        continue;
                    }

                    let extension = path.extension().and_then(OsStr::to_str);
                    if extension.map_or(false, |ext| TYPESCRIPT_EXTENSIONS.contains(&ext)) {
                        typescript_files.push(abs_path.clone());
                    } else if let Ok(content) = fs::read_to_string(&path) {
                        used_packages.extend(find_dependencies_in_content(&content, dependencies));
                        // deps from package.json only
                    }

                    explored_files.push(abs_path);
                }

                Ok(path) => {
                    let abs_path = normalize_path(&path);
                    if should_ignore(&path) && !seen_paths.contains(&abs_path) {
                        ignored_files.push(abs_path.clone());
                        seen_paths.insert(abs_path);
                    }
                }

                Err(_) => {}
            }
        }
    }

    // Process TypeScript files with tsc
    let unused_imports = get_typescript_unused_imports();
    for path in &typescript_files {
        if let Ok(content) = fs::read_to_string(path) {
            let found = find_dependencies_in_content(&content, dependencies);
            for dep in found {
                if !unused_imports.contains(&dep) {
                    used_packages.insert(dep);
                }
            }
        }
    }

    (used_packages, explored_files, ignored_files)
}

/// Searches file content for references to project dependencies using regex patterns.
///
/// This function builds regex patterns to match common import/require statements for each dependency
/// and checks if they appear in the provided content.
///
/// # Arguments
///
/// * `content` - A string slice containing the file content to search.
/// * `dependencies` - A reference to a `HashSet<String>` containing dependency names to look for.
///
/// # Returns
///
/// Returns a `HashSet<String>` containing the names of dependencies found in the content.
///
/// # Examples
///
/// ```
/// let content = r#"import { foo } from "lodash"; require("moment");"#;
/// let mut deps = HashSet::new();
/// deps.insert("lodash".to_string());
/// deps.insert("moment".to_string());
/// let found = find_dependencies_in_content(content, &deps);
/// assert!(found.contains("lodash"));
/// assert!(found.contains("moment"));
/// ```
fn find_dependencies_in_content(content: &str, dependencies: &HashSet<String>) -> HashSet<String> {
    let mut found = HashSet::new();

    for dep in dependencies {
        let dep_pattern = regex::escape(dep);
        let regex_str = format!(
            r#"(?m)(?:import\s*(?:\{{[^}}]*\}}|\w*)\s*from\s*['"]{}['"]|require\s*\(\s*['"]{}['"]\s*\)|import\s*['"]{}['"]\s*;)"#,
            dep_pattern, dep_pattern, dep_pattern
        );
        let regex = Regex::new(&regex_str).unwrap();

        if regex.is_match(content) {
            found.insert(dep.clone());
        }
    }

    found
}

/// Determines if a path should be ignored based on configured ignore folders.
///
/// Checks if any component of the path matches a folder in the `IGNORE_FOLDERS` list (e.g., `node_modules`).
///
/// # Arguments
///
/// * `path` - A reference to a `Path` to check.
///
/// # Returns
///
/// Returns `true` if the path contains an ignored folder, `false` otherwise.
///
/// # Examples
///
/// ```
/// let path = Path::new("node_modules/package/file.js");
/// assert!(should_ignore(&path)); // node_modules is ignored
/// let path = Path::new("src/file.js");
/// assert!(!should_ignore(&path)); // src is not ignored
/// ```
fn should_ignore(path: &Path) -> bool {
    path.components().any(|component| {
        IGNORE_FOLDERS
            .iter()
            .any(|folder| component.as_os_str() == OsStr::new(folder))
    })
}
