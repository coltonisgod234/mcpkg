pub mod plugins;

use std::{fmt::Display, fs::{self, OpenOptions, remove_dir_all, remove_file}, io::{self, Write}};
use log::{info, warn};
use crate::{config::{Config, ConfigReloadMethod}, server_setup::download_file};

const SERVER_JAR_NAME: &str = "server.jar";

pub async fn download_fabric_server_at<T: Display>(minecraft_version: T, fabric_version: T, installer_version: T) {
    info!("downloading fabric server (minecraft {}, fabric {}, installer {})", minecraft_version, fabric_version, installer_version);

    // probably vulnerable..
    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar",
        minecraft_version,
        fabric_version,
        installer_version
    );

    download_file(SERVER_JAR_NAME, &url).await;
}

pub fn gen_server_props(c: &Config) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("server.properties")
        .unwrap();

    writeln!(file, "server-port={}", c.server.port)?;
    writeln!(file, "server-ip={}", c.server.ip)?;
    writeln!(file, "gamemode={}", c.server.props.gamemode.to_string())?;
    writeln!(file, "difficulty={}", c.server.props.difficulty.to_string())?;

    if (c.management.conf_reload_method == ConfigReloadMethod::RconSync)
        && (c.management.rcon.enable == false)
    {
        panic!("in order to use the `RconSync` reload method, you need to enable rcon.")
    }

    writeln!(file, "enable-rcon={}", c.management.rcon.enable)?;
    writeln!(file, "rcon.port={}", c.management.rcon.port)?;
    writeln!(file, "rcon.password={}", c.management.rcon.pass)?;

    writeln!(file, "level-name={}", c.world.path)?;
    writeln!(file, "level-seed={}", c.world.seed)?;

    return Ok(())
}

pub async fn setup_server(c: &Config) -> io::Result<()> {
    // write eula.txt
    info!("writing eula.txt");
    fs::write("eula.txt", "eula=true")?;

    // write server.properties
    info!("generating server properties...");
    gen_server_props(c)?;

    return Ok(())
}

pub async fn teardown() {
    const REMOVE_DIRS: [&str; 4] = ["libraries", ".fabric", "mods", "versions"];
    //const REMOVE_FILE: [&str; 8] = ["banned-ips.json", "banned-players.json", "eula.txt", "ops.json", SERVER_JAR_NAME, "server.properties", "usercache.json", "whitelist.json"];
    const REMOVE_FILE: [&str; 3] = ["eula.txt", SERVER_JAR_NAME, "server.properties"];

    for dir in REMOVE_DIRS {
        if let Err(e) = remove_dir_all(dir) {
            warn!("error removing [{:20}]. this is normal if you have no server set up: {}", dir, e)
        }
    }

    for file in REMOVE_FILE {
        if let Err(e) = remove_file(file) {
            warn!("error removing [{:20}]. this is normal if you have no server set up: {}", file, e)
        }
    }
}
