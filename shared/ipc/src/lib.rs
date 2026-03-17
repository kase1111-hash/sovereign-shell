//! Sovereign Shell — Inter-Module IPC Library
//!
//! Provides a simple named pipe server and client for communication between
//! Sovereign Shell modules. Protocol: newline-delimited JSON over Windows Named Pipes.
//!
//! Pipe names follow the pattern: `\\.\pipe\sovereign-shell-<module>`

use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Standard IPC message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message type for routing (e.g., "search", "status", "notify")
    #[serde(rename = "type")]
    pub msg_type: String,
    /// JSON payload — interpretation depends on msg_type
    pub payload: serde_json::Value,
}

impl Message {
    /// Create a new message with a typed payload.
    pub fn new<T: Serialize>(msg_type: &str, payload: &T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            msg_type: msg_type.to_string(),
            payload: serde_json::to_value(payload)?,
        })
    }

    /// Deserialize the payload into a concrete type.
    pub fn parse_payload<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }

    /// Serialize this message to a newline-delimited JSON string.
    pub fn to_line(&self) -> Result<String, serde_json::Error> {
        let mut s = serde_json::to_string(self)?;
        s.push('\n');
        Ok(s)
    }

    /// Parse a message from a JSON line.
    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }

    /// Convenience: create an error response.
    pub fn error(message: &str) -> Self {
        Self {
            msg_type: "error".to_string(),
            payload: serde_json::json!({ "message": message }),
        }
    }
}

/// Returns the full pipe name for a module.
/// Format: `\\.\pipe\sovereign-shell-<module_name>`
pub fn pipe_name(module_name: &str) -> String {
    format!(r"\\.\pipe\sovereign-shell-{}", module_name)
}

// ── Windows Implementation ──────────────────────────────────────────────────

#[cfg(windows)]
pub mod server {
    //! Named pipe server for daemon modules.

    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows::Win32::Storage::FileSystem::{
        FlushFileBuffers, ReadFile, WriteFile,
    };
    use windows::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe,
        PIPE_ACCESS_DUPLEX, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_WAIT,
    };

    /// A handler function that receives a request Message and returns a response Message.
    pub type Handler = Box<dyn Fn(Message) -> Message + Send + Sync>;

    /// Configuration for the IPC server.
    pub struct ServerConfig {
        pub pipe_name: String,
        pub buffer_size: u32,
    }

    impl ServerConfig {
        pub fn new(module_name: &str) -> Self {
            Self {
                pipe_name: super::pipe_name(module_name),
                buffer_size: 65536,
            }
        }
    }

    /// Run a blocking named pipe server. Accepts one client at a time.
    /// Calls `handler` for each incoming message and writes the response back.
    ///
    /// This function loops forever. Run it in a dedicated thread.
    ///
    /// # Safety
    /// Uses Win32 named pipe APIs.
    pub fn run(config: &ServerConfig, handler: &Handler) -> std::io::Result<()> {
        loop {
            // Create the named pipe instance
            let pipe_name = HSTRING::from(&config.pipe_name);
            let pipe = unsafe {
                CreateNamedPipeW(
                    &pipe_name,
                    PIPE_ACCESS_DUPLEX,
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                    1,                      // max instances
                    config.buffer_size,     // out buffer
                    config.buffer_size,     // in buffer
                    0,                      // default timeout
                    None,                   // default security
                )
            };

            if pipe == INVALID_HANDLE_VALUE {
                return Err(std::io::Error::last_os_error());
            }

            // Wait for a client to connect
            let connected = unsafe { ConnectNamedPipe(pipe, None) };
            if connected.is_err() {
                // ERROR_PIPE_CONNECTED means client connected between Create and Connect — that's fine
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() != Some(535) {
                    unsafe { CloseHandle(pipe).ok() };
                    continue;
                }
            }

            // Handle the client session
            if let Err(e) = handle_client(pipe, handler) {
                eprintln!("[sovereign-ipc] Client error: {e}");
            }

            // Disconnect and close for next client
            unsafe {
                DisconnectNamedPipe(pipe).ok();
                CloseHandle(pipe).ok();
            }
        }
    }

    fn handle_client(pipe: HANDLE, handler: &Handler) -> std::io::Result<()> {
        // Read all available data into a buffer
        let mut raw_buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let mut bytes_read = 0u32;
            let ok = unsafe {
                ReadFile(
                    pipe,
                    Some(&mut chunk),
                    Some(&mut bytes_read),
                    None,
                )
            };

            if bytes_read > 0 {
                raw_buf.extend_from_slice(&chunk[..bytes_read as usize]);
            }

            // Check if we have a complete line
            if raw_buf.contains(&b'\n') {
                break;
            }

            if ok.is_err() || bytes_read == 0 {
                break;
            }
        }

        // Process each line (typically just one per connection)
        let text = String::from_utf8_lossy(&raw_buf);
        for line in text.lines() {
            if line.trim().is_empty() { continue; }

            let response = match Message::from_line(line) {
                Ok(msg) => handler(msg),
                Err(e) => Message::error(&format!("Parse error: {e}")),
            };

            let resp_bytes = response.to_line()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let buf = resp_bytes.as_bytes();
            let mut written = 0u32;
            unsafe {
                WriteFile(pipe, Some(buf), Some(&mut written), None).ok();
                FlushFileBuffers(pipe).ok();
            }
        }

        Ok(())
    }
}

#[cfg(windows)]
pub mod client {
    //! Named pipe client for querying daemon modules.

    use super::*;
    use std::fs::OpenOptions;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::time::Duration;

    /// Send a message to a named pipe server and return the response.
    ///
    /// Opens a connection, sends the message as newline-delimited JSON,
    /// reads the response, and closes the connection.
    pub fn query(module_name: &str, message: &Message) -> std::io::Result<Message> {
        let name = super::pipe_name(module_name);

        // Open the named pipe as a file
        let mut pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&name)?;

        // Send the request
        let line = message.to_line()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        pipe.write_all(line.as_bytes())?;
        pipe.flush()?;

        // Read the response
        let mut response = String::new();
        let mut reader = BufReader::new(&mut pipe);
        reader.read_line(&mut response)?;

        Message::from_line(&response)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Check if a module's IPC server is running by attempting a connection.
    pub fn is_available(module_name: &str) -> bool {
        let name = super::pipe_name(module_name);
        OpenOptions::new().read(true).write(true).open(&name).is_ok()
    }
}

// ── Stub for non-Windows (allows compilation on Linux for CI/dev) ───────────

#[cfg(not(windows))]
pub mod server {
    use super::*;

    pub type Handler = Box<dyn Fn(Message) -> Message + Send + Sync>;

    pub struct ServerConfig {
        pub pipe_name: String,
        pub buffer_size: u32,
    }

    impl ServerConfig {
        pub fn new(module_name: &str) -> Self {
            Self {
                pipe_name: super::pipe_name(module_name),
                buffer_size: 65536,
            }
        }
    }

    pub fn run(_config: &ServerConfig, _handler: &Handler) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Named pipe server requires Windows",
        ))
    }
}

#[cfg(not(windows))]
pub mod client {
    use super::*;

    pub fn query(_module_name: &str, _message: &Message) -> std::io::Result<Message> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Named pipe client requires Windows",
        ))
    }

    pub fn is_available(_module_name: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_roundtrip() {
        let msg = Message::new("search", &serde_json::json!({"query": "test"})).unwrap();
        let line = msg.to_line().unwrap();
        let parsed = Message::from_line(&line).unwrap();
        assert_eq!(parsed.msg_type, "search");
        assert_eq!(parsed.payload["query"], "test");
    }

    #[test]
    fn test_pipe_name() {
        assert_eq!(
            pipe_name("search-daemon"),
            r"\\.\pipe\sovereign-shell-search-daemon"
        );
    }

    #[test]
    fn test_error_message() {
        let err = Message::error("something broke");
        assert_eq!(err.msg_type, "error");
        assert_eq!(err.payload["message"], "something broke");
    }

    #[test]
    fn test_parse_payload() {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct Query { query: String, max_results: usize }

        let msg = Message::new("search", &serde_json::json!({
            "query": "budget.xlsx",
            "max_results": 10
        })).unwrap();

        let q: Query = msg.parse_payload().unwrap();
        assert_eq!(q.query, "budget.xlsx");
        assert_eq!(q.max_results, 10);
    }
}
