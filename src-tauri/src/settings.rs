use std::{collections::HashSet, path::PathBuf};

use serde::{Deserialize, Serialize};

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
