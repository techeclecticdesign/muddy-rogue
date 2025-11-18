use crate::room::RoomLocation;
use crate::zone::RoomMap;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Serialize)]
pub struct MinimapNode {
    pub x: i32,
    pub y: i32,
    pub room_key: String,
    pub room_name: String,
    pub is_player: bool,
    pub connections: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    fn apply_direction(&self, direction: &str) -> Option<Self> {
        match direction {
            "north" => Some(Coord {
                x: self.x,
                y: self.y + 1,
            }),
            "south" => Some(Coord {
                x: self.x,
                y: self.y - 1,
            }),
            "east" => Some(Coord {
                x: self.x + 1,
                y: self.y,
            }),
            "west" => Some(Coord {
                x: self.x - 1,
                y: self.y,
            }),
            "northeast" => Some(Coord {
                x: self.x + 1,
                y: self.y + 1,
            }),
            "northwest" => Some(Coord {
                x: self.x - 1,
                y: self.y + 1,
            }),
            "southeast" => Some(Coord {
                x: self.x + 1,
                y: self.y - 1,
            }),
            "southwest" => Some(Coord {
                x: self.x - 1,
                y: self.y - 1,
            }),
            _ => None,
        }
    }
}

const CARDINAL_DIRECTIONS: [&str; 8] = [
    "north",
    "south",
    "east",
    "west",
    "northeast",
    "northwest",
    "southeast",
    "southwest",
];

pub fn generate_minimap(
    player_location: &RoomLocation,
    rooms: &RoomMap,
    max_distance: i32,
) -> Vec<MinimapNode> {
    let mut nodes = Vec::new();
    let mut visited = HashMap::new();
    let mut queue = VecDeque::new();

    let player_key = player_location.to_key();
    let origin = Coord { x: 0, y: 0 };

    queue.push_back((player_key.clone(), origin));
    visited.insert(player_key.clone(), origin);

    while let Some((room_key, coord)) = queue.pop_front() {
        // Add current room to minimap
        if let Some((room, zone)) = rooms.get(&room_key) {
            let mut connections = Vec::new();

            for direction in &CARDINAL_DIRECTIONS {
                let exit_str = match room.exits.get(*direction) {
                    Some(s) => s,
                    None => continue,
                };

                let next_location = RoomLocation::parse(exit_str, zone);
                let next_key = next_location.to_key();

                if !rooms.contains_key(&next_key) {
                    continue;
                }

                let next_coord = match coord.apply_direction(direction) {
                    Some(c) => c,
                    None => continue,
                };

                // Check if next coordinate is within the max_distance boundary
                if next_coord.x.abs() <= max_distance && next_coord.y.abs() <= max_distance {
                    // Add to connections regardless of whether we've visited
                    connections.push(next_key.clone());

                    // Only queue if not visited
                    if !visited.contains_key(&next_key) {
                        visited.insert(next_key.clone(), next_coord);
                        queue.push_back((next_key, next_coord));
                    }
                }
            }

            nodes.push(MinimapNode {
                x: coord.x,
                y: coord.y,
                room_key: room_key.clone(),
                room_name: room.name.clone(),
                is_player: room_key == player_key,
                connections,
            });
        }
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::Room;
    use std::collections::{HashMap, HashSet};

    fn create_test_room(id: u32, name: &str, exits: Vec<(&str, &str)>) -> Room {
        Room {
            id,
            name: name.to_string(),
            description: "Test room".to_string(),
            exits: exits
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            objects: vec![],
        }
    }

    #[test]
    fn test_only_cardinal_directions() {
        let mut rooms = HashMap::new();

        let start = create_test_room(0, "Start", vec![("north", "1"), ("up", "2")]);
        let r1 = create_test_room(1, "North", vec![("south", "0")]);
        let r2 = create_test_room(2, "Up", vec![("down", "0")]);

        rooms.insert("test:0".to_string(), (start, "test".to_string()));
        rooms.insert("test:1".to_string(), (r1, "test".to_string()));
        rooms.insert("test:2".to_string(), (r2, "test".to_string()));

        let player_location = RoomLocation {
            zone: "test".to_string(),
            room_id: 0,
        };

        let minimap = generate_minimap(&player_location, &rooms, 2);
        let room_keys: HashSet<String> = minimap.iter().map(|n| n.room_key.clone()).collect();

        assert!(room_keys.contains("test:0"));
        assert!(room_keys.contains("test:1"));
        assert!(
            !room_keys.contains("test:2"),
            "Should not include 'up' exit"
        );
    }
}
