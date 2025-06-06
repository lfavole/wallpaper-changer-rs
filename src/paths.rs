//! Utility functions to get files and folders accessed by the program.
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// A macro to create a function that returns a file path and creates its parent directory if it doesn't exist.
macro_rules! file {
    ($name:ident, $path:expr) => {
        pub(crate) fn $name() -> &'static Path {
            static $name: OnceLock<&'static Path> = OnceLock::new();
            $name.get_or_init(|| {
                let ret = Self::base_dir().join($path);
                Self::create_file_parent_if_needed(&ret).expect(concat!(
                    "Could not create the parent directory for ",
                    stringify!($name)
                ));
                Box::leak(ret.into_boxed_path())
            })
        }
    };
}

/// A macro to define a function that returns a directory path and creates it if it doesn't exist.
macro_rules! dir {
    ($name:ident, $path:expr) => {
        pub(crate) fn $name() -> &'static Path {
            static $name: OnceLock<&'static Path> = OnceLock::new();
            $name.get_or_init(|| {
                let ret = Self::base_dir().join($path);
                Self::create_dir_if_needed(&ret).expect(concat!(
                    "Could not create the directory for ",
                    stringify!($name)
                ));
                Box::leak(ret.into_boxed_path())
            })
        }
    };
}

pub(crate) struct Paths;

#[expect(non_upper_case_globals)]
impl Paths {
    /// Returns the local data directory.
    ///
    /// The value is cached across multiple runs.
    pub(crate) fn base_dir() -> &'static Path {
        static BASE_DIR: OnceLock<&'static Path> = OnceLock::new();
        BASE_DIR.get_or_init(|| {
            Box::leak(
                dirs::data_local_dir()
                    .expect("Could not find the local data directory")
                    .join("wallpaper-changer-rs")
                    .into_boxed_path(),
            )
        })
    }

    /// Create a directory if it doesn't exist. Returns the directory path.
    ///
    /// # Errors
    /// Fails if the directory can't be created.
    fn create_dir_if_needed(dir: &Path) -> Result<(), Box<dyn Error>> {
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    /// Create the parent directory of a file if it doesn't exist.
    ///
    /// # Errors
    /// Fails if the parent directory can't be created.
    fn create_file_parent_if_needed(file: &Path) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = file.parent() {
            Self::create_dir_if_needed(parent)?;
        }
        Ok(())
    }

    dir!(logs_dir, "logs");
    dir!(downloaded_pictures_dir, "pictures");
    dir!(path_cache_dir, "path_cache");
    dir!(temp_dir, "tmp");

    file!(config_file, "config.toml");
    file!(image_data_path, "image_data.json");
    file!(crontab_temp_file, "tmp/crontab");

    /// Returns the path where the pictures list for the given directory is stored.
    pub(crate) fn get_path_cache_file_path(name: &Path) -> PathBuf {
        Self::path_cache_dir()
            .join(name.to_string_lossy().replace(['\\', '/'], "_"))
            .clone()
    }
}
