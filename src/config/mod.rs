use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use crate::config::plugins::ModsConfig;
#[derive(Debug, Deserialize, Serialize)]
pub enum Version {
    Exact(String),
    AnyOf(Vec<String>),
    NoneOf(Vec<String>),
    FilterList(Vec<Version>),
    Any,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ServerKind {
    Fabric {
        fabric_version: String,
        installer_version: String
    },
    Vanilla
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UpdateMode {
    AllCompatible,
    Instant
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    // software info
    pub kind: ServerKind,
    pub version: String,
    pub restart_every: Duration,
    pub update_mode: UpdateMode,

    // runtime info
    pub max_memory: String,

    // server hosting info
    pub name: String,
    pub port: u16,

    #[serde(default)]
    pub ip: String,

    #[serde(default)]
    pub motd: String,

    pub props: ServerProps
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorldConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerProps {
    pub difficulty: Difficulty,
    pub gamemode: Gamemode,
    pub is_hardcore: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManagementConfig {
    pub rcon: RconConfig,
    pub conf_reload_method: ConfigReloadMethod
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ConfigReloadMethod {
    RconSync,
    RestartServer,
    NoReload
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RconConfig {
    #[serde(default)]
    pub port: u16,

    #[serde(default)]
    pub pass: String,

    pub enable: bool
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Medium,
    Hard
}

impl ToString for Difficulty {
    fn to_string(&self) -> String {
        return match self {
            Self::Peaceful => "peaceful",
            Self::Easy => "easy",
            Self::Medium => "medium",
            Self::Hard => "hard",
        }.to_string()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Gamemode {
    Survival,
    Creative,
    Adventure,
    Spectator
}

impl ToString for Gamemode {
    fn to_string(&self) -> String {
        return match self {
            Self::Survival => "survival",
            Self::Creative => "creative",
            Self::Spectator => "spectator",
            Self::Adventure => "adventure",
        }.to_string()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub management: ManagementConfig,
    pub world: WorldConfig,
    pub plugins: ModsConfig
}

pub mod plugins;

pub fn parse_file(p: &str) -> Config {
    let yaml = ::std::fs::read_to_string(p)
        .expect("failed to open config file!");

    return serde_yaml::from_str(&yaml)
        .expect("failed to parse config file");
}
