use std::ops::Mul;

use bevy::{ecs::system::EntityCommands, prelude::*, render::pipeline::RenderPipeline};
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*, shapes::RegularPolygon};

use crate::{
    map_graph::{DisplayRoomReachable, Room, RoomEntity, RoomId},
    shapes::ShapeMeshes,
};

pub struct RoomGraphic {
    is_reachable: bool,
}

pub struct RoomGraphUpdate {
    pub mesh_bundle: MeshBundle,
    pub material: Handle<crate::shapes::ColorMaterial>,
}

impl RoomGraphic {
    pub fn init(
        &mut self,
        shapes: &Res<ShapeMeshes>,
        is_reachable: bool,
        room_to_update: &Room,
    ) -> (RoomGraphUpdate, Option<DisplayRoomReachable>) {
        let is_reachable = if is_reachable {
            Some(DisplayRoomReachable)
        } else {
            None
        };
        let new_room = create_room_bundle(&shapes, &is_reachable, room_to_update);
        (new_room, is_reachable)
    }
    pub fn updateReachability(
        &mut self,
        shapes: &Res<ShapeMeshes>,
        is_reachable: bool,
        room_to_update: &Room,
    ) -> Option<(RoomGraphUpdate, Option<DisplayRoomReachable>)> {
        if self.is_reachable == is_reachable {
            return None;
        }
        self.is_reachable = is_reachable;
        Some(self.init(&shapes, is_reachable, room_to_update))
    }
}

fn create_room_bundle(
    shapes: &Res<ShapeMeshes>,
    is_reachable: &Option<DisplayRoomReachable>,
    room: &Room,
) -> RoomGraphUpdate {
    let material = if is_reachable.is_some() {
        match room.room_type {
            crate::map_graph::RoomType::Safe => shapes.mat_white.clone(),
            crate::map_graph::RoomType::Danger => shapes.mat_orange.clone(),
            crate::map_graph::RoomType::Coins => shapes.mat_green.clone(),
            crate::map_graph::RoomType::Price(_) => shapes.mat_fuchsia.clone(),
        }
    } else {
        shapes.mat_gray.clone()
    };

    let mut transform = Transform::from_xyz(room.position.0, room.position.1, 10.0);
    transform.scale = Vec3::ONE * 15.0;
    let mesh = MeshBundle {
        mesh: shapes.quad2x2.clone(),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            shapes.pipeline_triangle.clone(),
        )]),
        transform,
        ..Default::default()
    };
    RoomGraphUpdate {
        material,
        mesh_bundle: mesh,
    }
}

pub fn create_room(
    shapes: &Res<ShapeMeshes>,
    commands: &mut Commands,
    room: &Room,
    id: RoomId,
    is_reachable: bool,
) -> Entity {
    let mut map_graphics = RoomGraphic { is_reachable: true };
    let mut spawning = commands.entity(room.entity);
    spawning.insert(id);
    spawning.insert(is_reachable);
    let for_init = map_graphics.init(shapes, is_reachable, room);
    spawning
        .insert_bundle(for_init.0.mesh_bundle)
        .insert(for_init.0.material)
        .insert(for_init.1);
    spawning.insert(map_graphics);
    return spawning.id();
}
