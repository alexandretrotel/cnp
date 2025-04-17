#[cfg(test)]
mod tests {
    use crate::file_scanner::{get_typescript_unused_imports, normalize_path};
    use serde_json::json;
    use std::io::Write;
    use std::{
        collections::HashSet,
        error::Error,
        fs,
        path::{Path, PathBuf},
    };
    use tempfile::TempDir;

    #[test]
    fn test_normalize_path_normal_case() -> Result<(), Box<dyn Error>> {
        // Normal path without /private prefix
        let temp_dir = TempDir::new().unwrap();
        let file_path = PathBuf::from(temp_dir.path()).join("file.txt");

        fs::write(&file_path, "Content").unwrap();

        let normalized_path = normalize_path(&file_path);
        let expected_path = file_path.display().to_string();
        assert_eq!(normalized_path, expected_path);

        Ok(())
    }

    #[test]
    fn test_normalize_path_with_private_prefix() -> Result<(), Box<dyn Error>> {
        // Path with /private prefix
        let temp_dir = TempDir::new().unwrap();
        let file_path = PathBuf::from(temp_dir.path()).join("file.txt");

        fs::write(&file_path, "Content").unwrap();

        let normalized_path = normalize_path(&Path::new(
            &("/private/".to_string() + &file_path.display().to_string()),
        ));
        let expected_path = file_path.display().to_string(); // On macOS
        assert_eq!(normalized_path, expected_path);

        Ok(())
    }

    #[test]
    fn test_normalize_path_canonicalization_failure() -> Result<(), Box<dyn Error>> {
        // Path where canonicalization fails (simulating a non-existent path)
        let non_existent_path = PathBuf::from("/nonexistent/path/to/file.txt");

        let normalized_path = normalize_path(&non_existent_path);
        let expected_path = "/nonexistent/path/to/file.txt"; // Original path
        assert_eq!(normalized_path, expected_path);

        Ok(())
    }

    #[test]
    fn test_normalize_path_cross_platform() -> Result<(), Box<dyn Error>> {
        // Ensure the function behaves correctly on different platforms
        if cfg!(target_os = "macos") {
            // Path with /private prefix on macOS
            let temp_dir = TempDir::new().unwrap();
            let file_path = PathBuf::from(temp_dir.path()).join("file.txt");

            fs::write(&file_path, "Content").unwrap();

            let normalized_path = normalize_path(&Path::new(
                &("/private/".to_string() + &file_path.display().to_string()),
            ));
            let expected_path = file_path.display().to_string(); // On macOS

            assert_eq!(normalized_path, expected_path);
        } else {
            // For other platforms, no /private prefix should be present
            let temp_dir = TempDir::new().unwrap();
            let file_path = PathBuf::from(temp_dir.path()).join("file.txt");

            fs::write(&file_path, "Content").unwrap();

            let normalized_path = normalize_path(&Path::new(
                &("/tmp/".to_string() + &file_path.display().to_string()),
            ));
            let expected_path = file_path.display().to_string();

            assert_eq!(normalized_path, expected_path);
        }

        Ok(())
    }

    #[test]
    fn test_get_typescript_unused_imports_valid_project() -> Result<(), Box<dyn Error>> {
        // Create a temporary directory and package.json file
        let temp_dir = TempDir::new().unwrap();
        let package_json_path = PathBuf::from(temp_dir.path()).join("package.json");
        fs::write(
            &package_json_path,
            json!({
                "scripts": {"tsc": "tsc"}
            })
            .to_string(),
        )
        .unwrap();

        // Write the tsconfig.json file
        let tsconfig_json_path = PathBuf::from(temp_dir.path()).join("tsconfig.json");
        fs::write(&tsconfig_json_path, "")?;

        // Create a TypeScript file with unused imports
        let ts_file_path = PathBuf::from(temp_dir.path()).join("src").join("index.ts");
        fs::create_dir_all(ts_file_path.parent().unwrap()).unwrap();

        let mut ts_file = fs::File::create(&ts_file_path).unwrap();
        writeln!(ts_file, "import analytics from 'analytics';").unwrap(); // Unused import
        writeln!(ts_file, "function main() {{}}").unwrap();

        // Execute the function and check results
        let unused_imports = get_typescript_unused_imports(&temp_dir.path().to_str().unwrap());
        let expected_imports = HashSet::from(["analytics".to_string()]);
        assert_eq!(unused_imports, expected_imports);

        Ok(())
    }
}
