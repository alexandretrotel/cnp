#[cfg(test)]
mod tests {
    use crate::file_scanner::scan_files;
    use indicatif::ProgressBar;
    use std::collections::HashSet;
    use std::env;
    use std::fs::{self};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_temp_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        temp_dir
    }

    #[test]
    fn test_scan_files() {
        let temp_dir = setup_temp_dir();
        let project_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_fixtures_dir = PathBuf::from(project_root).join("test_fixtures");
        let unused_ts_src = test_fixtures_dir.join("unused.ts");
        let tsconfig_src = test_fixtures_dir.join("tsconfig.json");
        let unused_ts_dest = temp_dir.path().join("unused.ts");
        let tsconfig_dest = temp_dir.path().join("tsconfig.json");

        fs::copy(&unused_ts_src, &unused_ts_dest).unwrap();
        fs::copy(&tsconfig_src, &tsconfig_dest).unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();
        let dependencies: HashSet<String> = ["react", "lodash", "@vercel/analytics"]
            .into_iter()
            .map(String::from)
            .collect();
        let pb = ProgressBar::new(1);

        let (used_packages, explored_files, ignored_files) = scan_files(&dependencies, &pb);

        let expected_used: HashSet<String> = ["react"].into_iter().map(String::from).collect();
        assert_eq!(
            used_packages, expected_used,
            "Should only detect react as used"
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
}
