use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::error::Error;
use std::fs::metadata;
use std::path::PathBuf;

use crate::date_format::format_date_in_french;

/// An image that has a path and a description.
pub(crate) trait Image {
    fn get_path(&self) -> Result<PathBuf, Box<dyn Error>>;
    fn get_description(&self) -> String;
}

#[derive(Clone, Deserialize, Serialize)]
/// A local image (image on the computer).
pub(crate) struct LocalImage {
    pub(crate) path: PathBuf,
    pub(crate) date: DateTime<Utc>,
}

impl Image for LocalImage {
    fn get_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(self.path.clone())
    }
    fn get_description(&self) -> String {
        // Get the filename and the current date
        let filename = self
            .get_path()
            .map(|path| {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_default();
        let date = format_date_in_french(self.date.into());

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
                filename = new_filename.to_string();
                break;
            }
        }
        filename = filename.trim_matches(['_', '-']).to_string();

        let mut date_format = None;
        #[expect(clippy::unwrap_used)]
        if filename.len() == 15 && filename.chars().nth(8).unwrap() == '_' {
            // "19700000_000000.jpg" or "IMG_19700000_000000.jpg"
            date_format = Some("%Y%m%d_%H%M%S");
        } else if filename.len() == 19 && filename.chars().nth(10).unwrap() == '_' {
            // "photo_1970-01-01_00-00-00.jpg"
            date_format = Some("%Y-%m-%d_%H-%M-%S");
        } else if filename.len() == 15 && filename[12..15] == *"-WA" {
            // "IMG-19700101-WA0000.jpg"
            filename = filename[0..12].to_string();
            date_format = Some("%Y%m%d");
        }

        let date = if let Some(format) = date_format {
            DateTime::parse_from_str(&filename, format).ok()
        } else {
            None
        }
        .map_or_else(
            || DateTime::from(metadata(&path).unwrap().modified().unwrap()),
            |date| date.to_utc(),
        );

        Self { path, date }
    }
}

#[derive(Clone, Deserialize, Serialize)]
/// An online image (image on Unsplash).
pub(crate) struct OnlineImage {
    pub(crate) id: String,
    pub(crate) url: String,
    pub(crate) date: Option<DateTime<Utc>>,
    pub(crate) description: String,
}

impl Image for OnlineImage {
    fn get_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(dirs::cache_dir()
            .ok_or("Could not find the local data directory")?
            .join(format!("wallpaper-changer-rs/unsplash_{}.jpg", self.id)))
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

// #[derive(Clone, Deserialize, Serialize)]
// pub(crate) enum Image {
//     Local(LocalImage),
//     Online(OnlineImage),
// }
