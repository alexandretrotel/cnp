mod config;
mod dependency;
mod file_scanner;
mod package_manager;
mod report;
mod uninstall;
mod utils;

use colored::*;
use config::PACKAGE_JSON_PATH;
use dependency::read_package_json;
use file_scanner::scan_files;
use report::print_dependency_report;
use std::collections::HashSet;
use uninstall::handle_unused_dependencies;

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    let interactive =
        args.contains(&"--interactive".to_string()) || args.contains(&"-i".to_string());
    let all = args.contains(&"--all".to_string()) || args.contains(&"-a".to_string());

    // Initialize progress bar
    let pb = utils::create_spinner("Initializing...");

    // Read package.json
    let package_json = read_package_json(PACKAGE_JSON_PATH).unwrap_or_else(|err| {
        eprintln!("{}", err.red());
        std::process::exit(1);
    });

    // Collect dependencies
    let dependencies: HashSet<String> = package_json
        .get("dependencies")
        .and_then(serde_json::Value::as_object)
        .map_or_else(HashSet::new, |map| map.keys().cloned().collect());

    // Scan for used dependencies
    pb.set_message("Scanning files...");
    let (used_packages, explored_files, ignored_files) = scan_files(&dependencies, &pb);

    pb.finish_with_message("Scanning complete!".green().to_string());

    // Identify unused dependencies
    let required_deps = dependency::get_required_dependencies();
    let ignored_deps = dependency::read_cnpignore();
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
