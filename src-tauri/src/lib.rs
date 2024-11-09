use std::sync::Arc;

use anyhow::Result;
use tauri::State;
use tokio::sync::Mutex;
use crate::models::Session;

pub mod models;

type SharedSession = Arc<Mutex<Option<Session>>>;

#[tauri::command]
async fn connect_ssh(
    username: String,
    host: String,
    port: u16,
    state: State<'_, SharedSession>,
) -> Result<String, String> {
    let session = Session::connect(username, (host.as_str(), port))
        .await
        .map_err(|e| e.to_string())?;
    *state.lock().await = Some(session); // Update the shared state
    Ok("Connected".to_string())
}

#[tauri::command]
async fn execute_command(
    command: String,
    state: State<'_, SharedSession>,
) -> Result<String, String> {
    let mut session_guard = state.lock().await;
    let session = session_guard
        .as_mut()
        .ok_or_else(|| "SSH session not connected".to_string())?;

    let output = session
        .send_command(&command)
        .await
        .map_err(|e| e.to_string())?;
    Ok(output)
}

#[tauri::command]
async fn get_cpu_usage(state: State<'_, SharedSession>) -> Result<String, String> {
    let mut session_guard = state.lock().await;
    let session = session_guard
        .as_mut()
        .ok_or_else(|| "SSH session not connected".to_string())?;

    let output = session
        .send_command("top -bn1 | grep 'Cpu(s)' | awk '{print $2 + $4}'")
        .await
        .map_err(|e| e.to_string())?;
    Ok(output)
}

#[tauri::command]
async fn get_memory_usage(state: State<'_, SharedSession>) -> Result<String, String> {
    let mut session_guard = state.lock().await;
    let session = session_guard
        .as_mut()
        .ok_or_else(|| "SSH session not connected".to_string())?;

    let output = session
        .send_command("free -m | awk 'NR==2{printf \"%.2f\", $3*100/$2 }'")
        .await
        .map_err(|e| e.to_string())?;
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
        .invoke_handler(tauri::generate_handler![
            connect_ssh,
            execute_command,
            get_cpu_usage,
            get_memory_usage,
            disconnect_ssh
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
