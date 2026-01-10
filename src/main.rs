use std::io::{Write, stdin, stdout};

use clap::{Parser, Subcommand};
use log::{error, info};

use crate::{config::ServerKind, server_reload::rcon_connect, vcs::{CommitInfo, structures::Change}};

pub mod config;
pub mod server_setup;
pub mod server_run;
pub mod server_reload;
pub mod vcs;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command
}

#[derive(Subcommand)]
enum Command {
    // server management
    Sync,
    RunOnce,
    RunForever,
    Teardown,
    Stop,
    Repl,

    // VCS
    Init,
    Commit {
        /// Commit message
        message: String,

        /// If the world folder should be commited
        #[arg(short = 'W', long)]
        world: bool,

        /// If the configs should be commited
        #[arg(short = 'C', long)]
        config: bool,

        #[arg(long = "allow-null-commit")]
        allow_null_commit: bool
    },
    GetUnreplicatedChanges,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    env_logger::init();

    let args = Cli::parse();
    let cfg = config::parse_file("config.yaml");

    match args.command {
        Command::Sync => {
            server_setup::sync_multithread(&cfg).await;
            info!("synchronization succeeded.");
        },
        Command::RunOnce => {
            server_run::run_server(&cfg).await;
        },
        Command::Stop => {
            server_reload::reload_rcon(&cfg).await  
                .expect("failed to reload via rcon");
        },
        Command::RunForever => {
            info!("running server forever, press CTRL-D to stop");
            loop {
                server_run::run_server(&cfg).await;
                info!("restarting server (it died??)")
            }
        }
        Command::Teardown => {
            match cfg.server.kind {
                ServerKind::Fabric { .. } => {
                    server_setup::fabric::teardown().await;
                },
                ServerKind::Vanilla => unimplemented!()
            }
        },
        Command::Init => {
            vcs::init_vcs()
                .expect("failed to initialize VCS");
        },
        Command::Commit { message, world, config, allow_null_commit } => {
            // make sure the user doesn't do anything dumb!
            if !world && !config && !allow_null_commit {
                error!("You are about to make a null commit. This commit will have no tracked files and does nothing but clutter up the repository");
                error!("However, null commits can be used to resolve very specific repository errors. If you actually want to make a null commit, specify --allow-null-commit");
                panic!("refusing to make null commit!")
            }

            let desired = CommitInfo::new(message, world, config);

            // try to diff the dirs
            let mut repo = vcs::repo();
            repo.commit_nonatomic(desired);
            repo.save_to(vcs::MCPKG_REPO_CONFIG);

            // info!("{:?}", diff);
        },
        Command::GetUnreplicatedChanges => {
            let repo = vcs::repo();
            let changes = repo.uncommited_changes();
            for change in changes {
                match change {
                    Change::Create(f) => println!("create\t{:?}", f),
                    Change::Modify(f) => println!("modify\t{:?}", f),
                    Change::Remove(f) => println!("remove\t{:?}", f),
                }
            }
        },
        Command::Repl => {
            let mut conn = rcon_connect(&cfg).await
                .expect("failed to connect to rcon");

            let stdin = stdin();
            let mut stdout = stdout();

            loop {
                let mut b = String::new();
                print!(">>> ");
                stdout.flush()
                    .expect("flush failed");

                stdin.read_line(&mut b)
                    .expect("failed to read line");

                let reply = conn.cmd(b.trim()).await
                    .expect("failed to send cmd");

                println!("<<< {}", reply);
            }
        }
    }
}
