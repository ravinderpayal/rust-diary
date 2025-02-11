// main.rs
mod config;
mod iplocation;
mod setup;
mod storage;
mod weather;

use chrono::Local;
use clap::{App, Arg};
use config::ConfigManager;
// use daemonize::Daemonize;
use dirs;
use iplocation::ipapi::get_ip_location;
use openssl::version::dir;
use setup::{add_auto_start_entry, SetupWizard};
use tracing::debug;

use storage::{local::LocalStorage, notion::NotionStorage};
use weather::open_weather::OpenWeatherService;

use diary_app::{Storage, StorageType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("The |\\|/\\R\\| /\\PP");

    let matches = App::new("The Journal")
        .version("1.0")
        .author("Ravinder Payal")
        .about("A diary which you can't forget, don't need to locate/find, it's where ever you are!! Download and setup it on the machines you use.")
        .arg(Arg::with_name("config")
             .long("config")
             .help("Run configuration setup"))
        .get_matches();

    if matches.is_present("config") {
        let existing_config = ConfigManager::load().unwrap_or_else(|_| None);
        let new_config = SetupWizard::run(existing_config.as_ref()).await?;
        ConfigManager::save(&new_config)?;
        println!("Configuration updated successfully.");
        // return Ok(());
        // Let's keep running the app but again an instance might be running alread
        // Let's add code to check if an instance is already running
        // If not, continue from here
    }

    let config_load_attempt = ConfigManager::load();

    let config = match config_load_attempt {
        Ok(Some(config)) => {
            println!("Using Existing Configuration");
            config
        }
        _ => {
            println!("Running Setup Wizard  :)");
            let new_config = SetupWizard::run(None).await?;
            ConfigManager::save(&new_config)?;
            new_config
        }
    };

    println!("Setup Done");
    // Check if this is the first run and set up the service if needed
    println!("Regisering desktop auto start entry");
    add_auto_start_entry()
        .await
        .expect("Failed to set up Diary Service");

    let storage: Box<dyn Storage> = match config.storage_type {
        StorageType::Local => Box::new(LocalStorage::new(
            dirs::home_dir()
                .expect("Home Directory Not Found")
                .join("Diary"),
        )),
        StorageType::Notion => Box::new(NotionStorage::new(
            config.notion_token.unwrap(),
            config.notion_database_id.unwrap(),
        )),
        // StorageType::GoogleDrive => Box::new(GoogleDriveStorage::new(config.google_drive_token.unwrap())),
    };

    println!("Got hold of Storage _/");
    /*
    // Daemonize the process
    let daemonize = Daemonize::new()
        .pid_file("/tmp/diary_app.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .user("nobody")
        .group("daemon")
        .privileged_action(|| println!("Diary App is now running in the background."));

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("Error, {}", e),
    }*/

    //
    // let dbus_connection = Connection::new_session()?;
    // let dbus_proxy = dbus_connection.with_proxy("org.example.DiaryApp", false, true, false);
    // let mut dbus_cr = Crossroads::new();
    // dbus_connection.request_name("org.example.DiaryApp", false, true, false)?;
    // let token = dbus_cr.register("com.ravinderpayal.DiaryApp", |b| {
    //    b.method(
    //        "LaunchEditor",
    //        (),
    //        (),
    //        |_: dbus_crossroads::Context, _, _| {
    //            // Signal to launch editor
    //            Ok(())
    //        },
    //    );
    // });

    // dbus_cr.serve(&dbus_connection)?;
    let mut weather_service = match std::env::var("WEATHER_API_KEY") {
        Ok(api_key) => Some(OpenWeatherService::new(&config.city, &api_key)),
        Err(_) => None,
    };

    loop {
        let now = Local::now();
        let today = now.date_naive();
        //  if now.hour() == 5 && now.minute() == 30 || now.hour() > 5 {
        if storage.get_entry(today).await?.is_none() {
            let mut city = get_ip_location().await?;
            if city.len() > 0 {
                weather_service = match std::env::var("WEATHER_API_KEY") {
                    Ok(api_key) => Some(OpenWeatherService::new(&city, &api_key)),
                    Err(_) => None,
                };
            } else {
                city = config.city.clone();
            }

            let weather = match &weather_service {
                Some(s) => s.get_weather()?,
                None => "API Not Configured".to_string(),
            };
            let content = format!(
                    "🌆 City: {}\n🌤️ Weather: {}\n# Tasks for today\n- [ ] Eat Healthy\n- [ ] Workout\n- [ ] Talk to someone", city, weather
                );
            launch_editor(&storage, content).await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(
                (60 * config.editor_frequency_minutes).into(),
            ))
            .await;
        }
        // }
        else {
            // Collect thoughts
            let entry = storage.get_entry(today).await?.unwrap_or_default();
            println!("Existing Content:`{}`", &entry);
            let new_content = format!("{}\n\n## {}\n\n", entry, now.format("%H:%M"));

            // let temp_file = tempfile::NamedTempFile::new()?;
            // std::fs::write(temp_file.path(), new_content)?;
            // Open the default text editor

            // dbus_connection.send(dbus::Message::new_method_call(
            //    "com.ravinderpayal.DiaryApp",
            //    "/org/example/DiaryApp",
            //    "com.ravinderpayal.DiaryApp",
            //    "LaunchEditor",
            // )?);

            // let updated_content = std::fs::read_to_string(temp_file.path())?;

            launch_editor(&storage, new_content).await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(
                (60 * config.editor_frequency_minutes).into(),
            ))
            .await;
        }
    }
}

async fn launch_editor(
    storage: &Box<dyn Storage>,
    new_content: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = Local::now();
    let today = now.date_naive();

    let mut temp_file = tempfile::NamedTempFile::new()?;
    std::fs::write(temp_file.path(), new_content)?;

    // ensure data is properly saved to temp location before passing control to editor
    temp_file.as_file().sync_all()?;

    // Open the default text editor
    let exit_status = if cfg!(target_os = "windows") {
        println!("Opening Notepad for Windows");
        std::process::Command::new("notepad")
            .arg(temp_file.path())
            .status()
            .expect("Notepad didn't closed successfully!");
    } else {
        println!("Opening editor for recording regular response");
        std::process::Command::new("x-terminal-emulator")
            .arg("-e")
            .arg("vim")
            .arg(temp_file.path())
            .env("DISPLAY", ":0".to_string())
            .status()
            .expect("Failed to open vim for recoriding input");
        /*std::process::Command::new("x-terminal-emulator")
        .arg("-e")
        .arg("vim")
        .arg(temp_file.path())
        .status()
        .expect("Failed to open vim for tasks in foreground");*/
    };

    let updated_content = std::fs::read_to_string(temp_file.path())?;
    storage.save_entry(today, &updated_content).await?;
    Ok(())
}
