#[cfg(test)]
mod tests {
    use crate::dependency::read_package_json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_read_package_json_success() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("package.json");

        // Move the `package.json` file from test_fixtures/ to the temporary directory
        fs::copy(
            "test_fixtures/package.json",
            &package_path,
        )
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
}
