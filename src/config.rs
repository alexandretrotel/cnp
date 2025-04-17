use std::path::Path;

pub const PACKAGE_JSON_PATH: &str = "package.json";
pub const EXTENSIONS: [&str; 7] = ["js", "ts", "jsx", "tsx", "mdx", "cjs", "mjs"];
pub const IGNORE_FOLDERS: [&str; 10] = [
    "node_modules",
    "dist",
    "build",
    "public",
    ".next",
    ".git",
    "coverage",
    "cypress",
    "test",
    "output",
];
pub const TYPESCRIPT_EXTENSIONS: [&str; 4] = ["ts", "tsx", "d.ts", "cts"];

/// Checks if the current directory is a TypeScript project by looking for a `tsconfig.json` file.
///
/// # Arguments
/// * `path` - A string slice representing the path to the directory to check.
///
/// # Returns
///
/// Returns `true` if a `tsconfig.json` file exists in the current directory, indicating a TypeScript project.
/// Returns `false` otherwise.
///
/// # Examples
///
/// ```
/// if is_typescript_project() {
///     println!("This is a TypeScript project!");
/// } else {
///     println!("This is not a TypeScript project.");
/// }
/// ```
pub fn is_typescript_project(path: &str) -> bool {
    Path::new(&path).join("tsconfig.json").exists()
}
