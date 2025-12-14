use log::info;
use tokio::{process::{Child, Command}, time::sleep};
use crate::{config::Config};

pub fn spawn_server_child(c: &Config) -> Child {
    let max_mem = format!("-Xmx{}", c.server.max_memory);

    return Command::new("java")
        .arg(max_mem)
        .arg("-jar")
        .arg("server.jar")
        .arg("nogui")
        .kill_on_drop(true)
        .spawn()
        .expect("failed to spawn server child");
}

/// Runs the server once. Returns if the child exists (server crash / killed)
/// or if the server should be restarted by this point
pub async fn run_server(c: &Config) {
    info!("starting server...");
    let mut server_child: Child = spawn_server_child(c);
    let restart_duration = c.server.restart_every;

    tokio::select! {
        status = server_child.wait() => {
            info!("server exited: {:?}", status);
            return
        },
        _ = sleep(restart_duration) => {
            info!("server should restart now.")
        }
    };
}