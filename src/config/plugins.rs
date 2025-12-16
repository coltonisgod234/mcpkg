use reqwest::Error;
use serde::{Deserialize, Serialize};

use crate::config::Version;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ModSource {
    Modrinth(String)
}

impl ModSource {
    pub async fn versions(&self) -> Result<Vec<ModrinthVersion>, Error> {
        match self {
            Self::Modrinth(id) => {
                let url = format!("https://api.modrinth.com/v2/project/{}/version", id);
                let resp = reqwest::get(&url).await?;
                let versions: Vec<ModrinthVersion> = resp.json().await?;

                return Ok(versions)
            }
        }
    }

    pub async fn version_change_since(&self, old_version: &str) -> Result<bool, Error> {
        let vers = self.versions().await?;

        let latest = &vers[0];
        if latest.version_number != old_version {
            return Ok(true)
        }

        return Ok(false);
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Mod {
    pub source: ModSource,
    
    #[serde(default)]
    pub version: Version,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModsConfig {
    pub mods: Vec<Mod>
}

// modrinth's API structures
#[derive(Deserialize, Debug, Clone)]
pub struct ModrinthVersion {
    pub version_number: String,
    pub game_versions: Vec<String>,

    pub files: Vec<ModrinthFile>,
    pub date_published: String,

    pub loaders: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,

    #[serde(default)]
    pub loader: Option<String>,
}