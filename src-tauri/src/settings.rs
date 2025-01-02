use std::{collections::HashSet, path::PathBuf};

use serde::{Deserialize, Serialize};

pub const SETTINGS_FILENAME: &str = "settings.json";
pub const AUDIO_SOURCES_SETTING: &str = "audio-sources";
pub const BOT_TOKEN_SETTING: &str = "bot-token";

pub const TRACKS_FILENAME: &str = "tracks.json";
pub const TRACKS_SETTING: &str = "tracks";

pub const DISCORD_FILENAME: &str = "discord.json";
pub const GUILDS_SETTING: &str = "guilds";
pub const CHANNELS_SETTING: &str = "channels";

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub audio_sources: HashSet<AudioSource>,
}

impl Default for AppSettings {
    fn default() -> Self {
        return AppSettings {
            audio_sources: HashSet::new(),
        };
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct AudioSource {
    pub path: PathBuf,
    pub recursive: bool,
    pub active: bool,
}

impl AudioSource {
    pub fn from_path(path: PathBuf) -> Self {
        return AudioSource {
            path,
            recursive: false,
            active: true,
        };
    }
}

impl PartialOrd for AudioSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AudioSource {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let maybe_name1 = self.path.file_stem();
        let maybe_name2 = other.path.file_stem();
        if let None = maybe_name1 {
            return std::cmp::Ordering::Less;
        }
        if let None = maybe_name2 {
            return std::cmp::Ordering::Greater;
        }
        return maybe_name1.unwrap().cmp(maybe_name2.unwrap());
    }
}
