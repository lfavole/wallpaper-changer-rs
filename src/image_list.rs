use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use log::debug;
use log::info;
use rand::seq::IteratorRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::image_structs::is_image;
use crate::image_structs::Image;
use crate::image_structs::LocalImage;
use crate::image_structs::OnlineImage;
use crate::paths::Paths;
use super::Config;
use super::NoImagesError;


// Imports are OK here
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default)]
/// Data for the online images stored on disk.
pub(crate) struct ImageData {
    pub(crate) urls: Vec<OnlineImage>,
    pub(crate) current_index: usize,
    pub(crate) needs_downloading: bool,
}

impl ImageData {
    /// Loads the image data from its file.
    ///
    /// # Errors
    /// Fails if the image data directory can't be determined
    /// or if the file is malformed.
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        let data_path = Paths::image_data_path();
        debug!("Loading image data from {:?}", data_path);

        let ret = if data_path.exists() {
            let image_data = serde_json::from_reader(fs::File::open(data_path)?)?;
            debug!("Image data loaded");
            Ok(image_data)
        } else {
            debug!("Image data file not found, using default values");
            Ok(Self::default())
        };
        if let Ok(ref data) = ret {
            info!(
                "Loaded {} images from the cache, current index is {}",
                data.urls.len(),
                data.current_index
            );
        }
        ret
    }

    /// Saves the image data to its file.
    ///
    /// # Errors
    /// Fails if the file can't be written to.
    pub(crate) fn store(&self) -> Result<(), Box<dyn Error>> {
        debug!("Storing image data to {:?}", Paths::image_data_path());
        Ok(serde_json::to_writer(
            fs::File::create(Paths::image_data_path())?,
            self,
        )?)
    }

    /// Deletes all the images in this [`ImageData`].
    ///
    /// # Errors
    /// Fails if an image or the data file can't be deleted.
    pub(crate) fn clear(&mut self) -> Result<(), Box<dyn Error>> {
        for image in &self.urls {
            let path = image.get_path();
            if path.exists() {
                debug!("Removing image {:?}", path);
                fs::remove_file(path)?;
            } else {
                debug!("Image {:?} not found", path);
            }
        }
        // Remove the file
        let data_path = Paths::image_data_path();
        if data_path.exists() {
            debug!("Removing image data file {:?}", data_path);
            fs::remove_file(data_path)?;
        } else {
            debug!("Image data file {:?} not found", data_path);
        }
        Ok(())
    }

    /// Downloads all the images in this [`ImageData`].
    ///
    /// # Errors
    /// Fails if an image can't be downloaded.
    pub(crate) fn download_all_images(&self) -> Result<(), Box<dyn Error>> {
        for image in &self.urls {
            image.download()?;
        }
        Ok(())
    }

    /// Deletes all the old online images and background images.
    ///
    /// # Errors
    /// Fails if an image can't be deleted.
    pub(crate) fn delete_old_images(
        &self,
        current_background: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let image_paths = self
            .urls
            .iter()
            .map(super::image_structs::Image::get_path)
            .collect::<Vec<_>>();
        debug!("Found {} images to keep", image_paths.len());
        let mut removed_images: usize = 0;
        for entry in fs::read_dir(Paths::downloaded_pictures_dir())? {
            let path = entry?.path();
            if path.is_file() && image_paths.iter().all(|image_path| path != *image_path) {
                debug!("Removing old image {:?}", path);
                fs::remove_file(path)?;
                removed_images += 1;
            } else {
                debug!("Keeping image {:?}", path);
            }
        }
        for entry in fs::read_dir(Paths::temp_dir())? {
            let path = entry?.path();
            if path.is_file() && path != current_background {
                debug!("Removing old background image {:?}", path);
                fs::remove_file(path)?;
                removed_images += 1;
            }
        }
        info!("Removed {} old images", removed_images);
        Ok(())
    }
}

/// Downloads pictures from Unsplash.
///
/// # Errors
/// Fails if the Unsplash API endpoint can't be contacted or if its response can't be decoded.
#[expect(clippy::missing_panics_doc)]
pub(crate) fn download_pictures(config: &Config) -> Result<Vec<OnlineImage>, Box<dyn Error>> {
    #[expect(clippy::unwrap_used)]
    let mut url = url::Url::parse(if config.api_key.is_empty() {
        debug!("No API key found, using the lfnewtab API");
        "https://lfnewtab.vercel.app/unsplash/"
    } else {
        debug!("Using the Unsplash API");
        "https://api.unsplash.com/"
    })
    .unwrap();

    let search_term = config
        .search_terms
        .split(',')
        .choose(&mut rand::rng())
        .unwrap_or_default();

    if search_term.is_empty() || search_term == "random" {
        debug!("Search term is {:?}, getting random images", search_term);
        url.set_path(&(url.path().to_string() + "photos/random"));
        url.query_pairs_mut()
            .append_pair("count", config.images_per_download.to_string().as_str());
    } else {
        debug!("Searching for random images with the term: {search_term:?}");
        url.set_path(&(url.path().to_string() + "photos/random"));
        url.query_pairs_mut().append_pair("query", search_term);
        url.query_pairs_mut()
            .append_pair("count", config.images_per_download.to_string().as_str());
    }

    if !config.api_key.is_empty() {
        url.query_pairs_mut()
            .append_pair("client_id", &config.api_key);
    }

    let response = ureq::get(url.as_str()).call()?;
    let response: Value = serde_json::from_reader(response.into_body().as_reader())?;

    let image_urls = if response.is_array() {
        response.as_array()
    } else {
        response["results"].as_array()
    }
    .ok_or("Error parsing response")?
    .iter()
    .map(OnlineImage::from)
    .collect::<Vec<_>>();
    debug!("Downloaded {} images", image_urls.len());

    Ok(image_urls)
}

/// Selects a random image, downloads it and returns it.
///
/// # Errors
/// Fails if the local or web images can't be obtained or downloaded.
pub(crate) fn select_random_image(
    config: &Config,
    image_data: &mut ImageData,
) -> Result<Box<dyn Image>, Box<dyn Error>> {
    let mut rng = rand::rng();

    // Randomly decide between a local or online image
    let use_local_image = rng.random::<bool>();

    if use_local_image {
        if let Ok(ret) = LocalImage::get(config, image_data) {
            return Ok(ret);
        }
    }

    if !use_local_image {
        if let Ok(ret) = OnlineImage::get(config, image_data) {
            return Ok(ret);
        }
    }

    // Check if there are no local images and no online images
    Err(Box::new(NoImagesError))
}

/// Returns all the images in a directory and in its subdirectories, without using a cache.
///
/// # Errors
/// Fails if a directory can't be read.
pub(crate) fn get_images_no_cache(pictures_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut images = Vec::new();
    for entry in fs::read_dir(pictures_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let mut other_images = get_images_no_cache(&path)?;
            images.append(&mut other_images);
        } else if path.is_file() && is_image(&path) {
            images.push(path);
        }
    }
    Ok(images)
}

/// Returns all the images in a directory and in its subdirectories.
///
/// # Errors
/// Fails if the cache directory can't be found or created or if a directory can't be read.
pub(crate) fn get_images(pictures_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let cache_path = Paths::get_path_cache_file_path(pictures_dir);
    // if the change time of the folder is newer than the cache file, regenerate the cache
    // otherwise, read the cache file and return the paths
    if let Ok(metadata) = fs::metadata(pictures_dir) {
        if let Ok(cache_metadata) = fs::metadata(&cache_path) {
            if metadata.modified()? <= cache_metadata.modified()? {
                let cache_file = fs::File::open(&cache_path)?;
                let paths: Vec<String> = serde_json::from_reader(cache_file)?;
                let images = paths
                    .iter()
                    .map(|path| pictures_dir.join(path))
                    .collect::<Vec<_>>();
                return Ok(images);
            }
        }
    }

    let images = get_images_no_cache(pictures_dir)?;

    // Write the paths to the cache file, but only the part after the pictures_dir
    let cache_file = fs::File::create(&cache_path)?;
    let paths = images
        .iter()
        .map(|path| path.strip_prefix(pictures_dir).unwrap().to_string_lossy())
        .collect::<Vec<_>>();
    serde_json::to_writer(cache_file, &paths)?;

    Ok(images)
}
