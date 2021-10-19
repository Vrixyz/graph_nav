use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*, shapes::RegularPolygon};

use crate::map_graph::{DisplayRoomReachable, Room, RoomId};

pub struct RoomGraphic {
    is_reachable: bool,
}

impl RoomGraphic {
    pub fn init(
        &mut self,
        is_reachable: bool,
        room_to_update: &Room,
    ) -> (ShapeBundle, Option<DisplayRoomReachable>) {
        let is_reachable = if is_reachable {
            Some(DisplayRoomReachable)
        } else {
            None
        };
        let new_room = create_room_bundle(&is_reachable, room_to_update);
        (new_room, is_reachable)
    }
    pub fn updateReachability(
        &mut self,
        is_reachable: bool,
        room_to_update: &Room,
    ) -> Option<(ShapeBundle, Option<DisplayRoomReachable>)> {
        if self.is_reachable == is_reachable {
            return None;
        }
        self.is_reachable = is_reachable;
        Some(self.init(is_reachable, room_to_update))
    }
}

fn create_room_bundle(
    is_reachable: &Option<DisplayRoomReachable>,
    room: &Room,
) -> bevy_prototype_lyon::entity::ShapeBundle {
    let character = GeometryBuilder::build_as(
        &RegularPolygon {
            sides: 3,
            feature: shapes::RegularPolygonFeature::Radius(10.0),
            ..RegularPolygon::default()
        },
        ShapeColors::outlined(
            if is_reachable.is_some() {
                match room.room_type {
                    crate::map_graph::RoomType::Safe => Color::WHITE,
                    crate::map_graph::RoomType::Danger => Color::ORANGE_RED,
                    crate::map_graph::RoomType::Coins => Color::GREEN,
                    crate::map_graph::RoomType::Price(_) => Color::FUCHSIA,
                }
            } else {
                Color::GRAY
            },
            Color::BLACK,
        ),
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default().with_line_width(3.0),
        },
        Transform::from_xyz(room.position.0, room.position.1, 1.0),
    );
    character
}

pub fn create_room(commands: &mut Commands, room: &Room, id: RoomId, is_reachable: bool) {
    let mut map_graphics = RoomGraphic { is_reachable: true };
    let mut spawning = commands.spawn();
    spawning.insert(id);
    spawning.insert(is_reachable);
    let for_init = map_graphics.init(is_reachable, room);
    spawning.insert_bundle(for_init.0).insert(for_init.1);
    spawning.insert(map_graphics);
}
