use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Room {
    id: u32,
    name: String,
    description: String,
    exits: HashMap<String, u32>,
    objects: Vec<u32>,
}

struct Player {
    current_room: u32,
}

impl Player {
    fn new(starting_room: u32) -> Self {
        Player {
            current_room: starting_room,
        }
    }

    fn move_to(&mut self, room_id: u32) {
        self.current_room = room_id;
    }
}

struct Game {
    rooms: HashMap<u32, Room>,
    player: Player,
}

impl Game {
    fn new(rooms: Vec<Room>) -> Self {
        let rooms_map: HashMap<u32, Room> = rooms.into_iter().map(|room| (room.id, room)).collect();

        Game {
            rooms: rooms_map,
            player: Player::new(0),
        }
    }

    fn load_from_json(json_data: &str) -> Result<Self, serde_json::Error> {
        let rooms: Vec<Room> = serde_json::from_str(json_data)?;
        Ok(Game::new(rooms))
    }

    fn get_current_room_display(&self) -> Vec<String> {
        let mut messages = Vec::new();

        if let Some(room) = self.rooms.get(&self.player.current_room) {
            messages.push(format!("**{}**", room.name));
            messages.push(room.description.clone());

            if !room.exits.is_empty() {
                messages.push(String::new()); // Empty line
                messages.push(self.format_exits(&room.exits));
            }
        }

        messages
    }

    fn format_exits(&self, exits: &HashMap<String, u32>) -> String {
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
            other => other.to_string(),
        }
    }

    fn process_move(&mut self, command: &str) -> Result<Vec<String>, String> {
        let direction = Self::expand_direction(command);

        if let Some(room) = self.rooms.get(&self.player.current_room) {
            if let Some(&destination) = room.exits.get(&direction) {
                self.player.move_to(destination);
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
    app.emit("stream-message", format!("> {command}"))
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
                "  Movement: n, s, e, w, ne, nw, se, sw (or full direction names)".to_string(),
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
            // Initialize game state
            let rooms_json = include_str!("../default.json");
            let game = Game::load_from_json(rooms_json).expect("Failed to load game data");

            app.manage(GameState {
                game: Mutex::new(Some(game)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_command, get_start_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
