use std::error::Error;
use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

extern "system" {
    fn SystemParametersInfoW(uiAction: u32, uiParam: u32, pvParam: *const u16, fWinIni: u32)
        -> i32;
}

const SPI_SETDESKWALLPAPER: u32 = 0x0014;
const SPIF_UPDATEINIFILE: u32 = 0x01;
const SPIF_SENDCHANGE: u32 = 0x02;

/// Set the desktop background on Windows.
///
/// # Errors
/// Fails if the registry key cannot be set or if the system parameters cannot be updated.
pub(crate) fn set_background(image_path: &Path) -> Result<(), Box<dyn Error>> {
    let image_path_wide: Vec<u16> = OsStr::new(image_path)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect();

    let result = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            image_path_wide.as_ptr() as *mut _,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    };

    if result == 0 {
        return Err(format!(
            "Could not set desktop wallpaper: {}",
            io::Error::last_os_error()
        )
        .into());
    }

    Ok(())
}
