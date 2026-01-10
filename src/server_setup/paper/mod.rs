use crate::{config::Config, server_setup::{SERVER_JAR_NAME, download_file, fabric::gen_server_props}};

pub async fn download_paper_server_at(minecraft_version: &str, paper_build: &str) {
    log::info!("installing paper {} at {}", minecraft_version, paper_build);

    let url = format!(
        "https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/paper-{}-{}.jar",
        &minecraft_version,
        &paper_build,
        &minecraft_version,
        &paper_build
    );

    download_file(SERVER_JAR_NAME, &url).await;
}

pub async fn setup_server(c: &Config, paper_build: &str) {
    download_paper_server_at(&c.server.version, paper_build).await;
}

pub async fn install_plugins(c: &Config) {
    gen_server_props(c)
        .expect("failed");
}
