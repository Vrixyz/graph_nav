use crate::{
    danger::{
        check_player_death, grow_danger_zone, update_danger_visual, DangerZone, GrowDangerZone,
    },
    math_utils,
    poisson::Poisson,
};
use bevy::{prelude::*, render::camera::OrthographicProjection, utils::HashMap};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{Circle, Line, RegularPolygon, RegularPolygonFeature},
};

pub struct MapGraphPlugin;

#[derive(PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct RoomId(usize);

pub struct DisplayRoomReachable;

#[derive(Default)]
pub struct Room {
    pub connections: Vec<RoomId>,
    pub position: (f32, f32),
}

#[derive(Default)]
pub struct MapDef {
    pub rooms: HashMap<RoomId, Room>,
}

pub struct MapCreateRoom {
    from_room_id: RoomId,
}

pub struct MapPosition {
    pub pos_id: RoomId,
}
pub struct PlayerPositionDisplay;

pub struct MainCamera;

#[derive(Default)]
pub struct UserInputs {
    pub list: Vec<UserInput>,
}

pub enum UserInput {
    Click(Vec2),
}

impl Plugin for MapGraphPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(ShapePlugin);
        app.add_startup_system(create_map.system());
        app.add_startup_system_to_stage(StartupStage::PostStartup, init_display_map.system());
        app.add_system(update_player_position.system());
        //app.add_system(input_player_position.system());
        app.add_system(base_input.system().label("base_input"));
        app.add_system(handle_input.system().after("base_input"));
        app.add_system(create_new_rooms.system());
        app.add_system(check_player_death.system());
        app.add_system(grow_danger_zone.system());
        app.add_system(update_danger_visual.system());
        app.add_system(update_map_reachabiliy.system());
        app.add_system(react_to_move_player.system());
        app.add_system(update_camera_position.system());
        app.insert_resource(UserInputs::default());
    }
}

fn update_camera_position(
    time: Res<Time>,
    mut qs: QuerySet<(
        Query<&mut Transform, With<MainCamera>>,
        Query<&Transform, With<PlayerPositionDisplay>>,
    )>,
) {
    let mut target = Vec3::ZERO;
    let mut position_count = 0;
    for target_pos in qs.q1().iter() {
        target += target_pos.translation;
        position_count += 1;
    }
    if position_count == 0 {
        return;
    }
    target /= position_count as f32;
    for mut camera in qs.q0_mut().iter_mut() {
        camera.translation =
            math_utils::move_towards(camera.translation, target, 100f32 * time.delta_seconds());
    }
}

fn create_map(mut commands: Commands) {
    let mut cameraBundle = OrthographicCameraBundle::new_2d();
    cameraBundle.orthographic_projection.scale = 0.8;
    commands.spawn_bundle(cameraBundle).insert(MainCamera);

    let mut positions = vec![(0f32, 0f32)];
    let poisson = Poisson::new();
    let mut root_index = RoomId(0);
    let nb_new_shapes = 2;
    let mut new_map = HashMap::default();
    for i in 0..positions.len() {
        new_map.insert(
            RoomId(i),
            Room {
                connections: Default::default(),
                position: positions[i],
            },
        );
    }
    let mut new_map = MapDef { rooms: new_map };

    let mut rng = rand::thread_rng();
    let mut room_id_to_create = RoomId(1);
    while root_index.0 < positions.len() && new_map.rooms.len() < nb_new_shapes {
        let ref_point = positions[root_index.0];

        if let Some(new_position) =
            poisson.compute_new_position(&positions, &ref_point, 40f32, 5, &mut rng)
        {
            {
                new_map
                    .rooms
                    .entry(root_index)
                    .or_default()
                    .connections
                    .push(room_id_to_create);
            }
            let new_room = Room {
                connections: vec![root_index],
                position: new_position,
            };
            {
                new_map.rooms.insert(room_id_to_create, new_room);
            }
            positions.push(new_position);
            room_id_to_create.0 += 1;
        } else {
            root_index.0 += 1;
        }
    }
    commands.insert_resource(new_map);
    commands.insert_resource(MapPosition { pos_id: RoomId(0) });

    let mut circle_transform = Transform::from_xyz(15.0, 15.0, 0.0);
    circle_transform.scale = Vec3::ONE;
    let circle = GeometryBuilder::build_as(
        &Circle {
            radius: 10f32,
            center: Vec2::ZERO,
        },
        ShapeColors::outlined(Color::NONE, Color::RED),
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default().with_line_width(3.0),
        },
        Transform::from_xyz(15.0, 15.0, 0.0),
    );
    commands
        .spawn()
        .insert_bundle(circle)
        .insert(DangerZone { size: 1f32 })
        .insert(GrowDangerZone {
            radius_increase_per_second: 10f32,
        });
}

fn init_display_map(mut commands: Commands, map: Res<MapDef>) {
    if map.rooms.is_empty() {
        return;
    }
    let mut visit_queue = vec![&RoomId(0)];
    let mut visit_index = RoomId(0);
    while visit_index.0 < visit_queue.len() {
        let room = &map.rooms[visit_queue[visit_index.0]];
        create_room(&mut commands, room, visit_index, Some(DisplayRoomReachable));
        for connection in &room.connections {
            create_link(&mut commands, &map.rooms[connection], room);
            if !visit_queue.contains(&connection) {
                visit_queue.push(connection);
            }
        }
        visit_index.0 += 1;
    }
    let first_room = &map.rooms[&RoomId(0)];
    let character = GeometryBuilder::build_as(
        &Circle {
            radius: 18.0,
            center: Default::default(),
        },
        ShapeColors::outlined(Color::NONE, Color::WHITE),
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default().with_line_width(3.0),
        },
        Transform::from_xyz(first_room.position.0, first_room.position.1, 0.0),
    );
    commands
        .spawn_bundle(character)
        .insert(PlayerPositionDisplay);
}

fn base_input(
    window: Res<Windows>,
    mut user_inputs: ResMut<UserInputs>,
    mouse_button_input: Res<Input<MouseButton>>,
    q_camera: Query<(&Transform, &OrthographicProjection), With<MainCamera>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let win = window.get_primary().expect("no primary window");
        if let Some(pos) = win.cursor_position() {
            let (camera_transform, projection) = q_camera.iter().next().unwrap();
            let size = Vec2::new(win.width() as f32, win.height() as f32);

            // the default orthographic projection is in pixels from the center;
            // just undo the translation
            let p = (pos - size / 2.0) * projection.scale;

            // apply the camera transform
            let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);

            user_inputs
                .list
                .push(UserInput::Click(Vec2::new(pos_wld.x, pos_wld.y)));
        }
    }
}

fn update_map_reachabiliy(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<f32>,
    map: Res<MapDef>,
    player_pos: Res<MapPosition>,
    rooms: Query<(Entity, &RoomId)>,
) {
    *timer += time.delta_seconds();
    if *timer < 0.1f32 {
        return;
    }
    *timer = 0f32;
    let current_room = &map.rooms[&player_pos.pos_id];
    for (e, r) in rooms.iter() {
        let room_to_update = &map.rooms[&r];
        if current_room.connections.contains(&r) {
            let isReachable = Some(DisplayRoomReachable);
            let new_room = create_room_bundle(&isReachable, room_to_update);
            commands.entity(e).insert_bundle(new_room);
            commands.entity(e).insert(isReachable);
        } else {
            let isReachable: Option<DisplayRoomReachable> = None;
            let new_room = create_room_bundle(&isReachable, room_to_update);
            commands.entity(e).insert_bundle(new_room);
            commands.entity(e).insert(isReachable);
        }
    }
}

fn react_to_move_player(
    mut commands: Commands,
    map: Res<MapDef>,
    position_changed: Res<MapPosition>,
) {
    if !position_changed.is_changed() {
        return;
    }
    commands.spawn().insert(MapCreateRoom {
        from_room_id: position_changed.pos_id,
    });
    commands.spawn().insert(MapCreateRoom {
        from_room_id: position_changed.pos_id,
    });
    // TODO: update reachable rooms display
}

fn handle_input(
    mut commands: Commands,
    map: Res<MapDef>,
    mut inputs: ResMut<UserInputs>,
    mut position: ResMut<MapPosition>,
) {
    for click in inputs.list.iter() {
        let UserInput::Click(click) = click;
        for id in map.rooms.keys() {
            let r = map.rooms.get(id).unwrap();
            let room_position = Vec2::new(r.position.0, r.position.1);

            let distance_to_room = room_position.distance(*click);
            if distance_to_room < 15.0 {
                position.pos_id = *id;
                break;
            }
        }
    }
    inputs.list.clear();
}

fn create_new_rooms(
    mut commands: Commands,
    mut map: ResMut<MapDef>,
    q_create: Query<(Entity, &MapCreateRoom)>,
) {
    for (e, create) in q_create.iter() {
        let mut rng = rand::thread_rng();
        let poisson = Poisson::new();
        let existing_points: Vec<(f32, f32)> = map.rooms.values().map(|r| r.position).collect();

        let room_id_to_create = RoomId(map.rooms.len());
        if let Some(ref_point) = map.rooms.get(&create.from_room_id).map(|f| f.position) {
            if let Some(new_position) =
                poisson.compute_new_position(&existing_points, &ref_point, 40f32, 10, &mut rng)
            {
                {
                    map.rooms
                        .entry(create.from_room_id)
                        .or_default()
                        .connections
                        .push(room_id_to_create);
                }
                let new_room = Room {
                    connections: vec![create.from_room_id],
                    position: new_position,
                };
                create_room(
                    &mut commands,
                    &new_room,
                    room_id_to_create,
                    Some(DisplayRoomReachable),
                );
                create_link(
                    &mut commands,
                    map.rooms.get(&create.from_room_id).unwrap(),
                    &new_room,
                );
                {
                    map.rooms.insert(room_id_to_create, new_room);
                }
            }
            commands.entity(e).despawn();
        }
    }
}

fn update_player_position(
    position: Res<MapPosition>,
    map: Res<MapDef>,
    mut q_pos: Query<&mut Transform, With<PlayerPositionDisplay>>,
) {
    if let Some(room_target) = map.rooms.get_key_value(&position.pos_id) {
        let target_position =
            Vec2::new(room_target.1.position.0, room_target.1.position.1).extend(0.0);
        for mut t in q_pos.iter_mut() {
            t.translation = target_position;
        }
    }
}

fn create_link(commands: &mut Commands, from: &Room, to: &Room) {
    let character = GeometryBuilder::build_as(
        &Line(
            Vec2::new(from.position.0, from.position.1),
            Vec2::new(to.position.0, to.position.1),
        ),
        ShapeColors::outlined(Color::CYAN, Color::BLACK),
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default().with_line_width(3.0),
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    );
    commands.spawn_bundle(character);
}

fn create_room(
    commands: &mut Commands,
    room: &Room,
    id: RoomId,
    is_reachable: Option<DisplayRoomReachable>,
) {
    let character = create_room_bundle(&is_reachable, room);
    let mut spawning = commands.spawn_bundle(character);
    spawning.insert(id);
    spawning.insert(is_reachable);
}

fn create_room_bundle(
    is_reachable: &Option<DisplayRoomReachable>,
    room: &Room,
) -> bevy_prototype_lyon::entity::ShapeBundle {
    let character = GeometryBuilder::build_as(
        &RegularPolygon {
            sides: 3,
            feature: RegularPolygonFeature::Radius(10.0),
            ..RegularPolygon::default()
        },
        ShapeColors::outlined(
            if is_reachable.is_some() {
                Color::GREEN
            } else {
                Color::GRAY
            },
            Color::BLACK,
        ),
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default().with_line_width(3.0),
        },
        Transform::from_xyz(room.position.0, room.position.1, 0.0),
    );
    character
}
