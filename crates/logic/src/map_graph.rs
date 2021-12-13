use crate::combat::{Battle, CombatPlugin, IsDirty};
use crate::danger::{
    danger_zone_grow_speedup, DangerSpeedModifier, SpawnDangerZone, SpawnDangerZoneCommand,
};
use crate::delayed_destroy::destroy_after;
use crate::graphics_rooms::{create_room, RoomGraphic};
use crate::shapes::{CircleGaugeMaterial, ShapeMeshes, ShapesPlugin};
use crate::text_feedback::{show_text_feedback, spawn_text_feedback, TextFeedbackSpawn};
use crate::AppState;
use crate::{
    danger::{check_player_death, grow_danger_zone, update_danger_visual},
    math_utils,
    poisson::Poisson,
};
use bevy::render::pipeline::RenderPipeline;
use bevy::{prelude::*, render::camera::OrthographicProjection, utils::HashMap};
use bevy_prototype_lyon::{prelude::*, shapes::Line};
use rand::{thread_rng, Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
pub struct MapGraphPlugin;

#[derive(PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct RoomId(usize);

pub struct DisplayRoomReachable;

pub struct Room {
    pub connections: Vec<RoomId>,
    pub position: (f32, f32),
    pub room_type: RoomType,
    pub entity: Entity,
}
pub struct RoomEntity {
    // TODO: this probably be the transform actually, and remove position from Room..
    pub position: (f32, f32),
}

#[derive(Default)]
pub struct MapDef {
    pub rooms: HashMap<RoomId, Room>,
}

#[derive(Clone, PartialEq)]
pub enum RoomType {
    Danger,
    Safe,
    Coins,
    Price(u32),
}

impl Default for RoomType {
    fn default() -> Self {
        Self::Safe
    }
}

pub struct MapConfiguration {
    pub start_with_danger_zone: bool,
    pub speed_gain_danger: f32,
    pub speed_init_danger: f32,

    pub weight_room_danger: f32,
    pub weight_room_shop: f32,
    pub weight_room_normal: f32,
}

pub struct RandomDeterministic {
    pub random: ChaCha20Rng,
    pub seed: u64,
}

impl Default for RandomDeterministic {
    fn default() -> Self {
        let seed = thread_rng().gen::<u64>();
        Self {
            random: ChaCha20Rng::seed_from_u64(seed),
            seed,
        }
    }
}

impl RandomDeterministic {
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
        self.random = ChaCha20Rng::seed_from_u64(seed);
    }
}

impl Default for MapConfiguration {
    fn default() -> Self {
        Self {
            start_with_danger_zone: true,
            speed_gain_danger: 0.1f32,
            speed_init_danger: 10f32,
            weight_room_danger: Default::default(),
            weight_room_shop: Default::default(),
            weight_room_normal: Default::default(),
        }
    }
}

pub struct MapCreateRoom {
    from_room_id: RoomId,
    room_type: RoomType,
    battle: Option<Battle>,
}

pub struct MapPosition {
    pub pos_id: RoomId,
    pub will_move: Option<RoomId>,
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

pub struct Coins {
    pub amount: u32,
}

pub struct Cooldown {
    last_action_time: f32,
    base_cooldown: f32,
}

impl Cooldown {
    pub fn is_ready(&self, time: &Time) -> bool {
        self.last_action_time + self.base_cooldown <= time.seconds_since_startup() as f32
    }

    pub fn get_ratio(&self, time: &Time) -> f32 {
        if self.is_ready(time) {
            return 1f32;
        }
        let total_time = self.base_cooldown;
        let time_left =
            (self.last_action_time + self.base_cooldown) - time.seconds_since_startup() as f32;
        let ratio = time_left / total_time;
        1f32 - ratio
    }
}

pub struct PermanentEntity;

impl Plugin for MapGraphPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(ShapesPlugin);
        app.add_plugin(CombatPlugin);
        let loading_startup_system_set =
            SystemSet::on_enter(AppState::Loading).with_system(create_map.system());
        app.add_system_set(loading_startup_system_set);
        let loading_update_system_set =
            SystemSet::on_update(AppState::Loading).with_system(loading_update.system());
        app.add_system_set(loading_update_system_set);

        let game_startup_system_set =
            SystemSet::on_enter(AppState::Game).with_system(init_display_map.system());
        app.add_system_set(game_startup_system_set);

        let game_exit_system_set =
            SystemSet::on_exit(AppState::Game).with_system(game_cleanup.system());
        app.add_system_set(game_exit_system_set);

        let game_update_system_set = SystemSet::on_update(AppState::Game)
            .with_system(update_player_position.system())
            .with_system(base_input.system().label("base_input"))
            .with_system(handle_input.system().after("base_input"))
            .with_system(create_new_rooms.system())
            .with_system(SpawnDangerZone.system())
            .with_system(check_player_death.system())
            .with_system(grow_danger_zone.system())
            .with_system(update_danger_visual.system())
            .with_system(update_map_reachabiliy.system())
            .with_system(react_to_move_player.system())
            .with_system(update_camera_position.system())
            .with_system(danger_zone_grow_speedup.system())
            .with_system(spawn_text_feedback.system())
            .with_system(destroy_after.system())
            .with_system(cooldown_material_update.system())
            .with_system(show_text_feedback.system());
        app.add_system_set(game_update_system_set);

        app.insert_resource(UserInputs::default());
        app.insert_resource(DangerSpeedModifier { multiplier: 1f32 });
        app.insert_resource(Coins { amount: 0u32 });
        app.insert_resource(MapConfiguration::default());
        app.insert_resource(RandomDeterministic::default());
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
        target.z = camera.translation.z;
        camera.translation =
            math_utils::move_towards(camera.translation, target, 500f32 * time.delta_seconds());
    }
}

fn game_cleanup(mut commands: Commands, to_remove: Query<Entity, Without<PermanentEntity>>) {
    for e in to_remove.iter() {
        commands.entity(e).despawn();
    }
}

fn loading_update(mut state: ResMut<State<AppState>>) {
    state.set(AppState::Game);
}

fn create_map(
    mut commands: Commands,
    time: Res<Time>,
    map_configuration: Res<MapConfiguration>,
    mut random: ResMut<RandomDeterministic>,
) {
    let seed = random.seed;
    random.set_seed(seed);

    let mut cameraBundle = OrthographicCameraBundle::new_2d();
    cameraBundle.orthographic_projection.scale = 0.3;
    commands.spawn_bundle(cameraBundle).insert(MainCamera);
    commands.insert_resource(Coins { amount: 0u32 });
    commands.insert_resource(DangerSpeedModifier { multiplier: 1f32 });

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
                room_type: RoomType::Safe,
                entity: commands
                    .spawn()
                    .insert(RoomEntity {
                        position: positions[i],
                    })
                    .id(),
            },
        );
    }
    let mut new_map = MapDef { rooms: new_map };

    let mut rng = &mut random.random;
    let mut room_id_to_create = RoomId(1);
    while root_index.0 < positions.len() && new_map.rooms.len() < nb_new_shapes {
        let ref_point = positions[root_index.0];

        if let Some(new_position) =
            poisson.compute_new_position(&positions, &ref_point, 40f32, 5, &mut rng)
        {
            {
                match new_map.rooms.entry(root_index) {
                    std::collections::hash_map::Entry::Occupied(mut room) => {
                        room.get_mut().connections.push(room_id_to_create);
                    }
                    std::collections::hash_map::Entry::Vacant(_) => return,
                };
            }
            let new_room = Room {
                connections: vec![root_index],
                position: new_position,
                room_type: RoomType::Safe,
                entity: commands
                    .spawn()
                    .insert(RoomEntity {
                        position: new_position,
                    })
                    .id(),
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
    commands.insert_resource(MapPosition {
        pos_id: RoomId(0),
        will_move: None,
    });
    // Spawn a first danger zone
    if map_configuration.start_with_danger_zone {
        commands.spawn().insert(SpawnDangerZoneCommand {
            position: [20f32, 20f32].into(),
            radius_increase_per_second: map_configuration.speed_init_danger,
        });
    }
    commands.spawn().insert(Cooldown {
        last_action_time: time.seconds_since_startup() as f32,
        base_cooldown: 0.5f32,
    });
}

fn init_display_map(mut commands: Commands, shapes: Res<ShapeMeshes>, map: Res<MapDef>) {
    if map.rooms.is_empty() {
        return;
    }
    let mut visit_queue = vec![&RoomId(0)];
    let mut visit_index = RoomId(0);
    while visit_index.0 < visit_queue.len() {
        let room = &map.rooms[visit_queue[visit_index.0]];
        create_room(&shapes, &mut commands, room, visit_index, true);
        for connection in &room.connections {
            create_link(&mut commands, &map.rooms[connection], room);
            if !visit_queue.contains(&connection) {
                visit_queue.push(connection);
            }
        }
        visit_index.0 += 1;
    }
    let first_room = &map.rooms[&RoomId(0)];
    let mut charac_transform =
        Transform::from_xyz(first_room.position.0, first_room.position.1, 0.0);
    charac_transform.scale = Vec3::ONE * 22.0;

    let character = MeshBundle {
        mesh: shapes.quad2x2.clone(),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            shapes.pipeline_circle_gauge.clone(),
        )]),
        transform: charac_transform,
        ..Default::default()
    };
    commands
        .spawn_bundle(character)
        .insert(PlayerPositionDisplay)
        .insert(shapes.mat_circle_gauge.clone());
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
    shapes: Res<ShapeMeshes>,
    time: Res<Time>,
    mut timer: Local<f32>,
    map: Res<MapDef>,
    player_pos: Res<MapPosition>,
    mut rooms: Query<(Entity, &RoomId, &mut RoomGraphic)>,
) {
    *timer += time.delta_seconds();
    if *timer < 0.2f32 {
        return;
    }
    *timer = 0f32;
    let current_room = &map.rooms[&player_pos.pos_id];
    for (e, r, mut g) in rooms.iter_mut() {
        let room_to_update = &map.rooms[r];
        let is_reachable = current_room.connections.contains(r);
        if let Some(update_components) = g.updateReachability(&shapes, is_reachable, room_to_update)
        {
            commands
                .entity(e)
                .insert_bundle(update_components.0.mesh_bundle)
                .insert(update_components.0.material)
                .insert(update_components.1);
        }
    }
}

fn react_to_will_move(mut position_changed: ResMut<MapPosition>) {
    if !position_changed.is_changed() {
        return;
    }
    if position_changed.will_move.is_none() {
        return;
    }
    position_changed.pos_id = position_changed.will_move.unwrap();
    position_changed.will_move = None;
}

fn react_to_move_player(
    mut commands: Commands,
    mut coins: ResMut<Coins>,
    mut map: ResMut<MapDef>,
    mut danger_zone_grow_speedup: ResMut<DangerSpeedModifier>,
    map_configuration: Res<MapConfiguration>,
    position_changed: Res<MapPosition>,
    mut random: ResMut<RandomDeterministic>,
) {
    if !position_changed.is_changed() {
        return;
    }
    if position_changed.will_move.is_some() {
        return;
    }

    let rng = &mut random.random;
    commands.spawn().insert(MapCreateRoom {
        from_room_id: position_changed.pos_id,
        room_type: RoomType::Safe,
        battle: if rng.next_u32() % 5 == 0 {
            Some(Battle {
                hp: 1.0,
                attack: 1.0,
            })
        } else {
            None
        },
    });
    /*
    commands.spawn().insert(MapCreateRoom {
        from_room_id: position_changed.pos_id,
        room_type: RoomType::Danger,
        battle: None,
    });*/
    commands.spawn().insert(MapCreateRoom {
        from_room_id: position_changed.pos_id,
        room_type: RoomType::Coins,
        battle: if rng.next_u32() % 3 == 0 {
            Some(Battle {
                hp: 1.0,
                attack: 1.0,
            })
        } else {
            None
        },
    });
    commands.spawn().insert(MapCreateRoom {
        from_room_id: position_changed.pos_id,
        room_type: RoomType::Price(7),
        battle: None,
    });

    if let Some(r) = map.rooms.get(&position_changed.pos_id) {
        let current_position = [r.position.0, r.position.1].into();
        let direction_for_danger: Vec2 = {
            let mut direction = Vec2::ZERO;
            for i in 0..r.connections.len() {
                let mut direction_to_room: Vec2 = map
                    .rooms
                    .get(&r.connections[i])
                    .expect("map is incorrect")
                    .position
                    .into();
                direction_to_room -= current_position;
                direction_to_room = direction_to_room.normalize_or_zero();
                direction += direction_to_room;
            }
            direction = direction.normalize_or_zero();
            direction * 30f32
        };
        match r.room_type {
            RoomType::Safe => {}
            RoomType::Danger => {
                commands.spawn().insert(SpawnDangerZoneCommand {
                    position: direction_for_danger + current_position,
                    radius_increase_per_second: map_configuration.speed_init_danger,
                });
            }
            RoomType::Coins => {
                coins.amount += 1;
            }
            RoomType::Price(price) => {
                coins.amount = (coins.amount - price).max(0);
                danger_zone_grow_speedup.multiplier *= 0.5f32;
            }
        }
        if r.room_type != RoomType::Safe {
            // We visited this room so reset its type to Safe.
            if let Some(r) = map.rooms.get_mut(&position_changed.pos_id) {
                r.room_type = RoomType::Safe;
            }
        }
    }
}
fn cooldown_material_update(
    time: Res<Time>,
    shapes: ResMut<ShapeMeshes>,
    mut materials_circle_gauge: ResMut<Assets<CircleGaugeMaterial>>,
    q_cooldown: Query<(&Cooldown)>,
) {
    let cooldown = match q_cooldown.iter().last() {
        Some(c) => c,
        None => return,
    };
    if let Some(mat) = materials_circle_gauge.get_mut(shapes.mat_circle_gauge.clone()) {
        mat.ratio = cooldown.get_ratio(&time);
        if mat.ratio >= 1.0f32 {
            mat.color = Color::WHITE
        } else {
            mat.color = Color::GRAY;
        }
    }
}

fn handle_input(
    mut commands: Commands,
    map: Res<MapDef>,
    time: Res<Time>,
    coins: Res<Coins>,
    mut inputs: ResMut<UserInputs>,
    mut position: ResMut<MapPosition>,
    mut q_cooldown: Query<(&mut Cooldown)>,
) {
    let current_room = map.rooms.get(&position.pos_id).unwrap();
    let mut cooldown = match q_cooldown.iter_mut().last() {
        Some(c) => c,
        None => return,
    };
    for click in inputs.list.iter() {
        let UserInput::Click(click) = click;
        if !cooldown.is_ready(&time) {
            commands.spawn().insert(TextFeedbackSpawn {
                text: format!("Not Ready\n"),
                pos: Vec2::new(0f32, 0f32),
            });
            break;
        }
        for id in map.rooms.keys() {
            if !current_room.connections.contains(id) {
                continue;
            }
            let r = map.rooms.get(id).unwrap();

            let room_position = Vec2::new(r.position.0, r.position.1);
            let distance_to_room = room_position.distance(*click);
            if distance_to_room < 15.0 {
                match r.room_type {
                    RoomType::Danger => {}
                    RoomType::Safe => {}
                    RoomType::Coins => {}
                    RoomType::Price(price) => {
                        if coins.amount < price {
                            // TODO: spawn a feedback: not enough coins!
                            commands.spawn().insert(TextFeedbackSpawn {
                                text: format!("Not enough coins\n{}/{}", coins.amount, price),
                                pos: r.position.into(),
                            });
                            continue;
                        }
                    }
                };
                position.will_move = Some(*id);
                cooldown.last_action_time = time.seconds_since_startup() as f32;
                break;
            }
        }
    }
    inputs.list.clear();
}

fn create_new_rooms(
    mut commands: Commands,
    shapes: Res<ShapeMeshes>,
    mut random: ResMut<RandomDeterministic>,
    mut map: ResMut<MapDef>,
    q_create: Query<(Entity, &MapCreateRoom)>,
) {
    let min_distance_between_rooms = 40f32;
    for (e, create) in q_create.iter() {
        let rng = &mut random.random;
        let poisson = Poisson::new();
        let existing_points: Vec<(f32, f32)> = map.rooms.values().map(|r| r.position).collect();

        let room_id_to_create = RoomId(map.rooms.len());
        if let Some(ref_point) = map.rooms.get(&create.from_room_id).map(|f| f.position) {
            if let Some(new_position) = poisson.compute_new_position(
                &existing_points,
                &ref_point,
                min_distance_between_rooms,
                10,
                rng,
            ) {
                {
                    match map.rooms.entry(create.from_room_id) {
                        std::collections::hash_map::Entry::Occupied(mut room) => {
                            room.get_mut().connections.push(room_id_to_create);
                        }
                        std::collections::hash_map::Entry::Vacant(_) => return,
                    };
                }
                let mut new_room = Room {
                    connections: vec![create.from_room_id],
                    position: new_position,
                    entity: commands
                        .spawn()
                        .insert(RoomEntity {
                            position: new_position,
                        })
                        .id(),
                    room_type: create.room_type.clone(),
                };
                if let Some(battle) = &create.battle {
                    commands
                        .entity(new_room.entity)
                        .insert(battle.clone())
                        .insert(IsDirty);
                }
                create_room(&shapes, &mut commands, &new_room, room_id_to_create, true);
                create_link(
                    &mut commands,
                    map.rooms.get(&create.from_room_id).unwrap(),
                    &new_room,
                );

                let origin_position = map.rooms.get(&create.from_room_id).unwrap().position;
                let mut existing_point_without_origin = existing_points.clone();
                existing_point_without_origin.retain(|r| r != &origin_position);
                let (closest_point, dist_sqrd) =
                    find_closest(&existing_point_without_origin, &new_position);

                if dist_sqrd < (min_distance_between_rooms * 1.5f32).powi(2) {
                    let r1 = map
                        .rooms
                        .iter_mut()
                        .find(|r| r.1.position == (closest_point.x, closest_point.y))
                        .map(|room_from| {
                            room_from.1.connections.push(room_id_to_create);
                            room_from.0.clone()
                        });
                    if let Some(r1) = r1 {
                        new_room.connections.push(r1);
                        create_link(&mut commands, map.rooms.get(&r1).unwrap(), &new_room);
                    }
                }

                {
                    map.rooms.insert(room_id_to_create, new_room);
                }
            }
            commands.entity(e).despawn();
        }
    }
}

fn find_closest(existing_points: &[(f32, f32)], ref_point: &(f32, f32)) -> (Vec2, f32) {
    let mut closest: Vec2 = Vec2::new(0f32, 0f32);
    let mut distance = f32::MAX;
    let ref_point = Vec2::new(ref_point.0, ref_point.1);
    for p in existing_points.iter() {
        let new_closest = Vec2::new(p.0, p.1);
        let new_distance = new_closest.distance_squared(ref_point);
        if new_distance < distance {
            closest = new_closest;
            distance = new_distance;
        }
    }
    (closest, distance)
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
            outline_options: StrokeOptions::default().with_line_width(2.0),
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    );
    commands.spawn_bundle(character);
}
