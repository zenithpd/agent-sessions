use super::model::SessionStatus;

/// Check if message content contains a tool_use block
pub fn has_tool_use(content: &serde_json::Value) -> bool {
    if let serde_json::Value::Array(arr) = content {
        arr.iter().any(|item| {
            item.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "tool_use")
                .unwrap_or(false)
        })
    } else {
        false
    }
}

/// Check if message content contains a tool_result block
pub fn has_tool_result(content: &serde_json::Value) -> bool {
    if let serde_json::Value::Array(arr) = content {
        arr.iter().any(|item| {
            item.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "tool_result")
                .unwrap_or(false)
        })
    } else {
        false
    }
}

/// Check if message content is a local slash command that doesn't trigger Claude response
/// These commands are handled locally by Claude Code and don't require thinking
pub fn is_local_slash_command(content: &serde_json::Value) -> bool {
    let text = match content {
        serde_json::Value::String(s) => s.as_str(),
        serde_json::Value::Array(arr) => {
            // Find first text block
            arr.iter().find_map(|v| {
                v.get("text").and_then(|t| t.as_str())
            }).unwrap_or("")
        }
        _ => return false,
    };

    let trimmed = text.trim();

    // Local commands that don't trigger Claude to think
    // These are handled by the CLI itself
    let local_commands = [
        "/clear",
        "/compact",
        "/help",
        "/config",
        "/cost",
        "/doctor",
        "/init",
        "/login",
        "/logout",
        "/memory",
        "/model",
        "/permissions",
        "/pr-comments",
        "/review",
        "/status",
        "/terminal-setup",
        "/vim",
    ];

    local_commands.iter().any(|cmd| {
        trimmed == *cmd || trimmed.starts_with(&format!("{} ", cmd))
    })
}

/// Returns sort priority for status (lower = higher priority in list)
/// Active sessions (thinking/processing) appear first, then waiting, then idle
pub fn status_sort_priority(status: &SessionStatus) -> u8 {
    match status {
        SessionStatus::Thinking => 0,   // Active - Claude is working - show first
        SessionStatus::Processing => 0, // Active - tool is running - show first
        SessionStatus::Waiting => 1,    // Needs attention - show second
        SessionStatus::Idle => 2,       // Inactive - show last
    }
}

/// Determine session status based on the last message in the conversation
///
/// Status determination logic:
/// - If file is being actively modified (within last 3s) -> active state (Thinking or Processing)
/// - If last message is user with tool_result -> Processing (tool just ran, Claude processing result)
/// - If last message is from assistant with tool_use -> Processing (tool is being executed)
/// - If last message is from assistant with only text -> Waiting (Claude finished, waiting for user)
/// - If last message is from user -> Thinking (Claude is generating a response)
/// - If last message is a local slash command (/clear, /help, etc.) -> Waiting (these don't trigger Claude)
pub fn determine_status(
    last_msg_type: Option<&str>,
    has_tool_use: bool,
    has_tool_result: bool,
    is_local_command: bool,
    file_recently_modified: bool,
) -> SessionStatus {
    // Key insight: Once an assistant text message (without tool_use) is written, Claude is done
    // and waiting for user input, regardless of file modification time

    match last_msg_type {
        Some("assistant") => {
            if has_tool_use {
                // Assistant sent a tool_use, tool is executing
                SessionStatus::Processing
            } else {
                // Assistant sent a text response - waiting for user input
                // Once Claude sends text without tool_use, it's done and waiting
                SessionStatus::Waiting
            }
        }
        Some("user") => {
            if is_local_command {
                // Local slash commands like /clear, /help, /compact don't trigger Claude
                // Session is waiting for actual user input
                SessionStatus::Waiting
            } else if has_tool_result {
                // User message contains tool_result - tool execution complete,
                // Claude is processing the result
                if file_recently_modified {
                    SessionStatus::Thinking
                } else {
                    // Tool result was sent but file not recently modified
                    // This can happen if tool result was recent - show Processing
                    SessionStatus::Processing
                }
            } else {
                // Regular user input, Claude is thinking/generating response
                SessionStatus::Thinking
            }
        }
        _ => {
            // No recognized message type - check if file is active
            if file_recently_modified {
                SessionStatus::Thinking
            } else {
                SessionStatus::Idle
            }
        }
    }
}
