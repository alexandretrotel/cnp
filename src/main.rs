use colored::*;
use comfy_table::{Cell, Color, Table};
use glob::glob;
use indicatif::ProgressBar;
use regex::Regex;
use serde_json::Value;
use std::{collections::HashSet, ffi::OsStr, fs, io, path::Path, process::Command};

const PACKAGE_JSON_PATH: &str = "package.json";
const EXTENSIONS: [&str; 5] = ["js", "ts", "jsx", "tsx", "mdx"];
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

    let pb = ProgressBar::new_spinner();
    pb.set_message("Scanning files...");
    for pattern in &patterns {
        for entry in glob(pattern).expect("Failed to read glob pattern") {
            pb.inc(1);
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
    pb.finish_with_message("Scanning complete!");

    // identify unused dependencies
    let required_deps = get_required_dependencies();
    let ignored_deps = read_cnpignore();
    let unused_dependencies: Vec<_> = dependencies
        .difference(&used_packages)
        .filter(|dep| !required_deps.contains(*dep) && !ignored_deps.contains(*dep))
        .cloned()
        .collect();

    // print unused dependencies
    print_dependency_report(
        &dependencies,
        &used_packages,
        &unused_dependencies,
        &explored_files,
    );

    // confirm deletion
    if !unused_dependencies.is_empty() {
        if dry_run {
            println!("{}", "Dry-run mode: No deletion will occur.".yellow());
        } else {
            if !ask_for_confirmation() {
                println!("{}", "Operation cancelled.".red());
                return;
            }
        }
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
        println!("{}", "\nDry-run mode: Would delete:".yellow());
        for dep in unused_dependencies {
            println!("- {}", dep.yellow());
        }
        return;
    }

    if unused_dependencies.is_empty() {
        return;
    }

    println!("\n{}", "Review Unused Dependencies:".yellow().bold());
    let mut to_delete = Vec::new();
    for dep in unused_dependencies {
        println!("Delete {}? (y/n)", dep.red());
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if input.trim().to_lowercase() == "y" {
            to_delete.push(dep.clone());
        }
    }

    if to_delete.is_empty() {
        println!("{}", "No dependencies selected for deletion.".green());
        return;
    }

    println!("{}", "Deleting selected dependencies...".red().bold());
    for dep in &to_delete {
        if uninstall_dependency(dep, &detect_package_manager()) {
            println!("{} {}", "Deleted:".green(), dep.green());
        } else {
            println!("{} {}", "Failed to delete:".red(), dep.red());
        }
    }

    reinstall_modules();
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

fn get_required_dependencies() -> HashSet<String> {
    // Read package-lock.json
    let mut required = HashSet::new();
    if let Ok(content) = fs::read_to_string("package-lock.json") {
        if let Ok(lock) = serde_json::from_str::<Value>(&content) {
            if let Some(deps) = lock.get("dependencies").and_then(|v| v.as_object()) {
                for (dep, info) in deps {
                    required.insert(dep.clone());
                    if let Some(peer) = info.get("peerDependencies").and_then(|v| v.as_object()) {
                        required.extend(peer.keys().cloned());
                    }
                }
            }
        }
    }

    // Read yarn.lock
    if let Ok(content) = fs::read_to_string("yarn.lock") {
        for line in content.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if let Some(dep) = line.split(':').next() {
                required.insert(dep.trim().to_string());
            }
        }
    }

    // Read pnpm-lock.yaml
    if let Ok(content) = fs::read_to_string("pnpm-lock.yaml") {
        for line in content.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if let Some(dep) = line.split(':').next() {
                required.insert(dep.trim().to_string());
            }
        }
    }

    // Read bun.lockb
    if let Ok(content) = fs::read_to_string("bun.lockb") {
        for line in content.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if let Some(dep) = line.split(':').next() {
                required.insert(dep.trim().to_string());
            }
        }
    }

    // Read bun.lock
    if let Ok(content) = fs::read_to_string("bun.lock") {
        for line in content.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if let Some(dep) = line.split(':').next() {
                required.insert(dep.trim().to_string());
            }
        }
    }

    required
}

fn read_cnpignore() -> HashSet<String> {
    fs::read_to_string(".cnpignore")
        .ok()
        .map(|content| {
            content
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

fn print_dependency_report(
    dependencies: &HashSet<String>,
    used_packages: &HashSet<String>,
    unused_dependencies: &[String],
    explored_files: &[String],
) {
    let mut table = Table::new();
    table.set_header(vec!["Metric", "Value"]);
    table.add_row(vec![Cell::new("Project"), Cell::new(PACKAGE_JSON_PATH)]);
    table.add_row(vec![
        Cell::new("Extensions"),
        Cell::new(EXTENSIONS.join(", ")),
    ]);
    table.add_row(vec![
        Cell::new("Ignored Folders"),
        Cell::new(IGNORE_FOLDERS.join(", ")),
    ]);
    table.add_row(vec![
        Cell::new("Explored Files"),
        Cell::new(explored_files.len().to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Total Dependencies"),
        Cell::new(dependencies.len().to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Used Dependencies"),
        Cell::new(used_packages.len().to_string()).fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Unused Dependencies"),
        Cell::new(unused_dependencies.len().to_string()).fg(Color::Red),
    ]);
    println!("\n{}", "Dependency Usage Report".bold().blue());
    println!("{}", table);

    if !used_packages.is_empty() {
        println!("\n{}", "Used Dependencies:".green().bold());
        for dep in used_packages {
            println!("- {}", dep.green());
        }
    }

    if !unused_dependencies.is_empty() {
        println!("\n{}", "Unused Dependencies:".red().bold());
        println!(
            "{}",
            "Note: Some may be required at runtime (e.g., react-dom).".yellow()
        );
        for dep in unused_dependencies {
            println!("- {}", dep.red());
        }
    } else {
        println!("\n{}", "No unused dependencies found!".green().bold());
    }
}
