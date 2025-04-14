use crate::package_manager::detect_package_manager;
use crate::utils::{create_bar, create_spinner};
use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::fs;
use std::io::{self};
use std::path::Path;
use std::process::Command;

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

pub fn handle_unused_dependencies(
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

fn uninstall_dependency(dependency: &str, package_manager: &str) -> bool {
    let output = Command::new(package_manager)
        .args(["uninstall", dependency])
        .output();

    matches!(output, Ok(result) if result.status.success())
}
