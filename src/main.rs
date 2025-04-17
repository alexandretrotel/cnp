mod config;
mod dependency;
mod file_scanner;
mod package_manager;
mod report;
mod uninstall;
mod utils;

#[cfg(test)]
mod tests;

use clap::{Arg, ArgAction, Command};
use colored::*;
use config::PACKAGE_JSON_PATH;
use dependency::read_package_json;
use file_scanner::scan_files;
use report::print_dependency_report;
use std::collections::HashSet;
use uninstall::handle_unused_dependencies;

/// Entry point for the dependency analysis tool.
///
/// This function orchestrates the process of analyzing a project's dependencies by:
/// - Parsing command-line arguments to determine modes (`--dry-run`, `--interactive`, `--all`).
/// - Reading the `package.json` file to extract dependencies.
/// - Scanning project files to identify used dependencies.
/// - Comparing used and declared dependencies to find unused ones, respecting required and ignored dependencies.
/// - Printing a dependency report.
/// - Handling unused dependencies (e.g., prompting for removal) based on the provided flags.
///
/// The program exits with a status code of 1 if `package.json` cannot be read or parsed.
/// A progress bar provides visual feedback during initialization and file scanning.
///
/// # Command-line Arguments
///
/// - `--dry-run`: Simulates actions without making changes (e.g., no uninstalls).
/// - `--interactive` or `-i`: Prompts the user before taking actions on unused dependencies.
/// - `--all` or `-a`: Automatically processes all unused dependencies without prompting.
///
/// # Examples
///
/// ```bash
/// # Run the tool in default mode
/// cargo run
///
/// # Run in dry-run mode to simulate actions
/// cargo run -- --dry-run
///
/// # Run in interactive mode to confirm actions
/// cargo run -- --interactive
///
/// # Process all unused dependencies automatically
/// cargo run -- --all
/// ```
fn main() {
    // Parse command-line arguments
    let matches = Command::new("Check Node Packages")
        .about("A utility tool written in Rust to check unused node packages.")
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Simulate actions without making changes (e.g., no uninstalls)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Prompt the user before taking actions on unused dependencies")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    // Parse the arguments
    let dry_run: bool = *matches.get_one("dry-run").unwrap_or(&false);
    let interactive: bool = *matches.get_one("interactive").unwrap_or(&false);

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
    let dir_path = std::env::current_dir().unwrap_or_default();
    let required_deps = dependency::get_required_dependencies(dir_path.to_str().unwrap());
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
        handle_unused_dependencies(&unused_dependencies, dry_run, interactive);
    }
}
