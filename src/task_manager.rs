// task_manager.rs
use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read};
use std::path::Path;
use chrono::Local;
use std::process::Command;

pub struct TaskManager {
    dir: String,
}

impl TaskManager {
    pub fn new(dir: &str) -> Self {
        TaskManager {
            dir: dir.to_string(),
        }
    }

    pub fn prompt_tasks(&self) {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let file_path = format!("{}/{}.md", self.dir, date);

        if !Path::new(&file_path).exists() {
            let mut file = File::create(&file_path).expect("Failed to create tasks file");

            writeln!(file, "# Tasks for {}", date).expect("Failed to write to tasks file");

            Command::new("x-terminal-emulator")
                .arg("-e")
                .arg("vim")
                .arg(&file_path)
                .status()
                .expect("Failed to open vim for tasks in foreground");

            println!("Tasks saved to {}", file_path);
        }
    }
}
