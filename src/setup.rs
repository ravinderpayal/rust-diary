// use std::fs;
// use std::path::PathBuf;
use chrono::NaiveTime;
// setup.rs
use dialoguer::{Input, Select};
use diary_app::Config;
use std::borrow::Borrow;
use std::error::Error;

use diary_app::StorageType;

use super::iplocation::ipapi::get_ip_location;
// use diary_app::iplocation::ipapi::get_api_location;

use std::env;
use std::io::empty;
use tokio::fs;
use tokio::process::Command;
use winreg::enums::*;
use winreg::RegKey;

use directories::BaseDirs;

pub struct SetupWizard;

impl SetupWizard {
    pub async fn run(current_config: Option<&Config>) -> Result<Config, Box<dyn Error>> {
        println!("Diary App Setup Wizard");

        let mut config = current_config.cloned().unwrap_or_else(Config::new);

        if let Some(current) = current_config {
            println!("Current settings:");
            println!("Storage: {:?}", current.storage_type);
            println!("City: {}", current.city);
            println!("Day starts at: {}", current.day_start_time);
            println!(
                "Editor frequency: {} minutes",
                current.editor_frequency_minutes
            );
            println!("\nPress Enter to keep current value, or input a new value.");
        }

        config.storage_type = Self::prompt_storage_type(current_config.map(|c| &c.storage_type))?;
        let ip_location = get_ip_location().await?;
        let city_default_setting = current_config
            .filter(|c| c.city.len() > 0)
            .map_or(&ip_location, |c| &c.city);
        config.city = Self::prompt_string("City", Some(city_default_setting))?;
        config.day_start_time = Self::prompt_time(
            "Day start time (HH:MM)",
            current_config.map(|c| &c.day_start_time),
        )?;
        config.editor_frequency_minutes = Self::prompt_u32(
            "Editor frequency (minutes)",
            current_config.map(|c| &c.editor_frequency_minutes),
        )?;

        match config.storage_type {
            StorageType::Notion => {
                config.notion_token = Some(Self::prompt_string(
                    "Notion API token",
                    current_config.and_then(|c| c.notion_token.as_ref()),
                )?);
                config.notion_database_id = Some(Self::prompt_string(
                    "Notion database ID",
                    current_config.and_then(|c| c.notion_database_id.as_ref()),
                )?);
            }

            /*StorageType::GoogleDrive => {
                println!("To set up Google Drive, please follow these steps:");
                println!("1. Go to https://developers.google.com/drive/api/v3/quickstart/python");
                println!("2. Click 'Enable the Drive API'");
                println!("3. Download the configuration file");
                println!("4. Copy the content of the file and paste it here:");
                config.google_drive_token = Some(Self::prompt_string("Google Drive credentials", current_config.and_then(|c| c.google_drive_token.as_ref()))?);
            }*/
            StorageType::Local => {
                // No additional setup needed for local storage
            }
        }

        Ok(config)
    }

    fn prompt_storage_type(current: Option<&StorageType>) -> Result<StorageType, Box<dyn Error>> {
        let options = vec!["Local", "Notion", "Google Drive"];
        let default = current
            .map(|s| match s {
                StorageType::Local => 0,
                StorageType::Notion => 1,
                // StorageType::GoogleDrive => 2,
            })
            .unwrap_or(0);

        let selected = Select::new()
            .with_prompt("Choose your preferred storage method")
            .items(&options)
            .default(default)
            .interact()?;

        Ok(match selected {
            0 => StorageType::Local,
            1 => StorageType::Notion,
            // 2 => StorageType::GoogleDrive,
            _ => unreachable!(),
        })
    }

    fn prompt_string(prompt: &str, current: Option<&String>) -> Result<String, Box<dyn Error>> {
        let input: String = Input::new()
            .with_prompt(prompt)
            .with_initial_text(current.map(|s| s.as_str()).unwrap_or(""))
            .allow_empty(true)
            .interact_text()?;

        Ok(if input.is_empty() && current.is_some() {
            current.unwrap().clone()
        } else {
            input
        })
    }

    fn prompt_time(prompt: &str, current: Option<&NaiveTime>) -> Result<NaiveTime, Box<dyn Error>> {
        loop {
            let input: String = Input::new()
                .with_prompt(prompt)
                .with_initial_text(
                    current
                        .map(|t| t.format("%H:%M").to_string())
                        .unwrap_or_default(),
                )
                .allow_empty(true)
                .interact_text()?;

            if input.is_empty() && current.is_some() {
                return Ok(current.unwrap().clone());
            }

            if let Ok(time) = NaiveTime::parse_from_str(&input, "%H:%M") {
                return Ok(time);
            }

            println!("Invalid time format. Please use HH:MM.");
        }
    }

    fn prompt_u32(prompt: &str, current: Option<&u32>) -> Result<u32, Box<dyn Error>> {
        loop {
            let input: String = Input::new()
                .with_prompt(prompt)
                .with_initial_text(current.map(|n| n.to_string()).unwrap_or_default())
                .allow_empty(true)
                .interact_text()?;

            if input.is_empty() && current.is_some() {
                return Ok(*current.unwrap());
            }

            if let Ok(number) = input.parse::<u32>() {
                return Ok(number);
            }

            println!("Invalid number. Please enter a positive integer.");
        }
    }
}

pub async fn add_auto_start_entry() -> Result<(), Box<dyn Error>> {
    if cfg!(windows) {
        println!("Adding auto start entry for windows if not present.");
        add_windows_auto_start_entry()
    } else if cfg!(unix) {
        add_unix_auto_start_entry().await
    } else {
        println!("Your operating system is not in support yet for this app.");
        Ok(())
    }
}

fn add_windows_auto_start_entry() -> Result<(), Box<dyn Error>> {
    let entry_key = "da-desktop";

    let mut exe_path = env::current_exe()?;
    exe_path.set_extension(""); // Remove any extension like `.exe`

    let hku = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _disp) = hku.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;

    key.set_value(&entry_key, &exe_path.to_str().unwrap())?;

    println!("Application added to startup with entry {}.", &entry_key);
    Ok(())
}

async fn add_unix_auto_start_entry() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(base_dirs) = BaseDirs::new() {
        let autostart_dir = base_dirs.config_dir().join("autostart");
        std::fs::create_dir_all(&autostart_dir)?;

        let desktop_path = autostart_dir.join("com.ravinderpayal.DiaryApp.desktop");
        if !desktop_path.exists() {
            let bin_path: String = env::current_exe()?.to_str().unwrap().to_string();
            let deskop_entry = format!(
                r#"[Desktop Entry]
Type=Application
Exec={}
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
Name=Diary App
Comment[en_IN]=
Comment="#,
                bin_path
            );
            // Write the service file
            fs::write("/tmp/da-desktop.entry", deskop_entry).await?;

            // Move the service file to the correct location and set permissions
            println!("Adding Desktop Entry at {}", desktop_path.to_str().unwrap());
            Command::new("sudo")
                .args(&[
                    "mv",
                    "/tmp/da-desktop.entry",
                    desktop_path
                        .to_str()
                        .expect("Desktop Path vanished from memory, lol!!"),
                    // "~/.config/autostart/com.ravinderpayal.DiaryApp.desktop",
                ])
                .status()
                .await?;

            Command::new("sudo")
                .args(&["chmod", "644", desktop_path.to_str().unwrap()])
                .status()
                .await?;
            println!("Diary service has been set up and started.");
        }
    }
    Ok(())
}
