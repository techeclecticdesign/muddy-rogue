use tauri::{AppHandle, Emitter};

#[tauri::command]
async fn send_command(app: AppHandle, command: String) -> Result<(), String> {
    // Echo the command back
    app.emit("stream-message", format!("> {}", command))
        .map_err(|e| e.to_string())?;

    // Simulate processing and response
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let response = process_command(&command);
        let _ = app.emit("stream-message", response);
    });

    Ok(())
}

fn process_command(command: &str) -> String {
    // Simple command processing - extend this for your MUD logic
    match command.trim().to_lowercase().as_str() {
        "help" => "Available commands: help, time, hello, echo <text>".to_string(),
        "time" => format!("Current time: {}", chrono::Local::now().format("%H:%M:%S")),
        "hello" => "Hello, adventurer! Welcome to the MUD.".to_string(),
        cmd if cmd.starts_with("echo ") => cmd.strip_prefix("echo ").unwrap_or("").to_string(),
        _ => format!(
            "Unknown command: {}. Type 'help' for available commands.",
            command
        ),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![send_command])
        .setup(|app| {
            // Send welcome message on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let _ = app_handle.emit("stream-message", "=== MUD Client v1.0 ===");
                let _ = app_handle.emit("stream-message", "Type 'help' for available commands.");
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
