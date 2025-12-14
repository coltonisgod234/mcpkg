//! format version: 0
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::vcs::MCPKG_FOLDER;

/// A repository on the computer's physical disk
#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    // ALWAYS have this please!
    pub _config_version: usize,

    pub ignored_files: Vec<PathBuf>,
    
    pub revisions: Vec<Revision>,
}

impl Default for Repository {
    fn default() -> Self {
        return Self {
            _config_version: 0,
            revisions: Vec::new(),
            ignored_files: vec![
                // keep these in prod future me plz ok thx
                PathBuf::from(MCPKG_FOLDER),
                PathBuf::from(".gitignore"),
                PathBuf::from(".git"),

                // for tests, REMOVE IN PROD
                PathBuf::from("target"),
                PathBuf::from("src"),
                PathBuf::from("Cargo.lock"),
                PathBuf::from("Cargo.toml"),
            ],
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackedFile {
    /// The original path
    pub p_original: String,

    /// The path of the currently stored copy of the file from the
    /// revision's folder.
    pub p_stored: String,

    /// The current hash of the file. Assumed to be BLAKE3
    pub hash: String
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Change {
    Create(PathBuf),
    Remove(PathBuf),
    Modify(PathBuf),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Revision {
    /// The UUID for this revision. Also used as folder name oftentimes
    /// UUID is serialized as a string.
    pub uuid: String,

    /// The message for this commit
    pub msg: String,

    /// The path of this file's base directory.
    /// This is relative to the `.mcpkg/repository/` folder
    pub dir: PathBuf,

    /// The list of changes made from the previous commit
    pub changes: Vec<Change>,

    /// The time of creation, in UNIX time
    pub created_at: isize,
}
