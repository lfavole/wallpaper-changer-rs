use log::{debug, info};
use std::error::Error;
use std::path::Path;
use std::process::Command;

extern "C" {
    fn getuid() -> u32;
}

/// Set the desktop background on Linux.
///
/// # Errors
/// Fails if the call to `gsettings` fails.
pub(crate) fn set_background(image_path: &Path) -> Result<(), Box<dyn Error>> {
    info!("Setting background...");
    let uid = unsafe { getuid() };
    debug!("uid is {}", uid);
    Command::new("gsettings")
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path=/run/user/{uid}/bus"),
        )
        .args([
            "set",
            "org.cinnamon.desktop.background",
            "picture-uri",
            &format!("file://{}", image_path.to_string_lossy()),
        ])
        .output()
        .map_err(|err| format!("Could not set background using gsettings: {err}"))?;

    Ok(())
}
