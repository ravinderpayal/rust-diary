// diary.rs
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use chrono::Local;
use std::process::Command;

pub struct Diary {
    dir: String,
}

impl Diary {
    pub fn new(dir: &str) -> Self {
        Diary {
            dir: dir.to_string(),
        }
    }

    pub fn page_exists(&self) -> bool {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let file_path = format!("{}/{}.md", self.dir, date);
        Path::new(&file_path).exists()
    }

    pub fn new_page(&self) {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let file_path = format!("{}/{}.md", self.dir, date);

        if !Path::new(&file_path).exists() {
            let mut file = File::create(&file_path).expect("Failed to create new page");
            writeln!(file, "# {}", date).expect("Failed to write to new page");
        }
    }

    pub fn collect_thoughts(&self) {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let file_path = format!("{}/{}.md", self.dir, date);
        let time = Local::now().format("%H:%M").to_string();

        let mut file = OpenOptions::new()
            .append(true)
            .open(&file_path)
            .expect("Failed to open diary file");

        writeln!(file, "\n## {}", time).expect("Failed to write time heading");

        Command::new("x-terminal-emulator")
            .arg("-e")
            .arg("vim")
            .arg(&file_path)
            .status()
            .expect("Failed to open vim for tasks in foreground");
    }
}

