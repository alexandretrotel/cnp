use crate::config::{EXTENSIONS, IGNORE_FOLDERS};
use glob::glob;
use indicatif::ProgressBar;
use regex::Regex;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub fn scan_files(
    dependencies: &HashSet<String>,
    pb: &ProgressBar,
) -> (HashSet<String>, Vec<String>, Vec<String>) {
    let patterns: Vec<String> = EXTENSIONS
        .iter()
        .flat_map(|ext| vec![format!("*.{}", ext), format!("**/*.{}", ext)])
        .collect();
    let mut used_packages = HashSet::new();
    let mut ignored_files = Vec::new();
    let mut explored_files = Vec::new();

    for pattern in patterns {
        for entry in glob(&pattern).expect("Failed to read glob pattern") {
            pb.inc(1);
            match entry {
                Ok(path) if !path.is_dir() && !path.is_symlink() => {
                    if should_ignore(&path) {
                        ignored_files.push(path.display().to_string());
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        used_packages.extend(find_dependencies_in_content(&content, dependencies));
                    }
                    explored_files.push(path.display().to_string());
                }
                Ok(path) => {
                    if should_ignore(&path) {
                        ignored_files.push(path.display().to_string());
                    }
                }
                Err(_) => {}
            }
        }
    }

    (used_packages, explored_files, ignored_files)
}

fn find_dependencies_in_content(content: &str, dependencies: &HashSet<String>) -> HashSet<String> {
    let mut found = HashSet::new();
    for dep in dependencies {
        let dep_pattern = if dep.starts_with('@') {
            let parts: Vec<&str> = dep.split('/').collect();
            if parts.len() > 1 {
                format!("{}/{}", parts[0], parts[1])
            } else {
                dep.clone()
            }
        } else {
            dep.clone()
        };

        let import_regex = Regex::new(&format!(
            r#"import\s+.*?\s+from\s+['"]({}(/[^'"]*)?)['"]"#,
            regex::escape(&dep_pattern)
        ))
        .unwrap();
        let require_regex = Regex::new(&format!(
            r#"require\s*\(\s*['"]({}(/[^'"]*)?)['"]\s*\)"#,
            regex::escape(&dep_pattern)
        ))
        .unwrap();

        if import_regex.is_match(content) || require_regex.is_match(content) {
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
