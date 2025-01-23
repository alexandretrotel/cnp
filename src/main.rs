use colored::*;
use glob::glob;
use regex::Regex;
use serde_json::Value;
use std::{cmp::min, collections::HashSet, ffi::OsStr, fs, io, path::Path, process::Command};

const PACKAGE_JSON_PATH: &str = "package.json";
const EXTENSIONS: [&str; 5] = ["js", "ts", "jsx", "tsx", "mdx"];
const MAX_EXPLORED_FILES: usize = 5;
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
    // dry-run mode
    let dry_run = std::env::args().any(|arg| arg == "--dry-run");

    // glob patterns
    let patterns: Vec<String> = EXTENSIONS
        .iter()
        .flat_map(|ext| vec![format!("*.{}", ext), format!("**/*.{}", ext)])
        .collect();

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
    let mut explored_files = Vec::new();

    for pattern in &patterns {
        for entry in glob(pattern).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                if path.is_dir() || path.is_symlink() {
                    if should_ignore(&path) {
                        ignored_files.push(path.display().to_string());
                        continue;
                    }
                }

                if should_ignore(&path) {
                    ignored_files.push(path.display().to_string());
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    used_packages.extend(find_dependencies_in_content(&content, &dependencies));
                }

                explored_files.push(path.display().to_string());
            }
        }
    }

    // identify unused dependencies
    let unused_dependencies: Vec<_> = dependencies.difference(&used_packages).cloned().collect();

    // print unused dependencies
    println!("{}", "\nDependency Usage Report".bold().blue());
    println!("{}", "------------------------".blue());
    println!("{}: {}", "Project".bold(), PACKAGE_JSON_PATH);
    println!("{}: {:?}", "Extensions".bold(), EXTENSIONS);
    println!(
        "{}: {}",
        "Ignored Folders".bold(),
        IGNORE_FOLDERS.join(", ")
    );
    println!("{}: {}", "Explored Files".bold(), explored_files.len());
    println!(
        "{}: {}",
        "Some Explored Files".bold(),
        explored_files[..min(MAX_EXPLORED_FILES, explored_files.len())].join(", ")
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

    if !used_packages.is_empty() {
        println!("\n{}", "Used Dependencies:".green().bold());
        for dep in &used_packages {
            println!("- {}", dep.green());
        }
    } else {
        println!("{}", "\nNo used dependencies found!".red());
    }

    if !unused_dependencies.is_empty() {
        println!("\n{}", "Unused Dependencies:".red().bold());
        for dep in &unused_dependencies {
            println!("- {}", dep.red());
        }
    } else {
        println!("{}", "\nNo unused dependencies found!".green());
    }

    // prune unused dependencies
    if !unused_dependencies.is_empty() {
        handle_unused_dependencies(&unused_dependencies, dry_run);
    }
}

fn reinstall_modules() {
    println!("{}", "Reinstalling node_modules...".yellow());

    let node_modules_path = Path::new("node_modules");
    if node_modules_path.exists() {
        if let Err(e) = fs::remove_dir_all(node_modules_path) {
            eprintln!("Failed to remove node_modules: {}", e);
            return;
        }
        println!("{}", "Deleted node_modules".green());
    }

    let package_manager = detect_package_manager();
    let result = Command::new(package_manager).arg("install").output();

    match result {
        Ok(output) if output.status.success() => {
            println!("{}", "Reinstallation successful!".green());
        }
        _ => {
            eprintln!("{}", "Failed to reinstall dependencies".red());
        }
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

        // reinstall dependencies
        if !dry_run {
            reinstall_modules();
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

fn find_dependencies_in_content(content: &str, dependencies: &HashSet<String>) -> HashSet<String> {
    let mut found = HashSet::new();
    for dep in dependencies {
        let regex = Regex::new(&format!(r#"[\"']{}[\"']"#, regex::escape(dep))).unwrap();
        let require_regex =
            Regex::new(&format!(r#"require\([\"']{}[\"']\)"#, regex::escape(dep))).unwrap();
        if regex.is_match(content) || require_regex.is_match(content) {
            found.insert(dep.clone());
        }
    }
    found
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
