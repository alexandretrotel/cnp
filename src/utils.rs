use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

/// Creates a spinner-style progress bar with a custom message.
///
/// This function initializes a `ProgressBar` in spinner mode, displaying a green spinning animation
/// alongside the provided message. It is suitable for tasks with indeterminate duration.
///
/// # Arguments
///
/// * `message` - A string slice to display next to the spinner.
///
/// # Returns
///
/// Returns a configured `ProgressBar` instance in spinner mode with the specified message.
///
/// # Examples
///
/// ```
/// let spinner = create_spinner("Processing...");
/// // Displays a green spinner with "Processing..." until finished
/// spinner.finish_with_message("Done!");
/// ```
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb
}

/// Creates a bar-style progress bar with a custom message and length.
///
/// This function initializes a `ProgressBar` in bar mode, showing a cyan/blue progress bar, position,
/// total length, and the provided message. It is suitable for tasks with a known number of steps.
///
/// # Arguments
///
/// * `len` - The total number of steps for the progress bar (u64).
/// * `message` - A string slice to display next to the progress bar.
///
/// # Returns
///
/// Returns a configured `ProgressBar` instance in bar mode with the specified length and message.
///
/// # Examples
///
/// ```
/// let bar = create_bar(100, "Scanning files...");
/// // Displays a progress bar with "Scanning files..."
/// for _ in 0..100 {
///     bar.inc(1);
/// }
/// bar.finish_with_message("Scan complete!");
/// ```
pub fn create_bar(len: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb
}

/// Extracts the file name and extension from a given file path.
///
/// This function takes a file path as a string and returns an `Option` containing a tuple with the
/// file name and its extension. If the file name or extension cannot be determined, it returns `None`.
/// The file name is the last component of the path, and the extension is the part after the last dot.
/// If the path does not have a valid file name or extension, it returns `None`.
///
/// # Arguments
///
/// * `path` - A string slice representing the file path.
///
/// # Returns
///
/// Returns an `Option<(String, String)>` where the first element is the file name and the second
/// element is the file extension. If either cannot be determined, returns `None`.
///
/// # Examples
///
/// ```
/// let path = "/path/to/file.txt";
/// if let Some((name, ext)) = get_file_name_and_extension(path) {
///     println!("File name: {}, Extension: {}", name, ext);
/// } else {
///     println!("Could not extract file name or extension.");
/// }
/// ```
pub fn get_file_name_and_extension(path: &str) -> Option<(String, String)> {
    let path = Path::new(path);
    if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            return Some((file_name.to_string(), extension.to_string()));
        }
    }
    None
}
