mod command_parser;
mod minimap;
mod player;
mod room;
mod settings;
mod text_utils;
mod zone;

use command_parser::{get_room_display, process_move, HELP_TEXT};
use minimap::{generate_minimap, MinimapNode};
use player::Player;
use settings::Settings;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use zone::{load_rooms, RoomMap, ZoneConfig};

struct SettingsState {
    settings: Mutex<Settings>,
    settings_path: PathBuf,
}

struct Game {
    rooms: RoomMap,
    player: Player,
}

impl Game {
    fn load_from_zones(
        zones_json: &str,
        zone_files: &[(&str, &str)],
    ) -> Result<Self, serde_json::Error> {
        let zone_config: ZoneConfig = serde_json::from_str(zones_json)?;
        let room = load_rooms(&zone_config.zones, zone_files)?;

        Ok(Self {
            rooms: room,
            player: Player::new(zone_config.initial_zone, zone_config.initial_room),
        })
    }

    fn get_current_room_display(&self) -> Vec<String> {
        get_room_display(&self.player, &self.rooms)
    }

    fn process_move(&mut self, command: &str) -> Result<Vec<String>, String> {
        process_move(&mut self.player, &self.rooms, command)
    }
}

struct GameState {
    game: Mutex<Option<Game>>,
}

fn emit_game_message(app: &AppHandle, message: &str) -> Result<(), String> {
    let state = app.state::<SettingsState>();

    let (wrap_enabled, wrap_len) = match state.settings.lock() {
        Ok(s) => (s.word_wrap_enabled, s.word_wrap_length as usize),
        Err(_) => (true, 100),
    };

    let final_message = if wrap_enabled {
        // Wrap the lines, then join them back into a single string with \n
        text_utils::wrap_lines(message, wrap_len).join("\n")
    } else {
        message.to_string()
    };

    app.emit("stream-message", final_message)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_settings(app: AppHandle) -> Result<Settings, String> {
    let state = app.state::<SettingsState>();
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
async fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let state = app.state::<SettingsState>();
    let mut current_settings = state.settings.lock().map_err(|e| e.to_string())?;
    *current_settings = settings.clone();
    settings
        .save_to_path(&state.settings_path)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn send_command(app: AppHandle, command: String) -> Result<(), String> {
    emit_game_message(&app, &format!("> {command}"))?;

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let responses = process_command(&app, &command);
        for response in responses {
            let _ = emit_game_message(&app, &response);
        }
    });

    Ok(())
}

#[tauri::command]
async fn get_start_message(app: AppHandle) -> Result<(), String> {
    let state = app.state::<GameState>();
    let messages = if let Ok(game_lock) = state.game.lock() {
        if let Some(game) = game_lock.as_ref() {
            game.get_current_room_display()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    tauri::async_runtime::spawn(async move {
        let _ = emit_game_message(&app, "=== Welcome to Muddy Rogue ===");
        let _ = emit_game_message(&app, "Type 'help' for available commands.");
        let _ = emit_game_message(&app, "");

        for message in messages {
            let _ = emit_game_message(&app, &message);
        }
    });

    Ok(())
}

#[tauri::command]
async fn get_minimap(app: AppHandle) -> Result<Vec<MinimapNode>, String> {
    let state = app.state::<GameState>();
    let game_lock = state.game.lock().map_err(|e| e.to_string())?;

    let game = game_lock
        .as_ref()
        .ok_or_else(|| "Game not initialized".to_string())?;

    Ok(generate_minimap(
        &game.player.current_location,
        &game.rooms,
        2,
    ))
}

fn process_command(app: &AppHandle, command: &str) -> Vec<String> {
    let state = app.state::<GameState>();
    let Ok(mut game_lock) = state.game.lock() else {
        return vec!["Error: Failed to acquire game lock.".to_string()];
    };
    let Some(game) = game_lock.as_mut() else {
        return vec!["Error: Game not initialized.".to_string()];
    };

    let cmd = command.trim().to_lowercase();

    // Try movement command first
    if let Ok(messages) = game.process_move(&cmd) {
        let _ = app.emit("minimap-update", ());
        return messages;
    }

    // Other commands
    match cmd.as_str() {
        "help" => HELP_TEXT.iter().map(|s| s.to_string()).collect(),
        "look" | "l" => game.get_current_room_display(),
        "time" => vec![format!(
            "Current time: {}",
            chrono::Local::now().format("%H:%M:%S")
        )],
        _ => vec![format!(
            "Unknown command: '{}'. Type 'help' for available commands.",
            command
        )],
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            setup_menu(app)?;
            initialize_game(app)?;
            initialize_settings(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_command,
            get_start_message,
            get_minimap,
            get_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_menu(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder};

    let quit = MenuItemBuilder::with_id("quit", "Exit").build(app)?;
    let toggle_minimap = CheckMenuItemBuilder::new("Toggle Minimap")
        .id("toggle_minimap")
        .checked(true)
        .build(app)?;
    let open_settings = MenuItemBuilder::with_id("open_settings", "Settings...").build(app)?;

    let file_menu = SubmenuBuilder::new(app, "File").items(&[&quit]).build()?;
    let tools_menu = SubmenuBuilder::new(app, "Tools")
        .items(&[&toggle_minimap, &open_settings])
        .build()?;
    let menu = MenuBuilder::new(app)
        .items(&[&file_menu, &tools_menu])
        .build()?;

    app.set_menu(menu)?;

    app.on_menu_event(move |app, event| {
        match event.id().as_ref() {
            "quit" => app.exit(0),
            "toggle_minimap" => {
                // Emit event to frontend to toggle minimap
                let _ = app.emit("toggle-minimap", ());
            }
            "open_settings" => {
                let _ = app.emit("open-settings", ());
            }
            _ => {}
        }
    });

    Ok(())
}

fn initialize_settings(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_config_dir = app
        .path()
        .app_config_dir()
        .expect("failed to resolve app config dir");

    let settings_path = app_config_dir.join("settings.json");
    std::fs::create_dir_all(&app_config_dir).ok();

    let settings = Settings::load_from_path(&settings_path);
    app.manage(SettingsState {
        settings: Mutex::new(settings),
        settings_path,
    });
    Ok(())
}

fn initialize_game(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let zones_config = include_str!("../rooms/zones.json");
    let zone_files = [("millhaven.json", include_str!("../rooms/millhaven.json"))];
    let game = Game::load_from_zones(zones_config, &zone_files)?;
    app.manage(GameState {
        game: Mutex::new(Some(game)),
    });

    Ok(())
}
