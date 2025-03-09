//! Utility functions to register the wallpaper changer as a scheduled task on Windows.
use std::error::Error;
use std::io;
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
/// Registers the given `script_path` as a scheduled task on Windows.
///
/// # Errors
/// Fails if `schtasks` can't be called.
pub(crate) fn register_task(script_path: &Path) -> Result<(), Box<dyn Error>> {
    let task_name = "wallpaper-changer-rs";

    // Check if the task is already registered
    let status = Command::new("schtasks")
        .args(&["/Query", "/TN", task_name])
        .status()?;

    if status.success() {
        println!("Task '{task_name}' is already registered.");
        return Ok(());
    }

    // Create a task in Task Scheduler to run every 5 minutes
    let output = Command::new("schtasks")
        .args(&[
            "/Create",
            "/SC",
            "MINUTE",
            "/MO",
            "5",
            "/TN",
            task_name,
            "/TR",
            &script_path.to_string_lossy(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to create task: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        )));
    }

    println!("Task '{task_name}' created successfully.");

    Ok(())
}

#[cfg(target_os = "windows")]
/// Unregisters the given `script_path` as a scheduled task on Windows.
///
/// # Errors
/// Fails if `schtasks` can't be called.
pub(crate) fn unregister_task(script_path: &Path) -> Result<(), Box<dyn Error>> {
    let task_name = "wallpaper-changer-rs";

    // Check if the task is already registered
    let status = Command::new("schtasks")
        .args(&["/Query", "/TN", task_name])
        .status()?;

    if !status.success() {
        println!("Task '{task_name}' is not registered.");
        return Ok(());
    }

    // Delete the task from Task Scheduler
    let output = Command::new("schtasks")
        .args(&["/Delete", "/TN", task_name, "/F"])
        .output()?;

    if !output.status.success() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to delete task: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        )));
    }

    println!("Task '{task_name}' deleted successfully.");

    Ok(())
}
