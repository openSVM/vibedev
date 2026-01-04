// LLM Daemon - Keeps model loaded in memory for fast queries
use crate::embedded_llm::{DeviceType, EmbeddedLlm, Quantization};
use anyhow::{anyhow, Result};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

const SOCKET_NAME: &str = "vibedev.sock";

/// Get the socket path
pub fn get_socket_path() -> PathBuf {
    dirs::runtime_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(SOCKET_NAME)
}

/// Get the PID file path
pub fn get_pid_path() -> PathBuf {
    dirs::runtime_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("vibedev.pid")
}

/// Check if daemon is running
pub fn is_running() -> bool {
    let socket_path = get_socket_path();
    if !socket_path.exists() {
        return false;
    }

    // Try to connect
    UnixStream::connect(&socket_path).is_ok()
}

/// Send a query to the daemon
pub fn query(prompt: &str, context: Option<&str>) -> Result<String> {
    let socket_path = get_socket_path();
    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|_| anyhow!("Daemon not running. Start with: vibedev daemon start"))?;

    // Send request as JSON
    let request = serde_json::json!({
        "type": "query",
        "prompt": prompt,
        "context": context
    });

    writeln!(stream, "{}", request)?;
    stream.flush()?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;

    let resp: serde_json::Value = serde_json::from_str(&response)?;

    if let Some(error) = resp.get("error") {
        Err(anyhow!("{}", error.as_str().unwrap_or("Unknown error")))
    } else if let Some(result) = resp.get("result") {
        Ok(result.as_str().unwrap_or("").to_string())
    } else {
        Err(anyhow!("Invalid response from daemon"))
    }
}

/// Get daemon status
pub fn status() -> Result<String> {
    let socket_path = get_socket_path();
    let mut stream =
        UnixStream::connect(&socket_path).map_err(|_| anyhow!("Daemon not running"))?;

    let request = serde_json::json!({ "type": "status" });
    writeln!(stream, "{}", request)?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;

    Ok(response)
}

/// Stop the daemon
pub fn stop() -> Result<()> {
    let socket_path = get_socket_path();

    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        let request = serde_json::json!({ "type": "shutdown" });
        let _ = writeln!(stream, "{}", request);
        let _ = stream.flush();
    }

    // Clean up socket
    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    // Clean up PID file
    let pid_path = get_pid_path();
    if pid_path.exists() {
        fs::remove_file(&pid_path)?;
    }

    Ok(())
}

/// Start the daemon (blocking)
pub fn start(
    model_id: Option<&str>,
    device_type: Option<DeviceType>,
    quantization: Option<Quantization>,
) -> Result<()> {
    let socket_path = get_socket_path();

    // Clean up old socket if exists
    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    // Load the model
    println!("Starting vibedev daemon...");
    let llm = EmbeddedLlm::new_with_options(model_id, device_type, quantization)?;
    let llm = Arc::new(Mutex::new(llm));

    // Create Unix socket listener
    let listener = UnixListener::bind(&socket_path)?;
    println!("Daemon listening on: {}", socket_path.display());
    println!("Model loaded and ready for queries.");
    println!("\nUse 'vibedev daemon stop' to stop the daemon.");

    // Save PID
    let pid_path = get_pid_path();
    fs::write(&pid_path, std::process::id().to_string())?;

    // Handle connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let llm = Arc::clone(&llm);
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, llm) {
                        eprintln!("Client error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(stream: UnixStream, llm: Arc<Mutex<EmbeddedLlm>>) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    let mut line = String::new();
    reader.read_line(&mut line)?;

    let request: serde_json::Value = serde_json::from_str(&line)?;
    let req_type = request.get("type").and_then(|t| t.as_str()).unwrap_or("");

    let response = match req_type {
        "query" => {
            let prompt = request.get("prompt").and_then(|p| p.as_str()).unwrap_or("");
            let context = request.get("context").and_then(|c| c.as_str());

            let mut llm = llm.lock().map_err(|_| anyhow!("Lock error"))?;

            if let Some(ctx) = context {
                llm.set_context(ctx);
            }

            match llm.generate(prompt) {
                Ok(result) => serde_json::json!({ "result": result }),
                Err(e) => serde_json::json!({ "error": e.to_string() }),
            }
        }

        "status" => {
            let llm = llm.lock().map_err(|_| anyhow!("Lock error"))?;
            serde_json::json!({
                "status": "running",
                "model": llm.model_name(),
                "pid": std::process::id()
            })
        }

        "shutdown" => {
            println!("Shutdown requested, exiting...");
            std::process::exit(0);
        }

        _ => {
            serde_json::json!({ "error": "Unknown request type" })
        }
    };

    writeln!(writer, "{}", response)?;
    writer.flush()?;

    Ok(())
}

/// Daemon info for display
pub struct DaemonInfo {
    pub running: bool,
    pub model: Option<String>,
    pub pid: Option<u32>,
    pub socket: PathBuf,
}

pub fn info() -> DaemonInfo {
    let socket = get_socket_path();

    if !is_running() {
        return DaemonInfo {
            running: false,
            model: None,
            pid: None,
            socket,
        };
    }

    // Get status from daemon
    if let Ok(status_str) = status() {
        if let Ok(status) = serde_json::from_str::<serde_json::Value>(&status_str) {
            return DaemonInfo {
                running: true,
                model: status
                    .get("model")
                    .and_then(|m| m.as_str())
                    .map(String::from),
                pid: status.get("pid").and_then(|p| p.as_u64()).map(|p| p as u32),
                socket,
            };
        }
    }

    DaemonInfo {
        running: true,
        model: None,
        pid: None,
        socket,
    }
}
