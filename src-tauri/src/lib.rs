mod command_parser;
mod player;
mod room;
mod zone;

use command_parser::{get_room_display, process_move, HELP_TEXT};
use player::Player;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use zone::{load_rooms, RoomMap, ZoneConfig};

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

#[tauri::command]
async fn send_command(app: AppHandle, command: String) -> Result<(), String> {
    app.emit("stream-message", format!("&gt; {command}"))
        .map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let responses = process_command(&app, &command);
        for response in responses {
            let _ = app.emit("stream-message", response);
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
        let _ = app.emit("stream-message", "=== Welcome to Muddy Rogue ===");
        let _ = app.emit("stream-message", "Type 'help' for available commands.");
        let _ = app.emit("stream-message", "");

        for message in messages {
            let _ = app.emit("stream-message", message);
        }
    });

    Ok(())
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_command, get_start_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_menu(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};

    let quit = MenuItemBuilder::with_id("quit", "Exit").build(app)?;
    let file_menu = SubmenuBuilder::new(app, "File").items(&[&quit]).build()?;
    let menu = MenuBuilder::new(app).items(&[&file_menu]).build()?;

    app.set_menu(menu)?;

    app.on_menu_event(move |app, event| {
        if event.id() == "quit" {
            app.exit(0);
        }
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
