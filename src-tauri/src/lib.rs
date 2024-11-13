use crate::models::client::{Session, ShellChannel};
use anyhow::Result;
use tauri::State;
use tokio::sync::Mutex;

pub mod models;

type SharedSession = Mutex<Option<Session>>;
type SharedChannel = Mutex<Option<ShellChannel>>;

#[tauri::command]
async fn connect_ssh(
    username: String,
    host: String,
    port: u16,
    shared_session: State<'_, SharedSession>,
    shared_channel: State<'_, SharedChannel>,
) -> Result<String, String> {
    let mut session = Session::connect(username, (host.as_str(), port))
        .await
        .map_err(|e| e.to_string())?;

    let channel = session
        .channel_open_session()
        .await
        .map_err(|e| e.to_string())?;

    *shared_session.lock().await = Some(session);
    *shared_channel.lock().await = Some(ShellChannel::new(channel));

    Ok("Connected".to_string())
}

async fn execute_ssh_command(
    shell_channel: &mut ShellChannel,
    command: &str,
) -> Result<String, String> {
    let output = shell_channel
        .send_command(command)
        .await
        .map_err(|e| e.to_string())?;
    Ok(output)
}

#[tauri::command]
async fn execute_command(
    command: String,
    state: State<'_, SharedChannel>,
) -> Result<String, String> {
    let mut channel_guard = state.lock().await;
    let channel = channel_guard
        .as_mut()
        .ok_or_else(|| "SSH session not connected".to_string())?;

    let output = channel
        .send_command(&command)
        .await
        .map_err(|e| e.to_string())?;
    Ok(output)
}

#[tauri::command]
async fn get_cpu_usage(state: State<'_, SharedChannel>) -> Result<String, String> {
    let mut channel_guard = state.lock().await;
    let channel = channel_guard
        .as_mut()
        .ok_or_else(|| "SSH session not connected".to_string())?;

    execute_ssh_command(channel, "top -bn1 | grep 'Cpu(s)' | awk '{print $2 + $4}'").await
}

#[tauri::command]
async fn get_memory_usage(state: State<'_, SharedChannel>) -> Result<Vec<String>, String> {
    let mut channel_guard = state.lock().await;
    let channel = channel_guard
        .as_mut()
        .ok_or_else(|| "SSH session not connected".to_string())?;

    let output = execute_ssh_command(
        channel,
        "free -g | awk 'NR==2{printf  \"%s,%s,%.0f\", $3,$2,$3*100/$2 }'",
    )
    .await?;

    // Tidy up the output on a tuple
    let output = output
        .split(",")
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    Ok(output)
}

#[tauri::command]
async fn get_disk_usage(state: State<'_, SharedChannel>) -> Result<Vec<String>, String> {
    let mut channel_guard = state.lock().await;
    let channel = channel_guard
        .as_mut()
        .ok_or_else(|| "SSH session not connected".to_string())?;

    let output = execute_ssh_command(
        channel,
        "df -h / --output=size,used,pcent | awk 'NR==2 { printf \"%s,%s,%s\", $2, $1, $3 }' | tr -d 'G%'",
    )
    .await?;

    // Tidy up the output on a tuple
    let output = output
        .split(",")
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    Ok(output)
}

#[tauri::command]
async fn disconnect_ssh(state: State<'_, SharedSession>) -> Result<String, String> {
    let mut session_guard = state.lock().await;
    if let Some(mut session) = session_guard.take() {
        session.close().await.map_err(|e| e.to_string())?;
    }
    Ok("Disconnected".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(SharedSession::default())
        .manage(SharedChannel::default())
        .invoke_handler(tauri::generate_handler![
            connect_ssh,
            execute_command,
            get_cpu_usage,
            get_memory_usage,
            get_disk_usage,
            disconnect_ssh
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
