use crate::package_manager::detect_package_manager;
use crate::utils::{create_bar, create_spinner};
use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::fs;
use std::io::{self};
use std::path::Path;
use std::process::Command;

/// Reinstalls the project's `node_modules` directory.
///
/// This function removes the existing `node_modules` directory (if present) and runs the
/// appropriate package manager's install command (e.g., `npm install`, `yarn install`) to
/// reinstall dependencies. A progress spinner provides feedback during the process.
///
/// # Output
///
/// Prints success or failure messages to the console via a progress spinner:
/// - Success: "Reinstallation successful!" (in green).
/// - Failure: An error message (in red) if removal or installation fails.
///
/// # Examples
///
/// ```
/// reinstall_modules();
/// // If `node_modules` exists, it is deleted and reinstalled with the detected package manager.
/// // Outputs a spinner with status messages.
/// ```
pub fn reinstall_modules() {
    let pb = create_spinner("Reinstalling node_modules...");

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

/// Handles the deletion of unused dependencies based on user preferences.
///
/// This function processes unused dependencies, allowing deletion in three modes:
/// - Dry-run: Lists dependencies that would be deleted without making changes.
/// - Interactive: Prompts the user to select dependencies to delete.
/// Successfully deleted dependencies trigger a reinstall of `node_modules`.
///
/// # Arguments
///
/// * `unused_dependencies` - A slice of `String` containing unused dependency names.
/// * `dry_run` - If `true`, simulates deletion without making changes.
/// * `interactive` - If `true`, prompts the user to select dependencies to delete.
///
/// # Output
///
/// Prints to the console:
/// - In dry-run mode: A list of dependencies that would be deleted.
/// - In interactive mode: A selection prompt for dependencies.
/// - Progress bar updates for each deletion attempt (success in green, failure in red).
/// - A final message indicating completion and, if deletions occurred, a reinstallation message.
///
/// # Examples
///
/// ```
/// let unused = vec!["lodash".to_string(), "react".to_string()];
/// handle_unused_dependencies(&unused, true, false);
/// // Prints a dry-run list of dependencies without deleting.
/// // Output: "Dry-run mode: No changes will be made."
/// //         "Would delete:"
/// //         "- lodash"
/// //         "- react"
///
/// handle_unused_dependencies(&unused, false, true);
/// // Prompts interactively to select dependencies for deletion.
/// ```
pub fn handle_unused_dependencies(
    unused_dependencies: &[String],
    dry_run: bool,
    interactive: bool,
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
    } else {
        confirm_all_deletion(unused_dependencies)
    };

    if to_delete.is_empty() {
        println!(
            "\n{}",
            "No dependencies selected for deletion.".yellow().bold()
        );
        return;
    }

    let pb = create_bar(to_delete.len() as u64, "Deleting dependencies...");
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

/// Prompts the user to interactively select dependencies for deletion.
///
/// Displays a multi-select interface where the user can choose which dependencies to delete from
/// the provided list. All dependencies are unselected by default.
///
/// # Arguments
///
/// * `unused_dependencies` - A slice of `String` containing unused dependency names.
///
/// # Returns
///
/// Returns a `Vec<String>` containing the names of dependencies selected for deletion.
/// Returns an empty vector if the user cancels or no selections are made.
///
/// # Examples
///
/// ```
/// let unused = vec!["lodash".to_string(), "react".to_string()];
/// let selected = select_dependencies_interactively(&unused);
/// // Displays a prompt; if user selects "lodash", returns ["lodash"].
/// ```
fn select_dependencies_interactively(unused_dependencies: &[String]) -> Vec<String> {
    println!("\n{}", "Select dependencies to delete:".cyan().bold());

    let defaults = vec![false; unused_dependencies.len()];
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

/// Prompts the user to confirm deletion of all unused dependencies.
///
/// Displays a yes/no prompt asking the user to confirm deleting all provided dependencies.
///
/// # Arguments
///
/// * `unused_dependencies` - A slice of `String` containing unused dependency names.
///
/// # Returns
///
/// Returns a `Vec<String>` containing all dependency names if the user confirms with "y".
/// Returns an empty vector if the user declines or input fails.
///
/// # Examples
///
/// ```
/// let unused = vec!["lodash".to_string(), "react".to_string()];
/// let confirmed = confirm_all_deletion(&unused);
/// // Prompts "Confirm deletion of all unused dependencies? (y/n)".
/// // If user inputs "y", returns ["lodash", "react"]; otherwise, returns [].
/// ```
fn confirm_all_deletion(unused_dependencies: &[String]) -> Vec<String> {
    println!(
        "\n{}",
        "Confirm deletion of all unused dependencies? (y/n)".yellow()
    );

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
        unused_dependencies.to_vec()
    } else {
        Vec::new()
    }
}

/// Uninstalls a single dependency using the specified package manager.
///
/// Executes the package manager's uninstall command (e.g., `npm uninstall <dependency>`) for the
/// given dependency.
///
/// # Arguments
///
/// * `dependency` - The name of the dependency to uninstall.
/// * `package_manager` - The name of the package manager to use (e.g., "npm", "yarn").
///
/// # Returns
///
/// Returns `true` if the uninstall command succeeds, `false` otherwise.
///
/// # Examples
///
/// ```
/// let success = uninstall_dependency("lodash", "npm");
/// if success {
///     println!("Successfully uninstalled lodash");
/// } else {
///     println!("Failed to uninstall lodash");
/// }
/// ```
fn uninstall_dependency(dependency: &str, package_manager: &str) -> bool {
    let command = match package_manager {
        "npm" => "uninstall",
        "pnpm" | "yarn" | "bun" => "remove",
        _ => {
            eprintln!("Unsupported package manager: {}", package_manager);
            return false;
        }
    };

    let output = Command::new(package_manager)
        .args([command, dependency])
        .output();

    matches!(output, Ok(result) if result.status.success())
}
