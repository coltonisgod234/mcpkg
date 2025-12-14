use std::fs;

use log::{debug, error, info};

use crate::{config::{Config, Version, plugins::ModrinthVersion}, server_setup::download_file};

// pub fn version_spec_matches(spec: &Version, possible_versions: &ModrinthVersion) -> bool {
//     return match spec {
//         Version::Exact(v) => possible_versions.contains(v),
//         Version::AnyOf(v) => possible_versions.iter().any(|x| v.contains(x)),  // slow
//         Version::NoneOf(v) => !possible_versions.iter().any(|x| v.contains(x)),
//         Version::FilterList(filters) => {
//             for filter in filters {
//                 if !version_spec_matches(filter, possible_versions) {
//                     return false
//                 }
//             }

//             return true
//         },
//         Version::Any => true
//     }
// }

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

pub async fn apply_mod_updates(c: &Config) {
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