use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeProcess {
    pub pid: u32,
    pub cwd: Option<PathBuf>,
    pub cpu_usage: f32,
    pub memory: u64,
}

pub fn find_claude_processes() -> Vec<ClaudeProcess> {
    // Refresh process info - use Always to detect newly spawned processes
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(
            ProcessRefreshKind::new()
                .with_cmd(sysinfo::UpdateKind::Always)
                .with_cwd(sysinfo::UpdateKind::Always)
                .with_cpu()
                .with_memory()
        )
    );
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let mut processes = Vec::new();

    for (pid, process) in system.processes() {
        // Claude Code runs as a node process with "claude" as the first command argument
        let cmd = process.cmd();

        let is_claude = if let Some(first_arg) = cmd.first() {
            let first_arg_str = first_arg.to_string_lossy().to_lowercase();
            first_arg_str == "claude" || first_arg_str.ends_with("/claude")
        } else {
            false
        };

        // Exclude our own app
        let is_our_app = process.name().to_string_lossy().contains("claude-sessions")
            || process.name().to_string_lossy().contains("tauri-temp");

        if is_claude && !is_our_app {
            let cwd = process.cwd().map(|p| p.to_path_buf());

            processes.push(ClaudeProcess {
                pid: pid.as_u32(),
                cwd,
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
            });
        }
    }

    processes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_process_creation() {
        let process = ClaudeProcess {
            pid: 12345,
            cwd: Some(PathBuf::from("/Users/test/Projects/my-project")),
            cpu_usage: 5.5,
            memory: 1024 * 1024 * 100, // 100MB
        };

        assert_eq!(process.pid, 12345);
        assert_eq!(process.cwd, Some(PathBuf::from("/Users/test/Projects/my-project")));
        assert_eq!(process.cpu_usage, 5.5);
        assert_eq!(process.memory, 104857600);
    }

    #[test]
    fn test_claude_process_without_cwd() {
        let process = ClaudeProcess {
            pid: 99999,
            cwd: None,
            cpu_usage: 0.0,
            memory: 0,
        };

        assert_eq!(process.pid, 99999);
        assert!(process.cwd.is_none());
    }

    #[test]
    fn test_claude_process_serialization() {
        let process = ClaudeProcess {
            pid: 12345,
            cwd: Some(PathBuf::from("/Users/test/Projects/my-project")),
            cpu_usage: 5.5,
            memory: 104857600,
        };

        let serialized = serde_json::to_string(&process).unwrap();
        assert!(serialized.contains("\"pid\":12345"));
        assert!(serialized.contains("\"cpu_usage\":5.5"));
        assert!(serialized.contains("\"memory\":104857600"));

        // Test round-trip
        let deserialized: ClaudeProcess = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.pid, process.pid);
        assert_eq!(deserialized.cpu_usage, process.cpu_usage);
        assert_eq!(deserialized.memory, process.memory);
    }

    #[test]
    fn test_claude_process_clone() {
        let process = ClaudeProcess {
            pid: 12345,
            cwd: Some(PathBuf::from("/test/path")),
            cpu_usage: 10.0,
            memory: 2048,
        };

        let cloned = process.clone();
        assert_eq!(process, cloned);
    }

    #[test]
    fn test_find_claude_processes_returns_vec() {
        // This test verifies the function runs without panicking
        // and returns a vector (may be empty if no Claude processes are running)
        let processes = find_claude_processes();

        // If there are any processes, verify they have valid PIDs
        for process in &processes {
            assert!(process.pid > 0);
        }
    }
}
