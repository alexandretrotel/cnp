use std::path::Path;

pub const PACKAGE_JSON_PATH: &str = "package.json";
pub const EXTENSIONS: [&str; 5] = ["js", "ts", "jsx", "tsx", "mdx"];
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

pub fn is_typescript_project() -> bool {
    Path::new("tsconfig.json").exists()
}
