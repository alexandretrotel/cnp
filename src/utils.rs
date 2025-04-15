use indicatif::{ProgressBar, ProgressStyle};

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
