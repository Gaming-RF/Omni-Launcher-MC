// Process manager: tracks running game instances, captures stdout/stderr,
// and streams log lines to the frontend via Tauri events.
//
// Event names:
//   "game-log"      — { instance_id, line, stream: "stdout"|"stderr" }
//   "game-exit"     — { instance_id, exit_code }
//   "game-started"  — { instance_id, pid }

use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

#[derive(Debug, Clone, Serialize)]
pub struct GameLogEvent {
    pub instance_id: String,
    pub line: String,
    pub stream: String, // "stdout" or "stderr"
}

#[derive(Debug, Clone, Serialize)]
pub struct GameExitEvent {
    pub instance_id: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameStartedEvent {
    pub instance_id: String,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunningInstance {
    pub instance_id: String,
    pub pid: u32,
    pub started_at: String,
}

/// Manages running game processes. Stored in AppState.
pub struct ProcessManager {
    /// Running children keyed by instance_id.
    processes: Arc<Mutex<HashMap<String, Child>>>,
    /// Log ring buffer per instance (last N lines).
    logs: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

const MAX_LOG_LINES: usize = 5000;

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            logs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn a managed child process. Captures stdout/stderr and streams to frontend.
    pub fn spawn(
        &self,
        app: &tauri::AppHandle,
        instance_id: &str,
        mut child: Child,
        pid: u32,
    ) {
        let iid = instance_id.to_string();

        // Take stdout and stderr
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Store the child
        {
            let mut procs = self.processes.lock().unwrap();
            procs.insert(iid.clone(), child);
        }

        // Init log buffer
        {
            let mut logs = self.logs.lock().unwrap();
            logs.insert(iid.clone(), Vec::new());
        }

        // Emit game-started event
        let _ = app.emit(
            "game-started",
            GameStartedEvent {
                instance_id: iid.clone(),
                pid,
            },
        );

        // Spawn stdout reader
        let app_stdout = app.clone();
        let iid_stdout = iid.clone();
        let logs_stdout = self.logs.clone();
        tokio::spawn(async move {
            if let Some(stdout) = stdout {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    // Store in ring buffer
                    {
                        let mut logs = logs_stdout.lock().unwrap();
                        if let Some(buf) = logs.get_mut(&iid_stdout) {
                            buf.push(line.clone());
                            if buf.len() > MAX_LOG_LINES {
                                buf.remove(0);
                            }
                        }
                    }
                    let _ = app_stdout.emit(
                        "game-log",
                        GameLogEvent {
                            instance_id: iid_stdout.clone(),
                            line,
                            stream: "stdout".to_string(),
                        },
                    );
                }
            }
        });

        // Spawn stderr reader
        let app_stderr = app.clone();
        let iid_stderr = iid.clone();
        let logs_stderr = self.logs.clone();
        tokio::spawn(async move {
            if let Some(stderr) = stderr {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    {
                        let mut logs = logs_stderr.lock().unwrap();
                        if let Some(buf) = logs.get_mut(&iid_stderr) {
                            buf.push(line.clone());
                            if buf.len() > MAX_LOG_LINES {
                                buf.remove(0);
                            }
                        }
                    }
                    let _ = app_stderr.emit(
                        "game-log",
                        GameLogEvent {
                            instance_id: iid_stderr.clone(),
                            line,
                            stream: "stderr".to_string(),
                        },
                    );
                }
            }
        });

        // Spawn exit watcher
        let app_exit = app.clone();
        let iid_exit = iid.clone();
        let procs_exit = self.processes.clone();
        tokio::spawn(async move {
            // We need to wait on the child, but we already stored it.
            // Poll the process map periodically.
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let mut procs = procs_exit.lock().unwrap();
                if let Some(child) = procs.get_mut(&iid_exit) {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            procs.remove(&iid_exit);
                            let _ = app_exit.emit(
                                "game-exit",
                                GameExitEvent {
                                    instance_id: iid_exit.clone(),
                                    exit_code: status.code(),
                                },
                            );
                            break;
                        }
                        Ok(None) => continue,
                        Err(_) => {
                            procs.remove(&iid_exit);
                            let _ = app_exit.emit(
                                "game-exit",
                                GameExitEvent {
                                    instance_id: iid_exit.clone(),
                                    exit_code: None,
                                },
                            );
                            break;
                        }
                    }
                } else {
                    // Process was removed (killed externally)
                    break;
                }
            }
        });
    }

    /// Check if an instance is currently running.
    pub fn is_running(&self, instance_id: &str) -> bool {
        let procs = self.processes.lock().unwrap();
        procs.contains_key(instance_id)
    }

    /// Get list of running instances.
    pub fn running_instances(&self) -> Vec<String> {
        let procs = self.processes.lock().unwrap();
        procs.keys().cloned().collect()
    }

    /// Kill a running instance.
    pub fn kill(&self, instance_id: &str) -> Result<(), String> {
        let mut procs = self.processes.lock().unwrap();
        if let Some(mut child) = procs.remove(instance_id) {
            child.start_kill().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Instance is not running".to_string())
        }
    }

    /// Get stored log lines for an instance.
    pub fn get_logs(&self, instance_id: &str) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        logs.get(instance_id).cloned().unwrap_or_default()
    }
}
