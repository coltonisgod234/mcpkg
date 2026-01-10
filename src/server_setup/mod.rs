pub mod fabric;
pub mod paper;

const SERVER_JAR_NAME: &str = "server.jar";

use std::{fs::File, io::copy};
use crate::config::{Config, ServerKind};

pub async fn download_file(output_file: &str, url: &str) {
    let resp = reqwest::get(url).await
        .expect("failed to download file");

    let bytes = resp.bytes().await
        .expect("failed to get bytes for downloading file");

    let mut out = File::create(output_file)
        .expect("failed to create output file for download");

    copy(&mut bytes.as_ref(), &mut out)
        .expect("failed to copy downlaoded data to output file");
}

pub async fn sync_singlethread(c: &Config) {
    match &c.server.kind {
        ServerKind::Fabric { fabric_version, installer_version } => {
            fabric::teardown().await;

            fabric::download_fabric_server_at(
                &c.server.version,
                &fabric_version,
                &installer_version
            ).await;

            fabric::setup_server(c).await
                .expect("failed to set up server");

            fabric::plugins::apply_mod_updates_singlethread(c).await;
        },
        ServerKind::Vanilla => unimplemented!()
    }
}

pub async fn sync_multithread(c: &Config) {
    match &c.server.kind {
        ServerKind::Fabric { fabric_version, installer_version } => {
            // can't multithread this
            fabric::teardown().await;

            // download fabric
            fabric::download_fabric_server_at(
                c.server.version.clone(),
                fabric_version.clone(),
                installer_version.clone(),
            ).await;

            // set up the server configs
            fabric::setup_server(c).await
                .expect("failed to set up server");

            // apply mod updates
            fabric::plugins::apply_mod_updates_mt(
                c.clone()
            ).await;
        },
        ServerKind::Vanilla => unimplemented!()
    }
}
