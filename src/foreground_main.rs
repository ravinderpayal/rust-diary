mod config;
mod setup;
mod storage;
mod weather;

use chrono::Local;
use config::ConfigManager;
// use daemonize::Daemonize;
use dirs;
use dbus::blocking::Connection;

use diary_app::Storage;
use diary_app::StorageType;
use storage::local::LocalStorage;
use storage::notion::NotionStorage;
use weather::open_weather::OpenWeatherService;


/**
 * @deprecated
 * 
 * I couldn't figure out how to run foreground UI from a service running in background
 *  If I can't after one more try, will remove this code
 *
 *
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_not_found_message =
        "Run it after configuration. Run Main application with --config flag.";

    let config = ConfigManager::load()
        .expect(config_not_found_message)
        .expect(config_not_found_message);

    let c = Connection::new_session()?;
    // c.add_match("interface='org.example.DiaryApp',member='LaunchEditor'")?;

    // for item in c.iter(0) {
    //    if let dbus::Message::MethodCall(ref m) = item {
    //        if m.member().map(|s| s == "LaunchEditor").unwrap_or(false) {
    //        }
    //    }
    // }

    Ok(())
}



