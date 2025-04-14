#[cfg(test)]
mod tests {
    use crate::dependency::get_required_dependencies;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    #[test]
    fn test_package_lock_json() {
        let temp_dir = setup_temp_dir();
        let lockfile_path = temp_dir.path().join("package-lock.json");
        let content = r#"
        {
            "dependencies": {
                "react": {"version": "18.2.0"},
                "@vercel/analytics": {"version": "1.0.0"}
            }
        }
        "#;
        File::create(&lockfile_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "@vercel/analytics"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(required, expected);
    }

    #[test]
    fn test_yarn_lock() {
        let temp_dir = setup_temp_dir();
        let lockfile_path = temp_dir.path().join("yarn.lock");
        let content = r#"
        react@18.2.0:
          version "18.2.0"
        "@vercel/analytics@1.0.0":
          version "1.0.0"
        "#;
        File::create(&lockfile_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "@vercel/analytics"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(required, expected);
    }

    #[test]
    fn test_pnpm_lock_yaml() {
        let temp_dir = setup_temp_dir();
        let lockfile_path = temp_dir.path().join("pnpm-lock.yaml");
        let content = r#"
        packages:
          /react/18.2.0:
            version: 18.2.0
          /@vercel/analytics/1.0.0:
            version: 1.0.0
        "#;
        File::create(&lockfile_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "@vercel/analytics"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(required, expected);
    }

    #[test]
    fn test_bun_lock() {
        let temp_dir = setup_temp_dir();
        let lockfile_path = temp_dir.path().join("bun.lock");
        let content = r#"
        {
            "packages": {
                "react": "18.2.0",
                "@vercel/analytics": "1.0.0"
            }
        }
        "#;
        File::create(&lockfile_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "@vercel/analytics"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(required, expected);
    }

    #[test]
    fn test_missing_lockfiles() {
        let temp_dir = setup_temp_dir();
        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = HashSet::new();
        assert_eq!(required, expected);
    }

    #[test]
    fn test_malformed_package_lock_json() {
        let temp_dir = setup_temp_dir();
        let lockfile_path = temp_dir.path().join("package-lock.json");
        let content = r#"{ invalid json }"#;
        File::create(&lockfile_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = HashSet::new();
        assert_eq!(required, expected);
    }

    #[test]
    fn test_multiple_lockfiles() {
        let temp_dir = setup_temp_dir();

        let package_lock_path = temp_dir.path().join("package-lock.json");
        let package_lock_content = r#"
    {
        "dependencies": {
            "react": {"version": "18.2.0"}
        }
    }
    "#;
        File::create(&package_lock_path)
            .unwrap()
            .write_all(package_lock_content.as_bytes())
            .unwrap();

        let yarn_lock_path = temp_dir.path().join("yarn.lock");
        let yarn_lock_content = r#"
    @vercel/analytics@1.0.0:
      version "1.0.0"
    "#;
        File::create(&yarn_lock_path)
            .unwrap()
            .write_all(yarn_lock_content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = HashSet::new();
        assert_eq!(
            required, expected,
            "Expected empty HashSet when multiple lockfiles are present"
        );
    }

    #[test]
    fn test_empty_lockfile() {
        let temp_dir = setup_temp_dir();
        let lockfile_path = temp_dir.path().join("package-lock.json");
        let content = r#"{}"#;
        File::create(&lockfile_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = HashSet::new();
        assert_eq!(required, expected);
    }
}
