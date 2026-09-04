use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub schema_version: u32,
    pub automation_enabled: bool,
    pub combat_limit_bytes_per_second: u64,
    pub throttle_while_alive: bool,
    pub download_while_dead: bool,
    pub download_between_rounds: bool,
    pub pause_during_other_modes: bool,
    pub restore_on_exit: bool,
    pub start_with_windows: bool,
    pub close_to_tray: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: 2,
            automation_enabled: false,
            combat_limit_bytes_per_second: 10_000_000,
            throttle_while_alive: true,
            download_while_dead: true,
            download_between_rounds: true,
            pause_during_other_modes: true,
            restore_on_exit: true,
            start_with_windows: false,
            close_to_tray: true,
        }
    }
}

pub fn app_dir() -> PathBuf {
    env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("SteamThrottle")
}

pub fn load() -> AppSettings {
    let mut settings: AppSettings = fs::read_to_string(app_dir().join("settings.json"))
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();
    settings.schema_version = 2;
    settings
}

pub fn save(settings: &AppSettings) -> io::Result<()> {
    let directory = app_dir();
    fs::create_dir_all(&directory)?;
    let path = directory.join("settings.json");
    let temporary = directory.join("settings.json.tmp");
    let json = serde_json::to_vec_pretty(settings).map_err(io::Error::other)?;
    fs::write(&temporary, json)?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    fs::rename(temporary, path)
}
