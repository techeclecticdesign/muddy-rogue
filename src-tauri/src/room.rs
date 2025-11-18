use crate::text_utils::format_list;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Room {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, String>,
    pub objects: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct RoomLocation {
    pub zone: String,
    pub room_id: u32,
}

impl RoomLocation {
    pub fn parse(exit: &str, current_zone: &str) -> Self {
        if let Some((zone, room)) = exit.split_once(':') {
            Self {
                zone: zone.to_string(),
                room_id: room.parse().unwrap_or(0),
            }
        } else {
            Self {
                zone: current_zone.to_string(),
                room_id: exit.parse().unwrap_or(0),
            }
        }
    }

    pub fn to_key(&self) -> String {
        format!("{}:{}", self.zone, self.room_id)
    }
}

pub fn format_exits(exits: &HashMap<String, String>) -> String {
    if exits.is_empty() {
        return String::new();
    }

    let exit_names: Vec<String> = exits.keys().map(|s| format!("**{s}**")).collect();
    let formatted_list = format_list(&exit_names);

    if exit_names.len() == 1 {
        format!("There is an available exit to the {formatted_list}.")
    } else {
        format!("There are available exits to the {formatted_list}.")
    }
}
