//! Named pipe IPC server for receiving notifications from other Sovereign Shell modules.
//!
//! Listens on `\\.\pipe\sovereign-shell-notify` (Windows) or
//! `/tmp/sovereign-shell-notify.sock` (Unix) for JSON messages.

use crate::queue::{Notification, Priority};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

/// IPC message format expected from other modules.
#[derive(Debug, Deserialize)]
pub struct IpcMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub payload: Option<NotifyPayload>,
}

#[derive(Debug, Deserialize)]
pub struct NotifyPayload {
    pub title: String,
    pub body: String,
    pub source: String,
    #[serde(default = "default_priority")]
    pub priority: String,
}

fn default_priority() -> String {
    "normal".to_string()
}

/// IPC response.
#[derive(Debug, Serialize)]
pub struct IpcResponse {
    pub ok: bool,
    pub message: String,
}

/// Parse a priority string.
fn parse_priority(s: &str) -> Priority {
    match s.to_lowercase().as_str() {
        "low" => Priority::Low,
        "high" => Priority::High,
        "critical" => Priority::Critical,
        _ => Priority::Normal,
    }
}

/// Process an IPC message line and return a notification if it's a notify message.
pub fn process_message(line: &str) -> Result<Option<Notification>, String> {
    let msg: IpcMessage = serde_json::from_str(line)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    match msg.msg_type.as_str() {
        "notify" => {
            let payload = msg
                .payload
                .ok_or_else(|| "Missing payload for notify message".to_string())?;

            let priority = parse_priority(&payload.priority);
            let notif = Notification::new(&payload.source, &payload.title, &payload.body, priority);
            Ok(Some(notif))
        }
        "ping" => Ok(None),
        other => Err(format!("Unknown message type: {}", other)),
    }
}

/// Start the IPC server in a background thread.
/// Sends received notifications through the provided channel.
pub fn start_ipc_server(tx: mpsc::Sender<Notification>) -> Result<(), String> {
    std::thread::spawn(move || {
        if let Err(e) = run_server(tx) {
            log::error!("IPC server error: {}", e);
        }
    });
    Ok(())
}

#[cfg(windows)]
fn run_server(tx: mpsc::Sender<Notification>) -> Result<(), String> {
    use std::io::{BufRead, BufReader, Write};

    let pipe_name = r"\\.\pipe\sovereign-shell-notify";
    log::info!("IPC server starting on {}", pipe_name);

    loop {
        // Use sovereign_ipc or fallback to raw named pipe
        match sovereign_ipc::server::listen_once(pipe_name) {
            Ok(mut stream) => {
                let reader = BufReader::new(stream.try_clone().unwrap_or_else(|_| {
                    // Fallback: can't clone, just read
                    log::warn!("Could not clone pipe stream");
                    stream
                }));
                for line in reader.lines() {
                    match line {
                        Ok(l) if l.is_empty() => continue,
                        Ok(l) => {
                            let response = match process_message(&l) {
                                Ok(Some(notif)) => {
                                    let _ = tx.send(notif);
                                    IpcResponse {
                                        ok: true,
                                        message: "Notification queued".into(),
                                    }
                                }
                                Ok(None) => IpcResponse {
                                    ok: true,
                                    message: "Pong".into(),
                                },
                                Err(e) => IpcResponse {
                                    ok: false,
                                    message: e,
                                },
                            };
                            let resp_json = serde_json::to_string(&response).unwrap_or_default();
                            let _ = stream.write_all(resp_json.as_bytes());
                            let _ = stream.write_all(b"\n");
                        }
                        Err(e) => {
                            log::debug!("IPC read error: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("IPC listen error: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

#[cfg(not(windows))]
fn run_server(tx: mpsc::Sender<Notification>) -> Result<(), String> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;

    let sock_path = "/tmp/sovereign-shell-notify.sock";

    // Remove stale socket
    let _ = std::fs::remove_file(sock_path);

    let listener =
        UnixListener::bind(sock_path).map_err(|e| format!("Failed to bind socket: {}", e))?;

    log::info!("IPC server listening on {}", sock_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let mut writer = stream
                        .try_clone()
                        .expect("Failed to clone stream");
                    let reader = BufReader::new(stream);
                    for line in reader.lines() {
                        match line {
                            Ok(l) if l.is_empty() => continue,
                            Ok(l) => {
                                let response = match process_message(&l) {
                                    Ok(Some(notif)) => {
                                        let _ = tx.send(notif);
                                        IpcResponse {
                                            ok: true,
                                            message: "Notification queued".into(),
                                        }
                                    }
                                    Ok(None) => IpcResponse {
                                        ok: true,
                                        message: "Pong".into(),
                                    },
                                    Err(e) => IpcResponse {
                                        ok: false,
                                        message: e,
                                    },
                                };
                                let resp_json =
                                    serde_json::to_string(&response).unwrap_or_default();
                                let _ = writer.write_all(resp_json.as_bytes());
                                let _ = writer.write_all(b"\n");
                            }
                            Err(_) => break,
                        }
                    }
                });
            }
            Err(e) => {
                log::error!("IPC accept error: {}", e);
            }
        }
    }

    Ok(())
}
