use crate::config::{is_typescript_project, EXTENSIONS, IGNORE_FOLDERS};
use glob::glob;
use indicatif::ProgressBar;
use regex::Regex;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

// Helper function to normalize paths for macOS
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

// Run tsc and collect unused imports (ts(6133))
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

// Extract import name from TS6133 diagnostic
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
                    if extension == Some("ts") || extension == Some("tsx") {
                        typescript_files.push(abs_path.clone());
                    } else if let Ok(content) = fs::read_to_string(&path) {
                        used_packages.extend(find_dependencies_in_content(&content, dependencies));
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

fn should_ignore(path: &Path) -> bool {
    path.components().any(|component| {
        IGNORE_FOLDERS
            .iter()
            .any(|folder| component.as_os_str() == OsStr::new(folder))
    })
}
