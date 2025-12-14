use log::info;
use rcon::{AsyncStdStream};

use crate::config::{Config, ConfigReloadMethod};

async fn rcon_connect(c: &Config) -> rcon::Result<rcon::Connection<AsyncStdStream>> {
    let rcon_ip = format!("127.0.0.1:{}", c.management.rcon.port);

    let rconn = <rcon::Connection<AsyncStdStream>>::builder()
        .enable_minecraft_quirks(true)
        .connect(rcon_ip, &c.management.rcon.pass)
        .await?;

    return Ok(rconn)
}

pub async fn reload_rcon(c: &Config) -> rcon::Result<()> {
    info!("reloading via rcon");

    let mut conn = rcon_connect(c).await?;

    conn.cmd("/reload").await?;
    conn.cmd(&format!("/difficulty {}", c.server.props.difficulty.to_string())).await?;

    return Ok(())
}

pub async fn reload(c: &Config) {
    match c.management.conf_reload_method {
        ConfigReloadMethod::NoReload => info!("nothing to do. Reloading is disabled"),
        ConfigReloadMethod::RconSync => reload_rcon(c).await.expect("failed to reload via rcon"),
        ConfigReloadMethod::RestartServer => unimplemented!()
    }
}
