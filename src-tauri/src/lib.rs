use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Deserialize, Serialize)]
struct ZoneConfig {
    zones: Vec<ZoneInfo>,
    initial_zone: String,
    initial_room: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct ZoneInfo {
    id: String,
    name: String,
    file: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Room {
    id: u32,
    name: String,
    description: String,
    exits: HashMap<String, String>,
    objects: Vec<u32>,
}

#[derive(Debug, Clone)]
struct RoomLocation {
    zone: String,
    room_id: u32,
}

impl RoomLocation {
    fn parse(exit: &str, current_zone: &str) -> Self {
        if let Some((zone, room)) = exit.split_once(':') {
            RoomLocation {
                zone: zone.to_string(),
                room_id: room.parse().unwrap_or(0),
            }
        } else {
            RoomLocation {
                zone: current_zone.to_string(),
                room_id: exit.parse().unwrap_or(0),
            }
        }
    }

    fn to_key(&self) -> String {
        format!("{}:{}", self.zone, self.room_id)
    }
}

struct Player {
    current_location: RoomLocation,
}

impl Player {
    fn new(zone: String, room_id: u32) -> Self {
        Player {
            current_location: RoomLocation { zone, room_id },
        }
    }

    fn move_to(&mut self, location: RoomLocation) {
        self.current_location = location;
    }
}

struct Game {
    rooms: HashMap<String, (Room, String)>,
    player: Player,
}

impl Game {
    fn load_from_zones(
        zones_json: &str,
        zone_files: &[(&str, &str)],
    ) -> Result<Self, serde_json::Error> {
        let zone_config: ZoneConfig = serde_json::from_str(zones_json)?;

        let mut rooms_map = HashMap::new();

        for zone_info in &zone_config.zones {
            if let Some(&(_, json_data)) = zone_files.iter().find(|(id, _)| *id == zone_info.file) {
                let rooms: Vec<Room> = serde_json::from_str(json_data)?;
                for room in rooms {
                    let key = format!("{}:{}", zone_info.id, room.id);
                    rooms_map.insert(key, (room, zone_info.id.clone()));
                }
            }
        }

        Ok(Game {
            rooms: rooms_map,
            player: Player::new(zone_config.initial_zone, zone_config.initial_room),
        })
    }

    fn get_current_room_display(&self) -> Vec<String> {
        let mut messages = Vec::new();

        let key = self.player.current_location.to_key();
        if let Some((room, _)) = self.rooms.get(&key) {
            messages.push(format!("**{}**", room.name));
            messages.push(room.description.clone());

            if !room.exits.is_empty() {
                messages.push(String::new()); // Empty line
                messages.push(self.format_exits(&room.exits));
            }
        }

        messages
    }

    fn format_exits(&self, exits: &HashMap<String, String>) -> String {
        if exits.is_empty() {
            return String::new();
        }

        let exit_names: Vec<String> = exits.keys().map(|s| format!("**{s}**")).collect();

        if exit_names.len() == 1 {
            format!("There is an available exit to the {}.", exit_names[0])
        } else {
            let (last, rest) = exit_names.split_last().unwrap();
            if rest.len() == 1 {
                format!("There are available exits to the {} and {}.", rest[0], last)
            } else {
                format!(
                    "There are available exits to the {} and {}.",
                    rest.join(", "),
                    last
                )
            }
        }
    }

    fn expand_direction(input: &str) -> String {
        match input.to_lowercase().as_str() {
            "n" => "north".to_string(),
            "s" => "south".to_string(),
            "e" => "east".to_string(),
            "w" => "west".to_string(),
            "ne" => "northeast".to_string(),
            "nw" => "northwest".to_string(),
            "se" => "southeast".to_string(),
            "sw" => "southwest".to_string(),
            "u" => "up".to_string(),
            "d" => "down".to_string(),
            other => other.to_string(),
        }
    }

    fn process_move(&mut self, command: &str) -> Result<Vec<String>, String> {
        let direction = Self::expand_direction(command);

        let key = self.player.current_location.to_key();
        if let Some((room, zone)) = self.rooms.get(&key) {
            if let Some(destination_str) = room.exits.get(&direction) {
                let destination = RoomLocation::parse(destination_str, zone);

                // Check if destination exists
                if self.rooms.contains_key(&destination.to_key()) {
                    self.player.move_to(destination);
                } else {
                    return Err(format!(
                        "Error: Destination room not found ({})",
                        destination.to_key()
                    ));
                }

                Ok(self.get_current_room_display())
            } else {
                Err("You can't go that way.".to_string())
            }
        } else {
            Err("Error: Current room not found.".to_string())
        }
    }
}

struct GameState {
    game: Mutex<Option<Game>>,
}

#[tauri::command]
async fn send_command(app: AppHandle, command: String) -> Result<(), String> {
    // Echo the command back
    app.emit("stream-message", format!("&gt; {command}"))
        .map_err(|e| e.to_string())?;

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let responses = process_command(&app_clone, &command);
        for response in responses {
            let _ = app_clone.emit("stream-message", response);
        }
    });

    Ok(())
}

#[tauri::command]
async fn get_start_message(app: AppHandle) -> Result<(), String> {
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = app_clone.emit("stream-message", "=== Welcome to Muddy Rogue ===");
        let _ = app_clone.emit("stream-message", "Type 'help' for available commands.");
        let _ = app_clone.emit("stream-message", "");

        let state = app_clone.state::<GameState>();
        let game_lock = state.game.lock().unwrap();
        if let Some(game) = game_lock.as_ref() {
            for message in game.get_current_room_display() {
                let _ = app_clone.emit("stream-message", message);
            }
        }
    });
    Ok(())
}

fn process_command(app: &AppHandle, command: &str) -> Vec<String> {
    let state = app.state::<GameState>();
    let mut game_lock = state.game.lock().unwrap();

    if let Some(game) = game_lock.as_mut() {
        let cmd = command.trim().to_lowercase();

        // Try to process as movement command first
        match game.process_move(&cmd) {
            Ok(messages) => return messages,
            Err(_) => {
                // Not a movement command, fall through to other commands
            }
        }

        // Other commands
        match cmd.as_str() {
            "help" => vec![
                "Available commands:".to_string(),
                "  Movement: n, s, e, w, ne, nw, se, sw, u, d (or full direction names)"
                    .to_string(),
                "  Other: help, look, time".to_string(),
            ],
            "look" | "l" => game.get_current_room_display(),
            "time" => vec![format!(
                "Current time: {}",
                chrono::Local::now().format("%H:%M:%S")
            )],
            _ => vec![format!(
                "Unknown command: {}. Type 'help' for available commands.",
                command
            )],
        }
    } else {
        vec!["Error: Game not initialized.".to_string()]
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Add menu
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
            // Initialize game state
            let zones_config = include_str!("../rooms/zones.json");
            let zone_files = [
                ("millhaven.json", include_str!("../rooms/millhaven.json")),
                // Add more zone files here as needed
            ];

            let game =
                Game::load_from_zones(zones_config, &zone_files).expect("Failed to load game data");

            app.manage(GameState {
                game: Mutex::new(Some(game)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_command, get_start_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
