#[cfg(test)]
mod tests {
    use crate::config::is_typescript_project;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn it_returns_true_for_valid_typescript_project() {
        // Create a temporary directory with a tsconfig.json file
        let temp_dir = TempDir::new().unwrap();

        // Write "tsconfig.json" to the directory
        let tsconfig_path = temp_dir.path().join("tsconfig.json");
        File::create(&tsconfig_path).unwrap();
        println!("Created tsconfig.json at: {:?}", tsconfig_path);

        // Call the function and assert it returns true
        assert!(is_typescript_project(&temp_dir.path().to_string_lossy()));
    }

    #[test]
    fn it_returns_false_for_non_typescript_directory() {
        // Create a temporary directory without a tsconfig.json file
        let temp_dir = TempDir::new().unwrap();

        // Ensure there is no "tsconfig.json" in the directory
        assert!(!temp_dir.path().join("tsconfig.json").exists());

        // Call the function and assert it returns false
        assert!(!is_typescript_project(&temp_dir.path().to_string_lossy()));
    }
}
