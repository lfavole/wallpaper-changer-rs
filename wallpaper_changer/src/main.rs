//! A program that automatically changes the wallpaper,
//! choosing a local or online image.
use config::Config;
use ftail::channels::console::ConsoleLogger;
use ftail::channels::daily_file::DailyFileLogger;
use image::imageops::FilterType;
use log::{debug, LevelFilter};
use macros::compile_env;
use rand::distr::Alphanumeric;
use rand::Rng;
use add_scheduled_task::{register_task, unregister_task};
use screen_size::get_screen_size;
use sentry_log::LogFilter;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::{self, create_dir_all};
use std::path::Path;

#[derive(Debug)]
/// An error that is raised when no images are available.
struct NoImagesError;

impl fmt::Display for NoImagesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No images available locally or online")
    }
}

impl Error for NoImagesError {}

/// Changes the wallpaper or registers itself as a scheduled task if the "register" argument is provided.
///
/// # Errors
/// The program can fail for a number of reasons.
fn main() -> Result<(), Box<dyn Error>> {
    let dsn = compile_env!("SENTRY_DSN");
    let _guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 0.1,
            ..Default::default()
        },
    ));

    // Initialize the logger
    let logger1 = ConsoleLogger::new(ftail::Config {
        level_filter: LevelFilter::Info,
        ..Default::default()
    });

    let logger_dir = dirs::data_local_dir()
        .ok_or("Could not find the local data directory")?
        .join("wallpaper-changer-rs/logs");
    if !logger_dir.exists() {
        create_dir_all(&logger_dir)?;
    }
    let logger2 = DailyFileLogger::new(&logger_dir.to_string_lossy(), ftail::Config {
        level_filter: LevelFilter::Debug,
        retention_days: Some(7),
        ..Default::default()
    })?;

    let logger3 = sentry_log::SentryLogger::new()
        .filter(|md| match md.level() {
            log::Level::Error => LogFilter::Exception,
            _ => LogFilter::Breadcrumb,
        });

    log::set_boxed_logger(
        Box::new(multi_log::MultiLogger::new(vec![
            Box::new(logger1),
            Box::new(logger2),
            Box::new(logger3),
        ]))
    )?;

    log::set_max_level(LevelFilter::Trace);

    // if the first argument is register, register a scheduled task
    if env::args().nth(1).is_some_and(|arg| arg == "register") {
        return register_task(&env::current_exe()?);
    }

    // if the first argument is unregister, unregister a scheduled task
    if env::args().nth(1).is_some_and(|arg| arg == "unregister") {
        return unregister_task(&env::current_exe()?);
    }

    // on Linux
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn getuid() -> u32;
        }
        let uid = unsafe { getuid() };
        debug!("uid is {}", uid);
        unsafe {
            env::set_var(
                "DBUS_SESSION_BUS_ADDRESS",
                format!("unix:path=/run/user/{uid}/bus"),
            );
        }
    }

    // Load configuration
    let config = Config::load()?;

    // Load image data
    let mut image_data = image_list::ImageData::load()?;

    // Get the path to the Pictures directory
    let pictures_dir = Path::new(&config.pictures_folder);

    // Select a random image (local or online)
    let image = image_list::select_random_image(&config, &mut image_data, pictures_dir)?;

    // Load the image
    let img = image::open(&image.get_path()?)?;

    // Resize the background to fill the screen size
    let screen_size = get_screen_size();
    let mut background = img.resize_to_fill(screen_size.0, screen_size.1, FilterType::Lanczos3);

    // Write the filename and date on the image
    images::write_text_on_image(
        &mut background,
        &image.get_description(),
        config.font_size,
        &config.label_position,
    )?;

    // Save the modified image
    let output_path = dirs::cache_dir()
        .ok_or("Could not find the cache directory")?
        .join(format!(
            "wallpaper-changer-rs/background_{}.png",
            // https://stackoverflow.com/a/54277357
            rand::rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect::<String>(),
        ));
    // Create the parent directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
        // Find old background images and delete them
        for entry in fs::read_dir(parent)? {
            let path = entry?.path();
            if path.is_file()
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("background_"))
            {
                fs::remove_file(path)?;
            }
        }
    }
    println!("Saving image in {output_path:?}...");
    background.save(&output_path)?;

    // Set the image as the background
    set_background::set_background(&output_path)?;

    Ok(())
}

mod add_scheduled_task;
mod config;
mod date_format;
mod image_list;
mod image_structs;
mod images;
mod screen_size;
mod set_background;
