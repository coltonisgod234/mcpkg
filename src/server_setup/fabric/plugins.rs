use std::fs;

use log::{debug, error, info};
use crate::{config::{Config, Version, plugins::ModrinthVersion}, server_setup::download_file};

pub fn version_spec_matches(spec: &Version, version: &ModrinthVersion) -> bool {
    return match spec {
        Version::Any => true,
        Version::AnyOf(strs) => strs.contains(&version.version_number),
        Version::NoneOf(strs) => !strs.contains(&version.version_number),
        Version::Exact(s) => &version.version_number == s,
        Version::FilterList(filters) => {
            for filter in filters {
                if !version_spec_matches(filter, version) {
                    return false;
                }
            }

            return true;
        }
    };
}

pub async fn apply_mod_updates_singlethread(c: &Config) {
    info!("checking for mod updates...");
    let mut updates_found = Vec::new();
    let desired_loader = "fabric".to_string();

    // search for updates, ensuring they are compatible
    let mc_ver = &c.server.version;
    'mods: for plugdef in &c.plugins.mods {
        if let Ok(versions) = plugdef.source.versions().await {
            'versions: for version in &versions {
                debug!("checking {:?}", version);

                // skip this if it's not our desired loader
                if !version.loaders.contains(&desired_loader) {
                    continue 'versions;
                }

                // if it's not supporting our version of Minecraft
                if !version.game_versions.contains(mc_ver) {
                    continue 'versions;
                }

                // if it's not the right version of the mod
                if !version_spec_matches(&plugdef.version, version) {
                    continue 'versions;
                }

                // if they ARE the case we can do it
                info!("found compatible version ({}) of {:?}", version.version_number, plugdef.source);
                updates_found.push(version.clone());

                continue 'mods;
            }
        }

        // if we ever land outside this if-let, it's not compatible
        error!("mod {:?} at version {:?} is imcompatible with Minecraft '{}': aborting update scan", plugdef.source, plugdef.version, mc_ver);
    }

    info!("all mods are compatible, installing...");
    let _ = fs::create_dir_all("mods");

    for update in updates_found {
        let f = &update.files[0];
        info!("downloading {}...", f.url);

        let fpath: String = format!("mods/{}", f.filename);
        download_file(&fpath, &f.url).await;
    }
}


pub async fn apply_mod_updates_mt(c: Config) {
    info!("checking for mod updates...");
    let desired_loader = "fabric".to_string();
    let mc_ver = &c.server.version;

    // Ensure the mods directory exists
    let _ = fs::create_dir_all("mods")
        .expect("the mods folder didnt make craeted akjsh");

    let mut tasks = Vec::new();

    for plugdef in &c.plugins.mods {
        let plugdef = plugdef.clone();
        let mc_ver = mc_ver.clone();
        let desired_loader = desired_loader.clone();

        // Spawn a task per mod
        let handle = tokio::spawn(async move {
            let possible_err_or_maybe_what_we_want = plugdef.source.versions().await; 
            if let Ok(versions) = &possible_err_or_maybe_what_we_want {
                for version in versions {
                    debug!("checking {:?}", version);

                    if !version.loaders.contains(&desired_loader) {
                        continue;
                    }

                    if !version.game_versions.contains(&mc_ver) {
                        continue;
                    }

                    if !version_spec_matches(&plugdef.version, &version) {
                        continue;
                    }

                    info!("found compatible version ({}) of {:?}", version.version_number, plugdef.source);

                    // Start the download immediately
                    if let Some(f) = version.files.get(0) {
                        let fpath = format!("mods/{}", f.filename);
                        info!("downloading {}...", f.url);
                        download_file(&fpath, &f.url).await;
                    }

                    return;  // Stop after first compatible version
                }
            }

            error!(
                "mod {:?} at version {:?} is incompatible with Minecraft '{}': {:?}",
                plugdef.source, plugdef.version, mc_ver, &possible_err_or_maybe_what_we_want
            );
        });

        tasks.push(handle);
    }

    // Wait for all tasks to finish
    let _ = futures::future::join_all(tasks).await;

    info!("all mods are processed!");
}
