//! Utility functions to register the wallpaper changer as a scheduled task on Linux.
use log::info;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::Paths;

/// Registers the given `script_path` as a scheduled task on Linux.
///
/// # Errors
/// Fails if the crontab file can't be accessed or edited.
pub(crate) fn register_task(script_path: &Path) -> Result<(), Box<dyn Error>> {
    // Get the current user's crontab

    use log::info;
    let cron_result = Command::new("crontab").arg("-l").output()?;
    let mut cron_content: String = if cron_result.status.success() {
        String::from_utf8_lossy(&cron_result.stdout).to_string()
    } else {
        String::new()
    };

    // Ensure the script is not already registered
    if cron_content.contains(&*script_path.to_string_lossy()) {
        info!("The script is already registered as a cron job.");
        return Ok(());
    }

    // Register the script to run every 5 minutes
    cron_content.push_str(&format!("*/5 * * * * {}\n", script_path.to_string_lossy()));

    // Create a temporary file
    let cron_file = Paths::crontab_temp_file();
    if let Some(parent) = cron_file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&cron_file, cron_content)?;
    Command::new("crontab").arg(cron_file).output()?;

    fs::remove_file(cron_file)?;

    info!("Script added to crontab");

    Ok(())
}

/// Unregisters the given `script_path` as a scheduled task on Linux.
///
/// # Errors
/// Fails if the crontab file can't be accessed or edited.
pub(crate) fn unregister_task(script_path: &Path) -> Result<(), Box<dyn Error>> {
    // Get the current user's crontab
    let cron_result = Command::new("crontab").arg("-l").output()?;
    let mut cron_content: String = if cron_result.status.success() {
        String::from_utf8_lossy(&cron_result.stdout).to_string()
    } else {
        String::new()
    };

    // Ensure the script is registered
    if !cron_content.contains(&*script_path.to_string_lossy()) {
        info!("The script is not registered as a cron job.");
        return Ok(());
    }

    // Remove the script from the crontab
    cron_content = cron_content
        .lines()
        .filter(|line| !line.contains(&*script_path.to_string_lossy()))
        .collect::<Vec<&str>>()
        .join("\n");

    // Create a temporary file
    let cron_file = Paths::crontab_temp_file();
    if let Some(parent) = cron_file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&cron_file, cron_content)?;
    Command::new("crontab").arg(cron_file).output()?;

    fs::remove_file(cron_file)?;

    info!("Script added to crontab");

    Ok(())
}
