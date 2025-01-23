use colored::*;
use glob::glob;
use serde_json::Value;
use std::{collections::HashSet, ffi::OsStr, fs, io, path::Path, process::Command};

const PACKAGE_JSON_PATH: &str = "package.json";
const EXTENSIONS: &str = "**/*.{js,ts,jsx,tsx,mdx}";
const IGNORE_FOLDERS: [&str; 10] = [
    "node_modules",
    "dist",
    "build",
    "public",
    ".next",
    ".git",
    "coverage",
    "cypress",
    "test",
    "output",
];

fn main() {
    let dry_run = std::env::args().any(|arg| arg == "--dry-run");

    // read package.json
    let package_json: Value = match fs::read_to_string(PACKAGE_JSON_PATH) {
        Ok(content) => serde_json::from_str(&content).expect("Invalid JSON in package.json"),
        Err(_) => {
            eprintln!("Error: `package.json` not found.");
            return;
        }
    };

    // collect dependencies
    let dependencies = package_json
        .get("dependencies")
        .and_then(Value::as_object)
        .map_or_else(|| serde_json::Map::new(), |map| map.clone())
        .keys()
        .cloned()
        .collect::<HashSet<_>>();

    // used dependencies
    let mut used_packages = HashSet::new();
    let mut ignored_files = Vec::new();
    for entry in glob(EXTENSIONS).expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            if should_ignore(&path) {
                ignored_files.push(path.display().to_string());
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for dep in &dependencies {
                    if content.contains(dep) {
                        used_packages.insert(dep.clone());
                    }
                }
            }
        }
    }

    // identify unused dependencies
    let unused_dependencies: Vec<_> = dependencies.difference(&used_packages).cloned().collect();

    // print unused dependencies
    println!("{}", "\nDependency Usage Report".bold().blue());
    println!("{}", "------------------------".blue());
    println!("{}: {}", "Project".bold(), PACKAGE_JSON_PATH);
    println!("{}: {}", "Extensions".bold(), EXTENSIONS);
    println!(
        "{}: {}",
        "Ignored Folders".bold(),
        IGNORE_FOLDERS.join(", ")
    );
    println!("{}: {}", "Package Manager".bold(), detect_package_manager());
    println!("{}: {}", "Total dependencies".bold(), dependencies.len());
    println!(
        "{}: {}",
        "Used dependencies".bold().green(),
        used_packages.len()
    );
    println!(
        "{}: {}",
        "Unused dependencies".bold().red(),
        unused_dependencies.len()
    );

    if !unused_dependencies.is_empty() {
        println!("\n{}", "Unused Dependencies:".red().bold());
        for dep in &unused_dependencies {
            println!("- {}", dep.red());
        }
    } else {
        println!("{}", "\nNo unused dependencies found!".green());
    }

    // display ignored files and folders
    if !ignored_files.is_empty() {
        println!("\n{}", "Ignored Files and Folders:".yellow().bold());
        for file in ignored_files {
            println!("- {}", file.yellow());
        }
    }

    // prune unused dependencies
    if !unused_dependencies.is_empty() {
        handle_unused_dependencies(&unused_dependencies, dry_run);
    }
}

fn handle_unused_dependencies(unused_dependencies: &[String], dry_run: bool) {
    if dry_run {
        println!(
            "{}",
            "\nDry-run mode enabled. The following dependencies would be deleted:".yellow()
        );
        for dep in unused_dependencies {
            println!("- {}", dep.yellow());
        }
    } else {
        println!(
            "{}",
            "\nProceeding to delete the unused dependencies..."
                .red()
                .bold()
        );

        if !ask_for_confirmation() {
            println!("{}", "Aborted!".red());
            return;
        }
        for dep in unused_dependencies {
            if uninstall_dependency(dep, &detect_package_manager()) {
                println!("{} {}", "Deleted:".green(), dep.green());
            } else {
                println!("{} {}", "Failed to delete:".red(), dep.red());
            }
        }
    }
}

fn detect_package_manager() -> String {
    if Path::new("pnpm-lock.yaml").exists() {
        "pnpm".to_string()
    } else if Path::new("yarn.lock").exists() {
        "yarn".to_string()
    } else if Path::new("bun.lockb").exists() {
        "bun".to_string()
    } else {
        "npm".to_string()
    }
}

fn ask_for_confirmation() -> bool {
    println!(
        "{}",
        "Are you sure you want to delete the unused dependencies? (yes/no)"
            .yellow()
            .bold()
    );
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_lowercase() == "yes"
}

fn uninstall_dependency(dependency: &str, package_manager: &str) -> bool {
    let output = Command::new(package_manager)
        .args(&["uninstall", dependency])
        .output();

    match output {
        Ok(result) if result.status.success() => true,
        _ => false,
    }
}

fn should_ignore(path: &Path) -> bool {
    IGNORE_FOLDERS.iter().any(|folder| {
        path.components()
            .any(|component| component.as_os_str() == OsStr::new(folder))
    })
}
