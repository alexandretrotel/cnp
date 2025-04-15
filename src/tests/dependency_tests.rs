#[cfg(test)]
mod tests {
    use crate::dependency::{get_required_dependencies, read_package_json};
    use serde_json::Value;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_temp_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        temp_dir
    }

    #[test]
    fn test_read_package_json() {
        let temp_dir = setup_temp_dir();
        let package_json_path = temp_dir.path().join("package.json");
        let content = r#"{
            "name": "test",
            "version": "1.0.0",
            "dependencies": {
                "react": "^18.2.0",
                "lodash": "^4.17.21"
            }
        }"#;
        File::create(&package_json_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let result = read_package_json(package_json_path.to_str().unwrap());
        assert!(result.is_ok(), "Should read package.json successfully");

        let json = result.unwrap();
        let deps = json
            .get("dependencies")
            .and_then(Value::as_object)
            .expect("Dependencies should be an object");
        assert_eq!(
            deps.get("react"),
            Some(&Value::String("^18.2.0".to_string())),
            "Should contain react"
        );
        assert_eq!(
            deps.get("lodash"),
            Some(&Value::String("^4.17.21".to_string())),
            "Should contain lodash"
        );
    }

    #[test]
    fn test_get_required_dependencies() {
        let temp_dir = setup_temp_dir();
        let package_json_path = temp_dir.path().join("package.json");
        let content = r#"{
            "name": "test",
            "version": "1.0.0",
            "dependencies": {
                "react": "^18.2.0",
                "lodash": "^4.17.21"
            },
            "devDependencies": {
                "eslint": "^8.0.0"
            }
        }"#;
        File::create(&package_json_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "lodash", "eslint"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(
            required, expected,
            "Should return dependencies from package.json"
        );

        // Test with lockfile
        let lockfile_path = temp_dir.path().join("package-lock.json");
        let lock_content = r#"{
            "name": "test",
            "version": "1.0.0",
            "lockfileVersion": 3,
            "packages": {
                "": {
                    "dependencies": {
                        "react": "^18.2.0"
                    }
                },
                "node_modules/react": {
                    "version": "18.2.0"
                },
                "node_modules/@vercel/analytics": {
                    "version": "1.0.0"
                }
            }
        }"#;
        File::create(&lockfile_path)
            .unwrap()
            .write_all(lock_content.as_bytes())
            .unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "lodash", "eslint", "@vercel/analytics"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(
            required, expected,
            "Should include dependencies from lockfile"
        );
    }

    #[test]
    fn test_malformed_yaml_lockfile() {
        let temp_dir = setup_temp_dir();
        let lockfile_path = temp_dir.path().join("pnpm-lock.yaml");
        let content = r#"invalid: yaml: structure"#;
        File::create(&lockfile_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = HashSet::new();
        assert_eq!(
            required, expected,
            "Should return empty set for malformed pnpm-lock.yaml"
        );
    }
}
