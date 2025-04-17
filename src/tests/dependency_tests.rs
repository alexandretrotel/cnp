#[cfg(test)]
mod tests {
    use crate::dependency::{get_required_dependencies, read_cnpignore, read_package_json};
    use colored::Colorize;
    use std::io::Write;
    use std::{collections::HashSet, error::Error, fs, path::PathBuf};
    use tempfile::TempDir;

    #[test]
    fn test_read_package_json_success() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("package.json");

        // Move the `package.json` file from test_fixtures/ to the temporary directory
        fs::copy("test_fixtures/package.json", &package_path)
            .expect("Failed to copy package.json to temporary directory");

        // Test the function with the path to the temporary package.json file
        match read_package_json(package_path.to_str().unwrap()) {
            Ok(json) => {
                assert_eq!(json["name"].as_str(), Some("test-node-project"));
                assert_eq!(json["version"].as_str(), Some("1.0.0"));
            }
            Err(e) => panic!("Expected success, but got error: {}", e),
        }
    }

    #[test]
    fn test_read_package_json_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_path = temp_dir.path().join("non-existent.json");

        // Test the function with a path that does not exist
        match read_package_json(non_existent_path.to_str().unwrap()) {
            Ok(_) => panic!("Expected an error, but got success."),
            Err(e) => {
                assert!(e.contains("Error: `non-existent.json` not found."));
            }
        }
    }

    #[test]
    fn test_read_package_json_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("invalid.json");

        // Create a file with invalid JSON content
        let invalid_content = "this is not valid JSON";
        fs::write(&invalid_path, invalid_content).unwrap();

        // Test the function with an invalid JSON file
        match read_package_json(invalid_path.to_str().unwrap()) {
            Ok(_) => panic!("Expected an error, but got success."),
            Err(e) => {
                assert!(e.contains("Error: Invalid JSON in package.json."));
            }
        }
    }

    #[test]
    fn test_get_required_dependencies_single_package_json() {
        // TODO: handle devDependencies logic
        // Create a temporary directory and package.json file
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("package.json");

        let content = r#"{
            "name": "example-package",
            "version": "1.0.0",
            "dependencies": {
                "dep1": "^1.0.0"
            },
            "devDependencies": {
                "test-dep": "^2.0.0"
            }
        }"#;

        fs::write(&package_path, content).unwrap();

        // Check that only the dependencies are returned
        let deps = get_required_dependencies(&temp_dir.path().to_str().unwrap());
        println!("Dependencies: {:?}", deps);
        assert!(!deps.is_empty());
        assert_eq!(deps.len(), 2);
        assert!(deps.contains("dep1"));
        assert!(deps.contains("test-dep"));
    }

    #[test]
    fn test_get_required_dependencies_no_package_json() {
        // Create a temporary directory without package.json
        let temp_dir = TempDir::new().unwrap();

        // Check that an empty set is returned
        let deps = get_required_dependencies(temp_dir.path().to_str().unwrap());
        assert!(deps.is_empty());
    }

    #[test]
    fn test_get_required_dependencies_invalid_package_json() {
        // Create a temporary directory with an invalid package.json file
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("package.json");

        fs::write(&package_path, "invalid json").expect("Failed to write invalid JSON");

        // Check that an empty set is returned
        let deps = get_required_dependencies(temp_dir.path().to_str().unwrap());
        assert!(deps.is_empty());
    }

    #[test]
    fn test_get_required_dependencies_multiple_lockfiles() {
        // Create a temporary directory with multiple lockfiles and check for warning
        let temp_dir = TempDir::new().unwrap();

        // Move the `package.json` file from test_fixtures/ to the temporary directory
        let package_path = temp_dir.path().join("package.json");
        fs::copy("test_fixtures/package.json", &package_path)
            .expect("Failed to copy package.json to temporary directory");

        // Move the `package-lock.json` file from test_fixtures/ to the temporary directory
        let lockfile1_path = temp_dir.path().join("package-lock.json");
        fs::copy("test_fixtures/package-lock.json", &lockfile1_path)
            .expect("Failed to copy package-lock.json to temporary directory");

        // Move the `yarn.lock` file from test_fixtures/ to the temporary directory
        let lockfile2_path = temp_dir.path().join("yarn.lock");
        fs::copy("test_fixtures/yarn.lock", &lockfile2_path)
            .expect("Failed to copy yarn.lock to temporary directory");

        // Check that an empty set is returned and a warning is printed
        let deps = get_required_dependencies(temp_dir.path().to_str().unwrap());
        assert!(deps.is_empty());
        eprintln!(
            "\n{}: Multiple lockfiles detected ({}). Please use only one package manager.",
            "Warning".yellow().bold(),
            "package-lock.json, yarn.lock"
        );
    }

    #[test] // TODO: need to handle packages too
    fn test_get_required_dependencies_package_lock_json() {
        // Create a temporary directory with package-lock.json
        let temp_dir = TempDir::new().unwrap();
        let lockfile_path = temp_dir.path().join("package-lock.json");
        let content = r#"{
            "packages": {
                "node_modules/dep1": { "version": "1.0.0" }
            }
        }"#;
        fs::write(&lockfile_path, content).expect("Failed to write package-lock.json");

        // Check that only the dependencies are returned
        let deps = get_required_dependencies(temp_dir.path().to_str().unwrap());

        assert!(!deps.is_empty());
        assert_eq!(deps.len(), 1);
        assert!(deps.contains("dep1"));
    }

    #[test]
    fn test_get_required_dependencies_yarn_lock() {
        // Create a temporary directory with yarn.lock
        let temp_dir = TempDir::new().unwrap();
        let lockfile_path = temp_dir.path().join("yarn.lock");

        let content = r#"
        # THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
        # yarn lockfile v1
        
        package-1@^1.0.0:
          version "1.0.3"
          resolved "https://registry.npmjs.org/package-1/-/package-1-1.0.3.tgz#a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0"
          
        package-2@^2.0.0:
          version "2.0.1"
          resolved "https://registry.npmjs.org/package-2/-/package-2-2.0.1.tgz#a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0"
          dependencies:
            package-4 "^4.0.0"
        
        package-3@^3.0.0:
          version "3.1.9"
          resolved "https://registry.npmjs.org/package-3/-/package-3-3.1.9.tgz#a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0"
          dependencies:
            package-4 "^4.5.0"
        
        package-4@^4.0.0, package-4@^4.5.0:
          version "4.6.3"
          resolved "https://registry.npmjs.org/package-4/-/package-4-2.6.3.tgz#a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0"
        "#;

        // Write the content to yarn.lock file
        fs::write(&lockfile_path, content).expect("Failed to write yarn.lock");

        // Check that only the dependencies are returned
        let deps = get_required_dependencies(temp_dir.path().to_str().unwrap());

        assert_eq!(deps.len(), 4);
        assert!(deps.contains("package-1"));
        assert!(deps.contains("package-2"));
        assert!(deps.contains("package-3"));
        assert!(deps.contains("package-4"));
    }

    #[test] // TODO: need to handle packages too
    fn test_get_required_dependencies_pnpm_lock_yaml() {
        // Create a temporary directory with pnpm-lock.yaml
        let temp_dir = TempDir::new().unwrap();
        let lockfile_path = temp_dir.path().join("pnpm-lock.yaml");
        let content = r#"dependencies:
        "@dep1":
            version: "1.0.0"
            resolved: "http://registry.example.com/@dep1/-/dep1-1.0.0.tgz"

        devDependencies:
        "@test-dep":
            version: "2.0.0"
        "#;
        fs::write(&lockfile_path, content).expect("Failed to write pnpm-lock.yaml");

        // Check that both dependencies and devDependencies are returned
        let deps = get_required_dependencies(temp_dir.path().to_str().unwrap());

        assert!(!deps.is_empty());
        assert_eq!(deps.len(), 2);
        assert!(deps.contains("@dep1"));
    }

    #[test]
    fn test_get_required_dependencies_bun_lock() {
        // Create a temporary directory with bun.lock
        let temp_dir = TempDir::new().unwrap();
        let lockfile_path = temp_dir.path().join("bun.lock");
        let content = r#"{
        "lockfileVersion": 1,
        "workspaces": {
            "": {
                "name": "test-node-project",
                "dependencies": {
                    "@dep1": "^1.0.0",
                },
                "devDependencies": {
                    "@test-dep": "^2.0.0",
                }
            }
        }"#;
        fs::write(&lockfile_path, content).expect("Failed to write bun.lock");

        // Check that both dependencies and devDependencies are returned
        let deps = get_required_dependencies(temp_dir.path().to_str().unwrap());

        assert!(!deps.is_empty());
        assert_eq!(deps.len(), 2);
        assert!(deps.contains("@dep1"));
    }

    #[test]
    fn test_read_cnpignore_with_valid_patterns() -> Result<(), Box<dyn Error>> {
        // Create a temporary directory and file
        let temp_dir = TempDir::new().unwrap();
        let file_path = PathBuf::from(temp_dir.path()).join(".cnpignore");
        let mut file = fs::File::create(file_path.clone()).unwrap();

        // Write valid patterns to the file
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "pattern1").unwrap();
        writeln!(file, "").unwrap(); // Empty line
        writeln!(file, "  pattern2 ").unwrap();
        writeln!(file, "pattern3# This is an inline comment").unwrap();

        // Read the patterns and assert they match expected values
        let ignore_patterns = read_cnpignore(&file_path.to_str().unwrap());
        let expected_patterns = HashSet::from([
            "pattern1".to_string(),
            "pattern2".to_string(),
            "pattern3".to_string(),
        ]);

        assert_eq!(ignore_patterns, expected_patterns);

        Ok(())
    }
}
