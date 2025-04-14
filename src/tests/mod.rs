#[cfg(test)]
mod tests {
    use crate::dependency::{get_required_dependencies, read_cnpignore, read_package_json};
    use crate::file_scanner::scan_files;
    use crate::package_manager::detect_package_manager;
    use indicatif::ProgressBar;
    use std::collections::HashSet;
    use std::env;
    use std::fs::{self, File};
    use std::io::{self, Write};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    fn setup_lockfile(temp_dir: &TempDir, lockfile_name: &str) -> io::Result<()> {
        let project_root = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let test_fixtures_dir = project_root.join("test_fixtures");
        if !test_fixtures_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "test_fixtures directory not found",
            ));
        }
        let source_path = test_fixtures_dir.join(lockfile_name);
        if !source_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Lock file {} not found in test_fixtures", lockfile_name),
            ));
        }
        let dest_path = temp_dir.path().join(lockfile_name);
        fs::copy(&source_path, &dest_path)?;
        Ok(())
    }

    fn setup_package_json(temp_dir: &TempDir) -> std::io::Result<()> {
        let package_json_path = temp_dir.path().join("package.json");
        let content = r#"
        {
            "dependencies": {
                "react": "^18.2.0",
                "@vercel/analytics": "^1.0.0",
                "lodash": "^4.17.21"
            },
            "devDependencies": {
                "eslint": "^8.0.0"
            }
        }
        "#;
        File::create(&package_json_path)?.write_all(content.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_package_lock_json() {
        let temp_dir = setup_temp_dir();
        setup_lockfile(&temp_dir, "package-lock-test.json").unwrap();
        setup_package_json(&temp_dir).unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "@vercel/analytics", "lodash", "eslint"]
            .into_iter()
            .map(String::from)
            .collect();
        assert!(
            required.is_superset(&expected),
            "Expected at least {:?}",
            expected
        );
    }

    #[test]
    fn test_yarn_lock() {
        let temp_dir = setup_temp_dir();
        setup_lockfile(&temp_dir, "yarn-test.lock").unwrap();
        setup_package_json(&temp_dir).unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "@vercel/analytics", "lodash", "eslint"]
            .into_iter()
            .map(String::from)
            .collect();
        assert!(
            required.is_superset(&expected),
            "Expected at least {:?}",
            expected
        );
    }

    #[test]
    fn test_pnpm_lock_yaml() {
        let temp_dir = setup_temp_dir();
        setup_lockfile(&temp_dir, "pnpm-lock-test.yaml").unwrap();
        setup_package_json(&temp_dir).unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "@vercel/analytics", "lodash", "eslint"]
            .into_iter()
            .map(String::from)
            .collect();
        assert!(
            required.is_superset(&expected),
            "Expected at least {:?}",
            expected
        );
    }

    #[test]
    fn test_bun_lock() {
        let temp_dir = setup_temp_dir();
        setup_lockfile(&temp_dir, "bun-lock-test.lock").unwrap();
        setup_package_json(&temp_dir).unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = ["react", "@vercel/analytics", "lodash", "eslint"]
            .into_iter()
            .map(String::from)
            .collect();
        assert!(
            required.is_superset(&expected),
            "Expected at least {:?}",
            expected
        );
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
        setup_lockfile(&temp_dir, "package-lock-test.json").unwrap();
        setup_lockfile(&temp_dir, "yarn-test.lock").unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let required = get_required_dependencies();
        let expected: HashSet<String> = HashSet::new();
        assert_eq!(required, expected);
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

    #[test]
    fn test_cnpignore_parsing() {
        let temp_dir = setup_temp_dir();
        let cnpignore_path = temp_dir.path().join(".cnpignore");
        let content = r#"
        # Ignore these
        react
        @vercel/analytics
        
        lodash # inline comment
        "#;
        File::create(&cnpignore_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let ignored = read_cnpignore();
        let expected: HashSet<String> = ["react", "@vercel/analytics", "lodash"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(ignored, expected);
    }

    #[test]
    fn test_empty_cnpignore() {
        let temp_dir = setup_temp_dir();
        let cnpignore_path = temp_dir.path().join(".cnpignore");
        File::create(&cnpignore_path)
            .unwrap()
            .write_all(b"")
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let ignored = read_cnpignore();
        let expected: HashSet<String> = HashSet::new();
        assert_eq!(ignored, expected);
    }

    #[test]
    fn test_missing_cnpignore() {
        let temp_dir = setup_temp_dir();
        std::env::set_current_dir(&temp_dir).unwrap();

        let ignored = read_cnpignore();
        let expected: HashSet<String> = HashSet::new();
        assert_eq!(ignored, expected);
    }

    #[test]
    fn test_file_scanner_finds_dependencies() {
        let temp_dir = setup_temp_dir();
        let js_file_path = temp_dir.path().join("index.js");
        let content = r#"
        import React from 'react';
        import { analytics } from '@vercel/analytics';
        const _ = require('lodash');
        "#;
        File::create(&js_file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let dependencies: HashSet<String> = ["react", "@vercel/analytics", "lodash", "unused"]
            .into_iter()
            .map(String::from)
            .collect();

        std::env::set_current_dir(&temp_dir).unwrap();
        let pb = ProgressBar::new(1);
        let (used_packages, explored_files, ignored_files) = scan_files(&dependencies, &pb);

        let expected_used: HashSet<String> = ["react", "@vercel/analytics", "lodash"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(used_packages, expected_used);
        assert_eq!(explored_files, vec![js_file_path.display().to_string()]);
        assert_eq!(ignored_files, Vec::<String>::new());
    }

    #[test]
    fn test_file_scanner_ignores_folders() {
        let temp_dir = setup_temp_dir();
        let node_modules_file = temp_dir.path().join("node_modules").join("dep.js");
        let src_file = temp_dir.path().join("src.js");
        fs::create_dir(temp_dir.path().join("node_modules")).unwrap();
        File::create(&node_modules_file)
            .unwrap()
            .write_all(b"import 'react';")
            .unwrap();
        File::create(&src_file)
            .unwrap()
            .write_all(b"import 'react';")
            .unwrap();

        let dependencies: HashSet<String> = ["react"].into_iter().map(String::from).collect();

        std::env::set_current_dir(&temp_dir).unwrap();
        let pb = ProgressBar::new(2);
        let (used_packages, explored_files, ignored_files) = scan_files(&dependencies, &pb);

        let expected_used: HashSet<String> = ["react"].into_iter().map(String::from).collect();
        assert_eq!(used_packages, expected_used);
        assert_eq!(explored_files, vec![src_file.display().to_string()]);
        assert_eq!(ignored_files, vec![node_modules_file.display().to_string()]);
    }

    #[test]
    fn test_package_manager_detection() {
        let temp_dir = setup_temp_dir();

        // Test npm (default)
        std::env::set_current_dir(&temp_dir).unwrap();
        assert_eq!(detect_package_manager(), "npm");

        // Test yarn
        File::create(temp_dir.path().join("yarn.lock")).unwrap();
        assert_eq!(detect_package_manager(), "yarn");

        // Test pnpm
        fs::remove_file(temp_dir.path().join("yarn.lock")).unwrap();
        File::create(temp_dir.path().join("pnpm-lock.yaml")).unwrap();
        assert_eq!(detect_package_manager(), "pnpm");

        // Test bun
        fs::remove_file(temp_dir.path().join("pnpm-lock.yaml")).unwrap();
        File::create(temp_dir.path().join("bun.lock")).unwrap();
        assert_eq!(detect_package_manager(), "bun");
    }

    #[test]
    fn test_no_dependencies_in_package_json() {
        let temp_dir = setup_temp_dir();
        let package_json_path = temp_dir.path().join("package.json");
        let content = r#"{"name": "empty-project"}"#;
        File::create(&package_json_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();
        let result = read_package_json("package.json");
        assert!(result.is_ok());
        let value = result.unwrap();
        let dependencies = value
            .get("dependencies")
            .and_then(serde_json::Value::as_object)
            .map_or_else(HashSet::new, |map| {
                map.keys().cloned().collect::<HashSet<String>>()
            });
        assert_eq!(dependencies, HashSet::new());
    }
}
