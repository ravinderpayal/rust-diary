// config.rs
use diary_app::Config;
use directories::ProjectDirs;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub struct ConfigManager;

impl ConfigManager {
    fn config_path() -> PathBuf {
        let proj_dirs = ProjectDirs::from("com", "DiaryApp", "DiaryApp")
            .expect("Failed to get project directories");
        proj_dirs.config_dir().join("config.json")
    }

    pub fn load() -> Result<Option<Config>, Box<dyn Error>> {
        let config_path = Self::config_path();
        if config_path.exists() {
            println!("Config path exists.");
            let config_str = fs::read_to_string(config_path)?;
            Ok(Some(serde_json::from_str(&config_str)?))
        } else {
            eprintln!("Config path doesn't exists.");
            Ok(None)
        }
    }

    pub fn save(config: &Config) -> Result<(), Box<dyn Error>> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let config_str = serde_json::to_string_pretty(config)?;
        fs::write(config_path, config_str)?;
        Ok(())
    }
}
