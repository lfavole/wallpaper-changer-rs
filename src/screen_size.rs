//! Utility functions to get the screen size.
use screen_size::get_primary_screen_size;
use std::sync::OnceLock;

/// Returns the screen size.
///
/// The value is cached across multiple runs.
pub(crate) fn get_screen_size() -> &'static (u32, u32) {
    static SCREEN_SIZE: OnceLock<(u32, u32)> = OnceLock::new();
    SCREEN_SIZE.get_or_init(|| {
        let tmp = get_primary_screen_size().unwrap_or((1920, 1080));
        #[expect(clippy::cast_possible_truncation)]
        (tmp.0 as u32, tmp.1 as u32)
    })
}
