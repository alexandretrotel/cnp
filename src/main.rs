use clap::Command;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use walkdir::WalkDir;

const PACKAGE_JSON_PATH: &str = "package.json";
const EXTENSIONS: [&str; 4] = ["js", "ts", "jsx", "tsx"];
const IGNORE_FOLDERS: [&str; 4] = ["node_modules", "dist", "build", "public"];

fn main() {
    let matches = Command::new("cnp")
        .version("1.0.0")
        .author("Alexandre Trotel")
        .about("Checks for unused dependencies in a project")
        .get_matches();

    if matches.contains_id("version") {
        println!("cnp version 1.0.0");
        return;
    }

    // read package.json file into a string
    let package_json_path =
        fs::canonicalize(PACKAGE_JSON_PATH).expect("Failed to find package.json");
    let package_json_content =
        fs::read_to_string(package_json_path).expect("Failed to read package.json");
    // parse package.json content into a JSON Value
    let package_json: Value =
        serde_json::from_str(&package_json_content).expect("Invalid JSON in package.json");

    // extract dependencies from the parsed package.json
    let dependencies = extract_dependencies(&package_json);
    println!("Dependencies found: {}", dependencies.len());

    // search for JavaScript/TypeScript files in the project directory
    let project_files = find_files(".");
    println!("Files found: {} (showing 5 samples)", project_files.len());
    for file in project_files.iter().take(5) {
        println!("- {}", file);
    }

    // check which dependencies are unused by scanning project files
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

    // show completion progress
    let completion_percentage =
        (unused_dependencies.len() as f64 / dependencies.len() as f64) * 100.0;
    println!(
        "Progress: {:.2}% of dependencies are unused.",
        completion_percentage
    );
}

/// extract dependencies from package.json, including devDependencies
fn extract_dependencies(package_json: &Value) -> HashSet<String> {
    let mut dependencies = HashSet::new();
    if let Value::Object(map) = package_json {
        // look for both dependencies and devDependencies
        for key in ["dependencies", "devDependencies"] {
            if let Some(Value::Object(deps)) = map.get(key) {
                // collect the keys (dependency names)
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
        if path.is_dir() && IGNORE_FOLDERS.iter().any(|&folder| path.ends_with(folder)) {
            println!("Ignoring folder: {}", path.display());
            continue;
        }
        if let Some(ext) = path.extension() {
            if EXTENSIONS.contains(&ext.to_str().unwrap()) {
                files.push(path.to_str().unwrap().to_string());
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
                // if a dependency is found in the file, remove it from unused
                if content.contains(dep) {
                    unused.remove(dep);
                }
            }
        }
    }
    unused
}
