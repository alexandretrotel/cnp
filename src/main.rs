use colored::*;
use comfy_table::{Cell, Color, Table};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
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
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    let interactive =
        args.contains(&"--interactive".to_string()) || args.contains(&"-i".to_string());
    let all = args.contains(&"--all".to_string()) || args.contains(&"-a".to_string());

    // Initialize progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Initializing...");

    // Read package.json
    let package_json: Value = match fs::read_to_string(PACKAGE_JSON_PATH) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| {
            eprintln!("{}", "Error: Invalid JSON in package.json.".red());
            std::process::exit(1);
        }),
        Err(_) => {
            eprintln!("{}", "Error: `package.json` not found.".red());
            std::process::exit(1);
        }
    };

    // Collect dependencies
    let dependencies = package_json
        .get("dependencies")
        .and_then(Value::as_object)
        .map_or_else(HashSet::new, |map| {
            map.keys().cloned().collect::<HashSet<_>>()
        });

    // Scan for used dependencies
    pb.set_message("Scanning files...");
    let (used_packages, explored_files, ignored_files) = scan_files(&dependencies, &pb);

    pb.finish_with_message("Scanning complete!".green().to_string());

    // Identify unused dependencies
    let required_deps = get_required_dependencies();
    let ignored_deps = read_cnpignore();
    let unused_dependencies: Vec<_> = dependencies
        .difference(&used_packages)
        .filter(|dep| !required_deps.contains(*dep) && !ignored_deps.contains(*dep))
        .cloned()
        .collect();

    // Print report
    print_dependency_report(
        &dependencies,
        &used_packages,
        &unused_dependencies,
        &explored_files,
        &ignored_files,
    );

    // Process unused dependencies
    if !unused_dependencies.is_empty() {
        handle_unused_dependencies(&unused_dependencies, dry_run, interactive, all);
    } else {
        println!("\n{}", "No unused dependencies to process.".green().bold());
    }
}

fn scan_files(
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

fn reinstall_modules() {
    let pb = ProgressBar::new_spinner();
    pb.set_message("Reinstalling node_modules...");

    let node_modules_path = Path::new("node_modules");
    if node_modules_path.exists() {
        if let Err(e) = fs::remove_dir_all(node_modules_path) {
            pb.abandon_with_message(
                format!("Failed to remove node_modules: {}", e)
                    .red()
                    .to_string(),
            );
            return;
        }
    }

    let package_manager = detect_package_manager();
    let result = Command::new(&package_manager).arg("install").output();

    match result {
        Ok(output) if output.status.success() => {
            pb.finish_with_message("Reinstallation successful!".green().to_string());
        }
        _ => {
            pb.abandon_with_message("Failed to reinstall dependencies".red().to_string());
        }
    }
}

fn handle_unused_dependencies(
    unused_dependencies: &[String],
    dry_run: bool,
    interactive: bool,
    all: bool,
) {
    if dry_run {
        println!(
            "\n{}",
            "Dry-run mode: No changes will be made.".yellow().bold()
        );
        println!("{}", "Would delete:".yellow());
        for dep in unused_dependencies {
            println!("- {}", dep.yellow());
        }
        return;
    }

    let package_manager = detect_package_manager();
    let to_delete = if interactive {
        select_dependencies_interactively(unused_dependencies)
    } else if all {
        confirm_all_deletion(unused_dependencies)
    } else {
        println!(
            "\nUse {} or {} to delete unused dependencies.",
            "--interactive (-i)".cyan(),
            "--all (-a)".cyan()
        );
        return;
    };

    if to_delete.is_empty() {
        println!(
            "\n{}",
            "No dependencies selected for deletion.".yellow().bold()
        );
        return;
    }

    let pb = ProgressBar::new(to_delete.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap(),
    );
    pb.set_message("Deleting dependencies...");

    let mut deleted = Vec::new();
    for dep in &to_delete {
        pb.inc(1);
        if uninstall_dependency(dep, &package_manager) {
            pb.set_message(format!("Deleted: {}", dep).green().to_string());
            deleted.push(dep.clone());
        } else {
            pb.set_message(format!("Failed to delete: {}", dep).red().to_string());
        }
        pb.tick();
    }
    pb.finish_with_message("Deletion complete!".green().to_string());

    if !deleted.is_empty() {
        reinstall_modules();
    }
}

fn select_dependencies_interactively(unused_dependencies: &[String]) -> Vec<String> {
    println!("\n{}", "Select dependencies to delete:".cyan().bold());
    let defaults = vec![true; unused_dependencies.len()];
    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .items(unused_dependencies)
        .defaults(&defaults)
        .with_prompt("Use arrow keys and space to select, Enter to confirm")
        .interact_opt()
        .unwrap_or(None);

    match selection {
        Some(indices) => indices
            .into_iter()
            .map(|i| unused_dependencies[i].clone())
            .collect(),
        None => Vec::new(),
    }
}

fn confirm_all_deletion(unused_dependencies: &[String]) -> Vec<String> {
    println!(
        "\n{}",
        "Confirm deletion of all unused dependencies? (y/n)".yellow()
    );
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    if input.trim().to_lowercase() == "y" {
        unused_dependencies.to_vec()
    } else {
        Vec::new()
    }
}

fn detect_package_manager() -> String {
    if Path::new("pnpm-lock.yaml").exists() {
        "pnpm".to_string()
    } else if Path::new("yarn.lock").exists() {
        "yarn".to_string()
    } else if Path::new("bun.lockb").exists() || Path::new("bun.lock").exists() {
        "bun".to_string()
    } else {
        "npm".to_string()
    }
}

fn find_dependencies_in_content(content: &str, dependencies: &HashSet<String>) -> HashSet<String> {
    let mut found = HashSet::new();
    for dep in dependencies {
        let import_regex = Regex::new(&format!(
            r#"import\s+.*?\s+from\s+[\'"]{}[\'"]"#,
            regex::escape(dep)
        ))
        .unwrap();
        let require_regex = Regex::new(&format!(
            r#"require\s*\(\s*[\'"]{}[\'"]\s*\)"#,
            regex::escape(dep)
        ))
        .unwrap();
        if import_regex.is_match(content) || require_regex.is_match(content) {
            found.insert(dep.clone());
        }
    }
    found
}

fn uninstall_dependency(dependency: &str, package_manager: &str) -> bool {
    let output = Command::new(package_manager)
        .args(["uninstall", dependency])
        .output();

    matches!(output, Ok(result) if result.status.success())
}

fn should_ignore(path: &Path) -> bool {
    path.components().any(|component| {
        IGNORE_FOLDERS
            .iter()
            .any(|folder| component.as_os_str() == OsStr::new(folder))
    })
}

fn get_required_dependencies() -> HashSet<String> {
    let mut required = HashSet::new();

    // Package-lock.json
    if let Ok(content) = fs::read_to_string("package-lock.json") {
        if let Ok(lock) = serde_json::from_str::<Value>(&content) {
            if let Some(deps) = lock.get("dependencies").and_then(Value::as_object) {
                required.extend(deps.keys().cloned());
            }
        }
    }

    // Yarn.lock
    if let Ok(content) = fs::read_to_string("yarn.lock") {
        for line in content.lines() {
            if line.ends_with(':') && !line.starts_with('#') {
                let dep = line.trim_end_matches(':').trim();
                if let Some(package_name) = dep.split('@').next() {
                    required.insert(package_name.to_string());
                }
            }
        }
    }

    // Pnpm-lock.yaml
    if let Ok(content) = fs::read_to_string("pnpm-lock.yaml") {
        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(packages) = yaml.get("packages").and_then(|v| v.as_mapping()) {
                for key in packages.keys() {
                    if let Some(key_str) = key.as_str() {
                        let package_name = key_str
                            .split('/')
                            .nth(1)
                            .unwrap_or(key_str)
                            .split('@')
                            .next()
                            .unwrap_or(key_str)
                            .to_string();
                        required.insert(package_name);
                    }
                }
            }
        }
    }

    // Bun.lock
    if let Ok(content) = fs::read_to_string("bun.lock") {
        if let Ok(lock) = serde_json::from_str::<Value>(&content) {
            if let Some(packages) = lock.get("packages").and_then(Value::as_object) {
                required.extend(packages.keys().cloned());
            }
        }
    }

    required
}

fn read_cnpignore() -> HashSet<String> {
    fs::read_to_string(".cnpignore")
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
    ignored_files: &[String],
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
        Cell::new("Ignored Files"),
        Cell::new(ignored_files.len().to_string()),
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
        let mut used = used_packages.iter().collect::<Vec<_>>();
        used.sort();
        for dep in used {
            println!("- {}", dep.green());
        }
    }

    if !unused_dependencies.is_empty() {
        println!("\n{}", "Unused Dependencies:".red().bold());
        println!(
            "{}",
            "Note: Some may be required at runtime (e.g., react-dom).".yellow()
        );
        let mut unused = unused_dependencies.to_vec();
        unused.sort();
        for dep in unused {
            println!("- {}", dep.red());
        }
    } else {
        println!("\n{}", "No unused dependencies found!".green().bold());
    }
}
