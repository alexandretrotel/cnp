use crate::config::{EXTENSIONS, IGNORE_FOLDERS, PACKAGE_JSON_PATH};
use colored::*;
use comfy_table::{Cell, Color, Table};
use std::collections::HashSet;

/// Prints a formatted dependency usage report to the console.
///
/// This function generates a tabular report summarizing dependency analysis results, including
/// project details, file scanning metrics, and dependency usage. It also lists used and unused
/// dependencies with color-coded formatting. If no unused dependencies are found, a success message
/// is displayed. A note about potential runtime-required dependencies (e.g., `react-dom`) is included
/// when unused dependencies are listed.
///
/// # Arguments
///
/// * `dependencies` - A reference to a `HashSet<String>` containing all declared dependencies.
/// * `used_packages` - A reference to a `HashSet<String>` containing dependencies found in use.
/// * `unused_dependencies` - A slice of `String` containing unused dependency names.
/// * `explored_files` - A slice of `String` containing paths of explored files.
/// * `ignored_files` - A slice of `String` containing paths of ignored files.
///
/// # Output
///
/// Prints to the console:
/// - A table with metrics (project path, extensions, ignored folders, file counts, dependency counts).
/// - A sorted list of used dependencies (in green).
/// - A sorted list of unused dependencies (in red) with a warning about runtime requirements, or a
///   success message if none are found.
///
/// # Examples
///
/// ```
/// let dependencies: HashSet<String> = ["lodash", "react"].into_iter().map(String::from).collect();
/// let used_packages: HashSet<String> = ["lodash"].into_iter().map(String::from).collect();
/// let unused_dependencies = vec!["react".to_string()];
/// let explored_files = vec!["src/index.js".to_string()];
/// let ignored_files = vec!["node_modules/lodash/index.js".to_string()];
///
/// print_dependency_report(
///     &dependencies,
///     &used_packages,
///     &unused_dependencies,
///     &explored_files,
///     &ignored_files,
/// );
/// // Prints a table with metrics, followed by:
/// // Used Dependencies:
/// // - lodash (in green)
/// // Unused Dependencies:
/// // Note: Some may be required at runtime (e.g., react-dom).
/// // - react (in red)
/// ```
pub fn print_dependency_report(
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
