use clap::{Arg, Command};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use walkdir::WalkDir;

const PACKAGE_JSON_PATH: &str = "package.json";
const EXTENSIONS: [&str; 4] = ["js", "ts", "jsx", "tsx"];
const IGNORE_FOLDERS: [&str; 9] = [
    "node_modules",
    "dist",
    "build",
    "public",
    ".next",
    ".git",
    "coverage",
    "cypress",
    "test",
];
const VERSION: &str = "0.2.0";

fn main() {
    let matches = Command::new("cnp")
        .version(VERSION)
        .author("Alexandre Trotel")
        .about("Checks for unused dependencies in a project")
        .arg(
            Arg::new("clean")
                .long("clean")
                .action(clap::ArgAction::SetTrue)
                .help("Remove unused dependencies from package.json"),
        )
        .get_matches();

    let package_json_path = fs::canonicalize(PACKAGE_JSON_PATH).unwrap_or_else(|_| {
        panic!(
            "Failed to find package.json at the expected path: {}",
            PACKAGE_JSON_PATH
        )
    });
    let package_json_content = fs::read_to_string(&package_json_path).unwrap_or_else(|_| {
        panic!(
            "Failed to read package.json at path: {}",
            package_json_path.display()
        )
    });
    let mut package_json: Value =
        serde_json::from_str(&package_json_content).unwrap_or_else(|_| {
            panic!(
                "Invalid JSON format in package.json at path: {}",
                package_json_path.display()
            )
        });

    let dependencies = extract_dependencies(&package_json);
    println!("Dependencies found: {}", dependencies.len());

    let project_files = find_files(".");
    println!("Files found: {} (showing 5 samples)", project_files.len());
    for file in project_files.iter().take(5) {
        println!("- {}", file);
    }

    let unused_dependencies = find_unused_dependencies(&dependencies, &project_files);
    println!("Unused dependencies: {}", unused_dependencies.len());
    if !unused_dependencies.is_empty() {
        println!("Showing first 5 unused dependencies:");
        for dep in unused_dependencies.iter().take(5) {
            println!("- {}", dep);
        }
    } else {
        println!("All dependencies are used.");
    }

    if matches.contains_id("clean") {
        // clean unused dependencies
        clean_unused_dependencies(&mut package_json, &unused_dependencies);
        // write the modified package.json
        fs::write(
            &package_json_path,
            serde_json::to_string_pretty(&package_json)
                .unwrap_or_else(|_| panic!("Failed to serialize modified package.json to string")),
        )
        .unwrap_or_else(|_| {
            panic!(
                "Failed to write modified package.json at path: {}",
                package_json_path.display()
            )
        });
        println!("Cleaned unused dependencies.");
    }

    let completion_percentage =
        (unused_dependencies.len() as f64 / dependencies.len() as f64) * 100.0;
    println!(
        "Progress: {:.2}% of dependencies are unused.",
        completion_percentage
    );
}

/// extract dependencies from package.json
fn extract_dependencies(package_json: &Value) -> HashSet<String> {
    let mut dependencies = HashSet::new();
    if let Value::Object(map) = package_json {
        for key in ["dependencies"] {
            if let Some(Value::Object(deps)) = map.get(key) {
                dependencies.extend(deps.keys().cloned());
            }
        }
    }
    dependencies
}

/// search for files that matches the JS_TS_GLOB pattern in the given directory
fn find_files(directory: &str) -> Vec<String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            if IGNORE_FOLDERS.iter().any(|&folder| path.ends_with(folder)) {
                continue;
            }
        } else {
            if path.ancestors().any(|ancestor| {
                IGNORE_FOLDERS
                    .iter()
                    .any(|&folder| ancestor.ends_with(folder))
            }) {
                continue;
            }
            if let Some(ext) = path.extension() {
                if EXTENSIONS.contains(&ext.to_str().unwrap()) {
                    files.push(path.to_str().unwrap().to_string());
                }
            }
        }
    }
    files
}

/// check each project file for dependency usage and identify unused ones
fn find_unused_dependencies(dependencies: &HashSet<String>, files: &[String]) -> HashSet<String> {
    let mut unused = dependencies.clone();
    for file in files {
        if let Ok(content) = fs::read_to_string(file) {
            for dep in dependencies {
                if content.contains(dep) {
                    unused.remove(dep);
                }
            }
        }
    }
    unused
}

/// clean the unused dependencies from the package.json
fn clean_unused_dependencies(package_json: &mut Value, unused_dependencies: &HashSet<String>) {
    if let Value::Object(map) = package_json {
        if let Some(Value::Object(deps)) = map.get_mut("dependencies") {
            for dep in unused_dependencies {
                deps.remove(dep);
            }
        }
    }
}
