use chrono::Local;
use chrono::{DateTime, Utc};
use log::debug;
use log::error;
use log::info;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;
use std::error::Error;
use std::fs::{File, metadata};
use std::io::copy;
use std::path::Path;
use std::path::PathBuf;

use crate::date_format::format_date_in_french;
use crate::get_screen_size;
use crate::image_list::download_pictures;
use crate::image_list::get_images;
use crate::image_list::is_too_vertical;
use crate::image_list::ImageData;
use crate::paths::Paths;
use crate::Config;
use crate::NoImagesError;

/// An image that has a path and a description.
pub(crate) trait Image {
    /// Returns a random image.
    ///
    /// # Errors
    /// It depends on the implementation but it fails if no image can be found.
    fn get(config: &Config, image_data: &mut ImageData) -> Result<Box<Self>, Box<dyn Error>> where Self: Sized;
    /// Returns the path of the image.
    fn get_path(&self) -> PathBuf;
    /// Returns the description of the image.
    fn get_description(&self) -> String;
}

#[derive(Clone)]
/// A local image (image on the computer).
pub(crate) struct LocalImage {
    pub(crate) path: PathBuf,
    pub(crate) date: Option<DateTime<Local>>,
}

impl Image for LocalImage {
    #[expect(clippy::unwrap_in_result)]
    fn get(config: &Config, _image_data: &mut ImageData) -> Result<Box<Self>, Box<dyn Error>> {
        info!("Getting local images");

        // Get the path to the Pictures directory
        let pictures_dir = Path::new(&config.pictures_folder);

        let local_images = get_images(pictures_dir)?;
        debug!("Found {} local images", local_images.len());

        if local_images.is_empty() {
            return Err(Box::new(NoImagesError));
        }

        let mut rng = rand::rng();

        for _ in 0..10000 {
            // Select a random local image
            #[expect(clippy::unwrap_used)]
            let image_path = local_images.iter().choose(&mut rng).unwrap().clone();
            if is_too_vertical(&image_path) {
                debug!("Skipping {image_path:?} because it's too vertical");
                continue;
            }
            info!("Selecting {image_path:?}");
            return Ok(Box::new(Self::from(image_path)));
        }

        Err(Box::new(NoImagesError))
    }

    fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    fn get_description(&self) -> String {
        // Get the filename and the current date
        let filename = self
            .get_path()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let date = self.date.map(format_date_in_french).unwrap_or_default();

        format!("{filename}\n{date}")
    }
}

impl From<PathBuf> for LocalImage {
    fn from(path: PathBuf) -> Self {
        let mut filename = path
            .with_extension("")
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        for prefix in ["img", "photo"] {
            if let Some(new_filename) = filename.strip_prefix(prefix) {
                debug!("Stripping prefix: {prefix}");
                filename = new_filename.to_string();
                break;
            }
        }
        filename = filename.trim_matches(['_', '-']).to_string();
        if filename.ends_with(')') {
            if let Some(parenthesis_start) = filename.rfind('(') {
                filename = filename[..parenthesis_start].trim_end().to_string();
            }
        }

        let mut date_format = None;
        #[expect(clippy::unwrap_used)]
        if filename.len() == 15 && filename.chars().nth(8).unwrap() == '_' {
            // "19700000_000000.jpg" or "IMG_19700000_000000.jpg"
            date_format = Some("%Y%m%d_%H%M%S");
        } else if filename.len() == 19 && filename.chars().nth(10).unwrap() == '_' {
            // "photo_1970-01-01_00-00-00.jpg"
            date_format = Some("%Y-%m-%d_%H-%M-%S");
        } else if filename.len() == 15 && filename[9..12] == *"-WA" {
            // "IMG-19700101-WA0000.jpg"
            filename = filename[0..8].to_string();
            date_format = Some("%Y%m%d");
        }

        let date: Option<DateTime<Local>> = if let Some(format) = date_format {
            debug!("Parsing date with format: {}", format);
            DateTime::parse_from_str(&filename, format).ok().map(DateTime::<Local>::from)
        } else {
            None
        }.or_else(|| {
            debug!("Getting file metadata");
            metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .map(DateTime::<Local>::from)
        });

        Self { path, date }
    }
}

#[derive(Clone, Deserialize, Serialize)]
/// An online image (image on Unsplash).
pub(crate) struct OnlineImage {
    #[serde(default)]
    pub(crate) id: String,
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub(crate) description: String,
}

impl Image for OnlineImage {
    fn get(config: &Config, image_data: &mut ImageData) -> Result<Box<Self>, Box<dyn Error>> {
        info!("Getting online images");
        // Check if we need to download new images
        if image_data.needs_downloading || image_data.current_index >= image_data.urls.len() {
            info!("Downloading pictures from Unsplash");
            // Download random pictures from Unsplash
            match download_pictures(config) {
                Ok(image_urls) => {
                    // Clear the old images
                    image_data.clear()?;
                    // Store new images and reset current index
                    *image_data = ImageData {
                        urls: image_urls,
                        ..Default::default()
                    };
                    info!("{} pictures downloaded", image_data.urls.len());
                    image_data.store()?;
                }
                Err(err) => {
                    error!("Error: {err}");
                    image_data.needs_downloading = true;
                    image_data.store()?;
                },
            }
        }

        if image_data.current_index >= image_data.urls.len() {
            image_data.current_index = 0;
        }

        // Use the current online image
        let current_image = image_data.urls[image_data.current_index].clone();
        current_image.download()?;

        // Increment the current index and store it
        image_data.current_index += 1;
        debug!("Current index: {}", image_data.current_index);
        image_data.store()?;

        Ok(Box::new(current_image))
    }

    fn get_path(&self) -> PathBuf {
        Paths::downloaded_pictures_dir().join(format!("unsplash_{}.jpg", self.id))
    }

    fn get_description(&self) -> String {
        self.description.clone()
    }
}

impl From<&Value> for OnlineImage {
    fn from(image: &Value) -> Self {
        Self {
            id: image["id"].as_str().unwrap_or_default().to_string(),
            url: image["urls"]["raw"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            date: image["created_at"]
                .as_str()
                .and_then(|date| chrono::DateTime::parse_from_rfc3339(date).ok())
                .map(|date| date.to_utc()),
            description: image["alt_description"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        }
    }
}

impl OnlineImage {
    /// Download an [`OnlineImage`] to its destination file if needed.
    ///
    /// # Errors
    /// Fails if the URL can't be edited or if the destination file can't be written to.
    pub(crate) fn download(&self) -> Result<(), Box<dyn Error>> {
        let image_path = self.get_path();
        if image_path.exists() {
            debug!("Image already exists: {:?}", image_path);
            return Ok(());
        }

        let mut image_url = Url::parse(&self.url)?;
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

        let mut image_file = File::create(image_path)?;
        copy(&mut image_response.into_body().as_reader(), &mut image_file)?;

        Ok(())
    }
}
