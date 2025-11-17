use crate::room::Room;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ZoneConfig {
    pub zones: Vec<ZoneInfo>,
    pub initial_zone: String,
    pub initial_room: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ZoneInfo {
    pub id: String,
    pub name: String,
    pub file: String,
}

pub type RoomMap = HashMap<String, (Room, String)>;

pub fn load_rooms(
    zones: &[ZoneInfo],
    zone_files: &[(&str, &str)],
) -> Result<RoomMap, serde_json::Error> {
    let mut rooms = HashMap::new();

    for zone_info in zones {
        let Some(&(_, json_data)) = zone_files.iter().find(|(id, _)| *id == zone_info.file) else {
            continue;
        };

        let zone_rooms: Vec<Room> = serde_json::from_str(json_data)?;
        for room in zone_rooms {
            let key = format!("{}:{}", zone_info.id, room.id);
            rooms.insert(key, (room, zone_info.id.clone()));
        }
    }

    Ok(rooms)
}
