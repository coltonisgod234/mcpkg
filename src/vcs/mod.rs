use std::{fs, io, path::{Path, PathBuf}};

use log::{debug, info, trace};
use uuid::Uuid;

use crate::vcs::structures::{Change, Repository, Revision};

pub struct CommitInfo {
    pub message: String,

    pub commit_world: bool,
    pub commit_configs: bool,
}

impl CommitInfo {
    pub fn new(message: String, commit_world: bool, commit_configs: bool) -> Self {
        return Self { message, commit_configs, commit_world }
    }
}

pub const MCPKG_REPO_FOLDER: &str = ".mcpkg/repository";
pub const MCPKG_REPO_CONFIG: &str = ".mcpkg/repository.yml";
pub const MCPKG_FOLDER: &str = ".mcpkg";

impl Repository {
    /// Commits non-atomically. Causes disk writes.
    pub fn commit_nonatomic(&mut self, desired: CommitInfo) {
        let r = self.create_rev_from_commitinfo(desired);
        debug!("created revision from commit info: {:?}", r);

        // create the folder
        let mut target_commmit_dir = PathBuf::new();
        target_commmit_dir.push(MCPKG_REPO_FOLDER);
        target_commmit_dir.push(&r.dir);

        debug!("creating revision folder: {:?}", &target_commmit_dir);
        fs::create_dir_all(&target_commmit_dir)
            .expect("failed to create revision folder");

        // copy contents over
        for change in &r.changes {
            info!("\t{:?}", change);

            let mut target = target_commmit_dir.clone();

            // ugliest way to do this
            let relpath = match change {
                Change::Create(p) => p,
                Change::Modify(p) => p,
                Change::Remove(p) => p
            };
            target.push(relpath);

            fs::copy(relpath, target)
                .expect("failed to back up file");
        }

        // add this revision
        self.revisions.push(r);
    }

    fn create_rev_from_commitinfo(&mut self, desired: CommitInfo) -> Revision {
        let commit_uuid = Uuid::now_v7();
        let uuid = commit_uuid.to_string();
        
        let changes = self.get_uncommited_changes(self.get_ignores());

        let r = Revision {
            uuid: uuid.clone(),
            msg: desired.message,
            dir: PathBuf::from(uuid.clone()),
            changes,
            created_at: -1  // temp
        };

        return r
    }

    pub fn get_ignores(&self) -> Vec<&Path> {
        // no memory allocation takes place as far as I know..
        let path_refs: Vec<&Path> = self.ignored_files.iter().map(|p| p.as_path()).collect();
        return path_refs;
    }

    pub fn last_replication(&self) -> Option<&Revision> {
        return self.revisions.last()
    }

    pub fn get_changes_between(&self, now: &Revision, then: &Revision, ignored_files: Vec<&Path>) -> Vec<Change> {
        return diffing::dir(&now.dir, &then.dir, &ignored_files)
    }

    /// Returns the list of uncommited changes
    pub fn get_uncommited_changes_at(&self, path: &PathBuf, against: &Revision, ignored_files: Vec<&Path>) -> Vec<Change> {
        return diffing::dir(&against.dir, &path, &ignored_files);
    }

    pub fn get_uncommited_changes(&self, ignored_files: Vec<&Path>) -> Vec<Change> {
        let here = &PathBuf::from(".");

        // if there's a last replication available. Diff against that
        if let Some(last_commit) = self.last_replication() {
            trace!("last commit available ({:?}), using that", &last_commit);
            return self.get_uncommited_changes_at(
                &here,
                last_commit,
                ignored_files
            );
        } else {
            // otherwise, just say everything is added
            trace!("no last commit is available. This must be the initial commit");
            return diffing::say_everything_was_added(&here, &ignored_files);
        }
    }

    pub fn save_to<P>(&self, path: P)
    where
        P: AsRef<Path>
    {
        let out_file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .expect("failed to open output file");

        serde_yaml::to_writer(&out_file, &self)
            .expect("failed to serialize yaml");
    }
}

pub mod structures;
pub mod backup;
pub mod diffing;

/// Initializes a VCS repository in the current directory
pub fn init_vcs() -> io::Result<()> {
    fs::create_dir(MCPKG_FOLDER)?;
    fs::create_dir(MCPKG_REPO_FOLDER)?;

    let repo = Repository::default();
    repo.save_to(MCPKG_REPO_CONFIG);

    return Ok(());
}

/// Locks the VCS and acquires its config
pub fn repo() -> Repository {
    let yaml = ::std::fs::read_to_string(MCPKG_REPO_CONFIG)
        .expect("failed to open repository config file!");

    return serde_yaml::from_str(&yaml)
        .expect("failed to parse repository config file");
}