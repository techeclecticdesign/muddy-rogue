use crate::room::RoomLocation;

pub struct Player {
    pub(crate) current_location: RoomLocation,
}

impl Player {
    pub fn new(zone: String, room_id: u32) -> Self {
        Self {
            current_location: RoomLocation { zone, room_id },
        }
    }

    pub fn move_to(&mut self, location: RoomLocation) {
        self.current_location = location;
    }
}
