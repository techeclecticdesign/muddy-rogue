use crate::player::Player;
use crate::room::{format_exits, RoomLocation};
use crate::zone::RoomMap;

pub const HELP_TEXT: [&str; 3] = [
    "Available commands:",
    "  Movement: n, s, e, w, ne, nw, se, sw, u, d (or full direction names)",
    "  Other: help, look, time",
];

pub fn expand_direction(input: &str) -> &str {
    match input {
        "n" => "north",
        "s" => "south",
        "e" => "east",
        "w" => "west",
        "ne" => "northeast",
        "nw" => "northwest",
        "se" => "southeast",
        "sw" => "southwest",
        "u" => "up",
        "d" => "down",
        other => other,
    }
}

pub fn process_move(
    player: &mut Player,
    rooms: &RoomMap,
    command: &str,
) -> Result<Vec<String>, String> {
    let direction = expand_direction(command);

    let key = player.current_location.to_key();
    let (room, zone) = rooms
        .get(&key)
        .ok_or_else(|| "Error: Current room not found.".to_string())?;

    let destination_str = room
        .exits
        .get(direction)
        .ok_or_else(|| "You can't go that way.".to_string())?;

    let destination = RoomLocation::parse(destination_str, zone);

    if !rooms.contains_key(&destination.to_key()) {
        return Err(format!(
            "Error: Destination room not found ({})",
            destination.to_key()
        ));
    }

    player.move_to(destination);
    Ok(get_room_display(player, rooms))
}

pub fn get_room_display(player: &Player, rooms: &RoomMap) -> Vec<String> {
    let key = player.current_location.to_key();
    let Some((room, _)) = rooms.get(&key) else {
        return Vec::new();
    };

    let mut messages = vec![format!("**{}**", room.name), room.description.clone()];

    if !room.exits.is_empty() {
        messages.push(String::new());
        messages.push(format_exits(&room.exits));
    }

    messages
}
