// lib.rs
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[async_trait(?Send)]
pub trait Storage {
    async fn save_entry(&self, date: NaiveDate, content: &str) -> Result<(), Box<dyn Error>>;
    async fn get_entry(&self, date: NaiveDate) -> Result<Option<String>, Box<dyn Error>>;
    async fn get_latest_entry(&self) -> Result<Option<(NaiveDate, String)>, Box<dyn Error>>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StorageType {
    Local,
    Notion,
   // GoogleDrive,
}
/*
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub storage_type: StorageType,
    pub city: String,
    pub notion_token: Option<String>,
    pub notion_database_id: Option<String>,
    pub google_drive_token: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            storage_type: StorageType::Local,
            city: "Gurugram, Hr".to_string(),
            notion_token: None,
            notion_database_id: None,
            google_drive_token: None,
        }
    }
}
*/

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub storage_type: StorageType,
    pub city: String,
    pub notion_token: Option<String>,
    pub notion_database_id: Option<String>,
    pub google_drive_token: Option<String>,
    pub day_start_time: NaiveTime,
    pub editor_frequency_minutes: u32,
}

impl Config {
    pub fn new() -> Self {
        Config {
            storage_type: StorageType::Local,
            city: "Gurugram, Hr".to_string(),
            notion_token: None,
            notion_database_id: None,
            google_drive_token: None,
            day_start_time: NaiveTime::from_hms_opt(5, 30, 0).unwrap(),
            editor_frequency_minutes: 60,
        }
    }
}
