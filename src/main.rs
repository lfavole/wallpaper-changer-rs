//! A program that automatically changes the wallpaper,
//! choosing a local or online image.
use add_scheduled_task::{register_task, unregister_task};
use compile_dotenv::compile_env;
use config::Config;
use ftail::channels::console::ConsoleLogger;
use ftail::channels::daily_file::DailyFileLogger;
use image::imageops::FilterType;
use log::info;
use log::{debug, error, LevelFilter};
use paths::Paths;
use screen_size::get_screen_size;
use sentry_log::LogFilter;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;

#[derive(Debug)]
/// An error that is raised when no images are available.
struct NoImagesError;

impl fmt::Display for NoImagesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No images available locally or online")
    }
}

impl Error for NoImagesError {}

/// The real entry point for the program.
fn main() {
    match real_main() {
        Ok(()) => {}
        Err(err) => error!("Error: {err}"),
    }
}

/// Changes the wallpaper or registers itself as a scheduled task if the "register" argument is provided.
///
/// # Errors
/// The program can fail for a number of reasons.
fn real_main() -> Result<(), Box<dyn Error>> {
    log_panics::init();

    // Initialize the logger
    let logger1 = ConsoleLogger::new(ftail::Config {
        level_filter: LevelFilter::Info,
        ..Default::default()
    });

    let logger2 = DailyFileLogger::new(
        &Paths::logs_dir().to_string_lossy(),
        ftail::Config {
            level_filter: LevelFilter::Debug,
            retention_days: Some(7),
            ..Default::default()
        },
    )?;

    let logger3 = sentry_log::SentryLogger::new().filter(|md| match md.level() {
        log::Level::Error => LogFilter::Exception,
        _ => LogFilter::Breadcrumb,
    });

    log::set_boxed_logger(Box::new(multi_log::MultiLogger::new(vec![
        Box::new(logger1),
        Box::new(logger2),
        Box::new(logger3),
    ])))?;

    log::set_max_level(LevelFilter::Trace);

    let dsn = compile_env!("SENTRY_DSN");
    let _guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 0.1,
            ..Default::default()
        },
    ));

    // if the first argument is register, register a scheduled task
    if env::args().nth(1).is_some_and(|arg| arg == "register") {
        debug!("Found register argument, registering scheduled task");
        return register_task(&env::current_exe()?);
    }

    // if the first argument is unregister, unregister a scheduled task
    if env::args().nth(1).is_some_and(|arg| arg == "unregister") {
        debug!("Found unregister argument, unregistering scheduled task");
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
        debug!(
            "Environment variable DBUS_SESSION_BUS_ADDRESS is set to {:?}",
            env::var("DBUS_SESSION_BUS_ADDRESS")
        );
    }

    // Load configuration
    let config = Config::load()?;

    // Load image data
    let mut image_data = image_list::ImageData::load()?;

    // Select a random image (local or online)
    let image = image_list::select_random_image(&config, &mut image_data)?;

    // Load the image
    let img = image::open(image.get_path())?;

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
    let output_path = Paths::temp_dir().join(format!(
        "background_{}.png",
        chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
    ));
    // Create the parent directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    info!("Saving image in {output_path:?}...");
    background.save(&output_path)?;

    // Set the image as the background
    debug!("Setting background");
    set_background::set_background(&output_path)?;

    // Find old background images and delete them
    image_data.delete_old_images(&output_path)?;

    // Download all the other images
    debug!("Downloading all other images");
    image_data.download_all_images()?;

    Ok(())
}

mod add_scheduled_task;
mod config;
mod date_format;
mod image_list;
mod image_structs;
mod images;
mod paths;
mod screen_size;
mod set_background;
