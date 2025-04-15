#[cfg(test)]
mod tests {
    use crate::dependency::{get_required_dependencies, read_cnpignore, read_package_json};
    use crate::file_scanner::scan_files;
    use crate::uninstall::handle_unused_dependencies;
    use indicatif::ProgressBar;
    use serde_json::Value;
    use std::collections::HashSet;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_temp_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        temp_dir
    }

    #[test]
    fn test_dry_run_no_modifications() {
        let temp_dir = setup_temp_dir();
        let package_json_path = temp_dir.path().join("package.json");
        let content = r#"{
            "name": "test-dry-run",
            "version": "1.0.0",
            "dependencies": {
                "react": "^18.2.0",
                "lodash": "^4.17.21",
                "@vercel/analytics": "^1.0.0"
            }
        }"#;
        File::create(&package_json_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let project_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_fixtures_dir = PathBuf::from(project_root).join("test_fixtures");
        let unused_ts_src = test_fixtures_dir.join("unused.ts");
        let tsconfig_src = test_fixtures_dir.join("tsconfig.json");
        let unused_ts_dest = temp_dir.path().join("unused.ts");
        let tsconfig_dest = temp_dir.path().join("tsconfig.json");
        fs::copy(&unused_ts_src, &unused_ts_dest).unwrap();
        fs::copy(&tsconfig_src, &tsconfig_dest).unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();
        let pb = ProgressBar::new(1);

        let package_json = read_package_json("package.json").unwrap();
        let dependencies: HashSet<String> = package_json
            .get("dependencies")
            .and_then(Value::as_object)
            .map_or_else(HashSet::new, |map| map.keys().cloned().collect());

        // Case 1: Predefined unused dependencies
        let unused_dependencies = vec!["lodash".to_string(), "@vercel/analytics".to_string()];
        handle_unused_dependencies(&unused_dependencies, true, false, false);

        let package_json_after = read_package_json("package.json").unwrap();
        let dependencies_after: HashSet<String> = package_json_after
            .get("dependencies")
            .and_then(Value::as_object)
            .map_or_else(HashSet::new, |map| map.keys().cloned().collect());
        let expected_deps: HashSet<String> = ["react", "lodash", "@vercel/analytics"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(
            dependencies_after, expected_deps,
            "Dry-run should not modify package.json"
        );

        // Case 2: Scan and identify unused dependencies
        let (used_packages, explored_files, ignored_files) = scan_files(&dependencies, &pb);
        let required_deps = get_required_dependencies();
        let ignored_deps = read_cnpignore();
        let unused_dependencies: Vec<String> = dependencies
            .difference(&used_packages)
            .filter(|dep| !required_deps.contains(*dep) && !ignored_deps.contains(*dep))
            .cloned()
            .collect();

        handle_unused_dependencies(&unused_dependencies, true, false, false);

        let package_json_final = read_package_json("package.json").unwrap();
        let dependencies_final: HashSet<String> = package_json_final
            .get("dependencies")
            .and_then(Value::as_object)
            .map_or_else(HashSet::new, |map| map.keys().cloned().collect());
        assert_eq!(
            dependencies_final, expected_deps,
            "Dry-run with scanned dependencies should not modify package.json"
        );

        let mut expected_unused: Vec<String> =
            vec!["lodash".to_string(), "@vercel/analytics".to_string()];
        expected_unused.sort();
        let mut actual_unused = unused_dependencies.clone();
        actual_unused.sort();
        assert_eq!(
            actual_unused, expected_unused,
            "Should flag lodash and @vercel/analytics as unused"
        );
        assert_eq!(
            explored_files,
            vec![unused_ts_dest.display().to_string()],
            "Should explore unused.ts"
        );
        assert_eq!(
            ignored_files,
            Vec::<String>::new(),
            "No files should be ignored"
        );
    }

    #[test]
    fn test_unused_dependency_detection() {
        struct TestCase {
            name: &'static str,
            package_json_content: &'static str,
            expected_unused: Vec<&'static str>,
            expected_used: Vec<&'static str>,
        }

        let test_cases = vec![
            TestCase {
                name: "single_unused",
                package_json_content: r#"{
                "name": "test-unused",
                "version": "1.0.0",
                "dependencies": {
                    "@vercel/analytics": "^1.0.0"
                }
            }"#,
                expected_unused: vec!["@vercel/analytics"],
                expected_used: vec![],
            },
            TestCase {
                name: "mixed_used_and_unused",
                package_json_content: r#"{
                "name": "test-mixed",
                "version": "1.0.0",
                "dependencies": {
                    "react": "^18.2.0",
                    "lodash": "^4.17.21",
                    "@vercel/analytics": "^1.0.0"
                }
            }"#,
                expected_unused: vec!["lodash", "@vercel/analytics"],
                expected_used: vec!["react"],
            },
            TestCase {
                name: "all_unused",
                package_json_content: r#"{
                "name": "test-dry-run",
                "version": "1.0.0",
                "dependencies": {
                    "lodash": "^4.17.21",
                    "@vercel/analytics": "^1.0.0"
                }
            }"#,
                expected_unused: vec!["lodash", "@vercel/analytics"],
                expected_used: vec![],
            },
        ];

        for case in test_cases {
            let temp_dir = setup_temp_dir();
            let package_json_path = temp_dir.path().join("package.json");
            File::create(&package_json_path)
                .unwrap()
                .write_all(case.package_json_content.as_bytes())
                .unwrap();

            let project_root = env::var("CARGO_MANIFEST_DIR").unwrap();
            let test_fixtures_dir = PathBuf::from(project_root).join("test_fixtures");
            let unused_ts_src = test_fixtures_dir.join("unused.ts");
            let tsconfig_src = test_fixtures_dir.join("tsconfig.json");
            let unused_ts_dest = temp_dir.path().join("unused.ts");
            let tsconfig_dest = temp_dir.path().join("tsconfig.json");
            fs::copy(&unused_ts_src, &unused_ts_dest).unwrap();
            fs::copy(&tsconfig_src, &tsconfig_dest).unwrap();

            std::env::set_current_dir(&temp_dir).unwrap();
            let pb = ProgressBar::new(1);

            let package_json = read_package_json("package.json").unwrap();
            let dependencies: HashSet<String> = package_json
                .get("dependencies")
                .and_then(Value::as_object)
                .map_or_else(HashSet::new, |map| map.keys().cloned().collect());

            let (used_packages, explored_files, ignored_files) = scan_files(&dependencies, &pb);

            let required_deps = get_required_dependencies();
            let ignored_deps = read_cnpignore();
            let unused_dependencies: Vec<String> = dependencies
                .difference(&used_packages)
                .filter(|dep| !required_deps.contains(*dep) && !ignored_deps.contains(*dep))
                .cloned()
                .collect();

            let expected_used: HashSet<String> =
                case.expected_used.into_iter().map(String::from).collect();
            assert_eq!(
                used_packages, expected_used,
                "Test case {}: Expected used dependencies {:?}",
                case.name, expected_used
            );

            let mut expected_unused: Vec<String> =
                case.expected_unused.into_iter().map(String::from).collect();
            expected_unused.sort();
            let mut actual_unused = unused_dependencies.clone();
            actual_unused.sort();
            assert_eq!(
                actual_unused, expected_unused,
                "Test case {}: Expected unused dependencies {:?}",
                case.name, expected_unused
            );

            assert_eq!(
                explored_files,
                vec![unused_ts_dest.display().to_string()],
                "Test case {}: Should explore unused.ts",
                case.name
            );
            assert_eq!(
                ignored_files,
                Vec::<String>::new(),
                "Test case {}: No files should be ignored",
                case.name
            );

            handle_unused_dependencies(&unused_dependencies, true, false, false);

            let package_json_after = read_package_json("package.json").unwrap();
            let dependencies_after: HashSet<String> = package_json_after
                .get("dependencies")
                .and_then(Value::as_object)
                .map_or_else(HashSet::new, |map| map.keys().cloned().collect());
            assert_eq!(
                dependencies_after, dependencies,
                "Test case {}: Dry-run should not modify package.json",
                case.name
            );
        }
    }
}
