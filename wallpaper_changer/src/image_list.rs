use crate::get_screen_size;
use crate::image_structs::Image;
use crate::image_structs::LocalImage;
use crate::image_structs::OnlineImage;

use super::Config;
use super::NoImagesError;
use image::DynamicImage;
use image::GenericImageView;
use image::ImageDecoder;
use image::ImageReader;
use rand::seq::IteratorRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::io::copy;
use std::path::Path;
use std::path::PathBuf;
use url::Url;

#[derive(Deserialize, Serialize)]
/// Data for the online images stored on disk.
pub(crate) struct ImageData {
    urls: Vec<OnlineImage>,
    current_index: usize,
}

impl ImageData {
    /// Loads the image data from its file.
    ///
    /// # Errors
    /// Fails if the image data directory can't be determined
    /// or if the file is malformed.
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        let data_path = dirs::data_local_dir()
            .ok_or("Could not find the local data directory")?
            .join("wallpaper-changer-rs/image_data.json");

        let ret = if data_path.exists() {
            let image_data = serde_json::from_reader(fs::File::open(data_path)?)?;
            Ok(image_data)
        } else {
            Ok(Self {
                urls: Vec::new(),
                current_index: 0,
            })
        };
        if let Ok(ref data) = ret {
            println!(
                "Loaded {} images from the cache, current index is {}",
                data.urls.len(),
                data.current_index
            );
        }
        ret
    }

    /// Returns the path to the image data file.
    ///
    /// # Errors
    /// Fails if the image data directory can't be determined.
    pub(crate) fn get_data_path() -> Result<PathBuf, Box<dyn Error>> {
        Ok(dirs::data_local_dir()
            .ok_or("Could not find the local data directory")?
            .join("wallpaper-changer-rs/image_data.json"))
    }

    /// Saves the image data to its file.
    ///
    /// # Errors
    /// Fails if the image data directory can't be determined, created
    /// or if the file can't be written to.
    pub(crate) fn store(&self) -> Result<(), Box<dyn Error>> {
        let data_path = Self::get_data_path()?;

        // Create the parent directory if needed
        if let Some(parent) = data_path.parent() {
            fs::create_dir_all(parent)?;
        }
        serde_json::to_writer(fs::File::create(&data_path)?, self)?;

        Ok(())
    }

    /// Deletes all the images in this [`ImageData`].
    ///
    /// # Errors
    /// Fails if an image or the data file can't be deleted.
    pub(crate) fn clear(&mut self) -> Result<(), Box<dyn Error>> {
        for image in &self.urls {
            if let Ok(path) = image.get_path() {
                if path.exists() {
                    fs::remove_file(path)?;
                }
            }
        }
        // Remove the file
        let data_path = Self::get_data_path()?;
        if data_path.exists() {
            fs::remove_file(data_path)?;
        }
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
        "https://lfnewtab.vercel.app/unsplash/"
    } else {
        "https://api.unsplash.com/"
    })
    .unwrap();

    let search_term = config
        .search_terms
        .split(',')
        .choose(&mut rand::rng())
        .unwrap_or_default();
    if search_term.is_empty() || search_term == "random" {
        url.set_path(&(url.path().to_string() + "photos/random"));
        url.query_pairs_mut().append_pair("count", "10");
    } else {
        url.set_path(&(url.path().to_string() + "search/photos"));
        url.query_pairs_mut().append_pair("query", search_term);
        url.query_pairs_mut().append_pair("per_page", "10");
    }

    if !config.api_key.is_empty() {
        url.query_pairs_mut()
            .append_pair("client_id", &config.api_key);
    }

    let response = ureq::get(url.as_str()).call()?;
    let response: Value = serde_json::from_reader(response.into_body().as_reader())?;

    let mut image_urls = Vec::new();
    let images = if response.is_array() {
        response.as_array()
    } else {
        response["results"].as_array()
    }.ok_or("Error parsing response")?;
    for image in images {
        image_urls.push(OnlineImage::from(image));
    }

    Ok(image_urls)
}

/// Download an [`OnlineImage`] to its destination file.
///
/// # Errors
/// Fails if the URL can't be edited or if the destination file can't be written to.
pub(crate) fn download_image(image: &OnlineImage) -> Result<(), Box<dyn Error>> {
    let mut image_url = Url::parse(&image.url)?;
    // Keep only the ixid parameter
    let ixid = image_url
        .query_pairs()
        .find(|(key, _)| key == "ixid")
        .map(|(_, value)| value.to_string());
    image_url.query_pairs_mut().clear();
    if let Some(value) = ixid {
        image_url.query_pairs_mut().append_pair("ixid", &value);
    }
    let screen_dimensions = get_screen_size();
    image_url
        .query_pairs_mut()
        .append_pair("fm", "jpg")
        .append_pair("q", "85")
        .append_pair("w", &screen_dimensions.0.to_string())
        .append_pair("h", &screen_dimensions.1.to_string())
        .append_pair("fit", "crop")
        .append_pair("crop", "faces,edges");
    let image_response = ureq::get(image_url.to_string()).call()?;
    // Create the parent directory if needed
    if let Some(parent) = image.get_path()?.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut image_file = fs::File::create(image.get_path()?)?;
    copy(&mut image_response.into_body().as_reader(), &mut image_file)?;

    Ok(())
}

/// Selects a random image, downloads it and returns it.
///
/// # Errors
/// Fails if the local or web images can't be obtained or downloaded.
#[expect(clippy::missing_panics_doc)]
pub(crate) fn select_random_image(
    config: &Config,
    image_data: &mut ImageData,
    pictures_dir: &Path,
) -> Result<Box<dyn Image>, Box<dyn Error>> {
    let mut rng = rand::rng();

    // Randomly decide between a local or online image
    let use_local_image = rng.random::<bool>();

    if use_local_image {
        println!("Getting local images...");
        let local_images = get_images(pictures_dir)?;

        if !local_images.is_empty() {
            for _ in 0..10000 {
                // Select a random local image
                #[expect(clippy::unwrap_used)]
                let image_path = local_images.iter().choose(&mut rng).unwrap().clone();
                if is_too_vertical(&image_path) {
                    println!("Skipping {image_path:?} because it's too vertical");
                    continue;
                }
                println!("Selecting {image_path:?}");
                return Ok(Box::new(LocalImage::from(image_path)));
            }
        }
    }

    if !use_local_image {
        // Check if we need to download new images
        if image_data.current_index >= image_data.urls.len() {
            println!("Downloading pictures from Unsplash...");
            // Download random pictures from Unsplash
            match download_pictures(config) {
                Ok(image_urls) => {
                    // Clear the old images
                    image_data.clear()?;
                    // Store new images and reset current index
                    *image_data = ImageData {
                        urls: image_urls,
                        current_index: 0,
                    };
                    println!("{} pictures downloaded", image_data.urls.len());
                    image_data.store()?;
                }
                Err(err) => println!("Error: {err}"),
            }
        }

        if image_data.current_index < image_data.urls.len() {
            // Use the current online image
            let current_image = image_data.urls[image_data.current_index].clone();

            if !current_image.get_path()?.exists() {
                download_image(&current_image)?;
            }

            // Increment the current index and store it
            image_data.current_index += 1;
            image_data.store()?;

            return Ok(Box::new(current_image));
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
    let cache_path = dirs::data_local_dir()
        .ok_or("Could not find the local data directory")?
        .join(format!("wallpaper-changer-rs/path_cache/{}", pictures_dir.to_string_lossy().replace(['\\', '/'], "_")));
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
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

pub(crate) fn is_image(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("png" | "jpg" | "jpeg" | "bmp" | "gif")
    )
}

/// Opens an image file and rotates it according to its EXIF metadata.
///
/// # Errors
/// Fails if the image can't be opened or if its orientation can't be determined.
pub(crate) fn open_image(path: &Path) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // Rotate the image according to its EXIF metadata
    let mut decoder = ImageReader::open(path)?.into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    Ok(image)
}

/// Returns `true` if the image is too vertical for the current screen size.
///
/// If the image size can't be determined, it returns `false`.
pub(crate) fn is_too_vertical(path: &Path) -> bool {
    #[expect(clippy::cast_precision_loss)]
    if let Ok(img) = open_image(path) {
        let dimensions = img.dimensions();
        let screen_size = get_screen_size();

        (dimensions.1 as f32 / dimensions.0 as f32) / (screen_size.1 as f32 / screen_size.0 as f32)
            > 1.5
    } else {
        false
    }
}
