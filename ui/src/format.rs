//! Formatting utilities.

/// Formats a size in bytes to be human-readable.
pub fn format_size(size: u64) -> String {
    if size < (1 << 10) {
        format!("{size} bytes")
    } else if size < (1 << 20) {
        format!("{} KB", size >> 10)
    } else if size < (1 << 30) {
        format!("{} MB", size >> 20)
    } else {
        format!("{} GB", size >> 30)
    }
}
