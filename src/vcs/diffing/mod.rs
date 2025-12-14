use std::{collections::HashMap, fs, io::{self, Read}, path::{Path, PathBuf}};

use walkdir::WalkDir;

use crate::vcs::structures::Change;

/// returns the BLAKE3 hash of a file at `path`
fn hash_file(path: &PathBuf) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192]; // 8 KB buffer, adjust if you want

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    return Ok(hasher.finalize().to_hex().to_string())
}

/// Used to avoid typing a bunch of shit each time
pub type RelAbsMap = HashMap<PathBuf, PathBuf>;

// I'm too lazy to code most of this so I'm just gonna vibe code it.
// it's just boilerplate anyway.

/// Returns a map of `relative path -> absolute path`.
/// Does not include directories.
// pub fn create_relabsmap(root: &Path, ignore: &Vec<&Path>) -> RelAbsMap {
//     return WalkDir::new(root)
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(|e| e.file_type().is_file())  // only iterate over files, not directories
//         .filter(|e| !ignore.contains(&e.path()))  // skip all files that were ignored
//         .map(|e| {
//             let rel = e.path().strip_prefix(root).unwrap().to_path_buf();
//             return (rel, e.path().to_path_buf());
//         })
//         .collect()
// }
pub fn create_relabsmap(root: &Path, ignore: &Vec<&Path>) -> RelAbsMap {
    let mut map = RelAbsMap::new();

    // Convert ignore paths to absolute paths relative to root
    let ignore_paths: Vec<PathBuf> = ignore.iter().map(|p| root.join(p)).collect();

    for entry in WalkDir::new(root) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // skip unreadable entries
        };

        // Skip ignored directories and their contents
        if ignore_paths.iter().any(|p| entry.path().starts_with(p)) {
            if entry.file_type().is_dir() {
                // Tell WalkDir not to descend into this directory
                continue;
            } else {
                // Skip ignored files
                continue;
            }
        }

        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(root).unwrap().to_path_buf();
            map.insert(rel, entry.path().to_path_buf());
        }
    }

    map
}

pub fn diff_binary_dirs(
    a: &RelAbsMap,
    b: &RelAbsMap,
) -> Vec<Change> {
    let mut changes = Vec::new();

    for (rel, a_path) in a {
        match b.get(rel) {
            None => changes.push(Change::Remove(rel.clone())),
            Some(b_path) => {
                let a_hash = hash_file(a_path).unwrap();
                let b_hash = hash_file(b_path).unwrap();
                if a_hash != b_hash {
                    changes.push(Change::Modify(rel.clone()));
                }
            }
        }
    }

    for rel in b.keys() {
        if !a.contains_key(rel) {
            changes.push(Change::Create(rel.clone()));
        }
    }

    return changes
}

pub fn dir(a: &Path, b: &Path, ignore: &Vec<&Path>) -> Vec<Change> {
    let am = create_relabsmap(a, ignore);
    let bm = create_relabsmap(b, ignore);
    return diff_binary_dirs(&am, &bm);
}

pub fn say_everything_was_added(at: &Path, ignore: &Vec<&Path>) -> Vec<Change> {
    let mut changes = Vec::new();

    // Convert ignore paths to absolute paths relative to root
    let ignore_paths: Vec<PathBuf> = ignore.iter().map(|p| at.join(p)).collect();

    let mut walker = WalkDir::new(at).into_iter();

    while let Some(entry) = walker.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // skip unreadable entries
        };

        // If this entry is in an ignored path, skip it
        if ignore_paths.iter().any(|p| entry.path().starts_with(p)) {
            if entry.file_type().is_dir() {
                walker.skip_current_dir(); // stops descending into this directory
            }
            continue; // skip this file or directory
        }

        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(at).unwrap().to_path_buf();
            changes.push(Change::Create(rel));
        }
    }

    return changes
}