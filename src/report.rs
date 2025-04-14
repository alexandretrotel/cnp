use crate::config::{EXTENSIONS, IGNORE_FOLDERS, PACKAGE_JSON_PATH};
use colored::*;
use comfy_table::{Cell, Color, Table};
use std::collections::HashSet;

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
