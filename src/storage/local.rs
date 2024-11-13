// storage/local.rs
use async_trait::async_trait;
use chrono::NaiveDate;
use diary_app::Storage;
use std::error::Error;
use std::fmt::Debug;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use tracing::debug;

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: PathBuf) -> Self {
        LocalStorage { base_path }
    }
}

#[async_trait(?Send)]
impl Storage for LocalStorage {
    async fn save_entry(&self, date: NaiveDate, content: &str) -> Result<(), Box<dyn Error>> {
        let file_path = self.base_path.join(format!("{}.md", date));
        // Create the directory if it does not exist(not created automatically in windows).
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Ok(())
    }

    async fn get_entry(&self, date: NaiveDate) -> Result<Option<String>, Box<dyn Error>> {
        let file_path = self.base_path.join(format!("{}.md", date));
        if file_path.exists() {
            let mut content = String::new();
            File::open(file_path)?.read_to_string(&mut content)?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    async fn get_latest_entry(&self) -> Result<Option<(NaiveDate, String)>, Box<dyn Error>> {
        let mut entries: Vec<_> = fs::read_dir(&self.base_path)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext == "md")
                    .unwrap_or(false)
            })
            .collect();

        entries.sort_by_key(|entry| entry.path());

        if let Some(latest) = entries.last() {
            let date = NaiveDate::parse_from_str(
                latest.path().file_stem().unwrap().to_str().unwrap(),
                "%Y-%m-%d",
            )?;
            let mut content = String::new();
            File::open(latest.path())?.read_to_string(&mut content)?;
            Ok(Some((date, content)))
        } else {
            Ok(None)
        }
    }
}
