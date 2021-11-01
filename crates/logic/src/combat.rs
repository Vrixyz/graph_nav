use bevy::{prelude::*, render::pipeline::RenderPipeline};

use crate::{
    map_graph::{MapDef, MapPosition, RoomEntity},
    shapes::{self, ShapeMeshes},
    AppState,
};

pub struct CombatPlugin;

#[derive(Clone)]
pub struct Battle {
    pub hp: f32,
    pub attack: f32,
}
pub struct BattleGraphicRef {
    pub entity: Entity,
}

// TODO: prefer to use Changed<>
pub struct IsDirty;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let game_update_system_set = SystemSet::on_update(AppState::Game)
            .with_system(update_battle_room.system())
            .with_system(react_to_will_move.system());
        app.add_system_set(game_update_system_set);
    }
}

fn react_to_will_move(
    mut commands: Commands,
    mut map: ResMut<MapDef>,
    mut position_changed: ResMut<MapPosition>,
    mut q_b: Query<(&mut Battle)>,
) {
    // TODO: I guess we should move into the room and then go back rather than cancel the move...
    if !position_changed.is_changed() {
        return;
    }
    if position_changed.will_move.is_none() {
        return;
    }
    let room_entity = map.rooms[&position_changed.will_move.unwrap()].entity;
    let b = q_b.get_component_mut::<Battle>(room_entity);
    if let Ok(mut b) = b {
        if b.hp > 0f32 {
            // No update of pos_id, because battle is still here
            b.hp -= 1f32;
            position_changed.will_move = None;
            commands.entity(room_entity).insert(IsDirty);
            return;
        }
    }
    position_changed.pos_id = position_changed.will_move.unwrap();
    position_changed.will_move = None;
}

fn update_battle_room(
    mut commands: Commands,
    shapes: Res<ShapeMeshes>,
    q_r: Query<(Entity, &Battle, &RoomEntity), With<IsDirty>>,
    mut q_graphic: Query<&mut BattleGraphicRef>,
) {
    for (e, b, r) in q_r.iter() {
        let mut command_entity = commands.entity(e);
        command_entity.remove::<IsDirty>();
        if b.hp <= 0f32 {
            dbg!("will remove");
            match q_graphic.get_component_mut::<BattleGraphicRef>(dbg!(e)) {
                Ok(graphic) => {
                    dbg!("removing");
                    commands
                        .entity(graphic.entity)
                        .remove_bundle::<MeshBundle>()
                        .remove::<Handle<ColorMaterial>>();
                    commands.entity(e).remove::<BattleGraphicRef>();
                }
                Err(e) => {
                    dbg!(e);
                }
            }
            continue;
        }
        let graphic_entity = q_graphic
            .get_component_mut::<BattleGraphicRef>(e)
            .map_or_else(
                |b| {
                    dbg!(e);
                    let graphic_entity = dbg!(commands.spawn().id());
                    let graphic = BattleGraphicRef {
                        entity: graphic_entity,
                    };
                    commands.entity(e).insert(graphic);
                    graphic_entity
                },
                |b| dbg!(b.entity),
            );
        let bundle = create_health_bundle(&shapes, b, r.position);
        commands.entity(graphic_entity).insert_bundle(bundle);
        //        command_entity.insert(bundle.0).insert(bundle.1);
    }
}
fn create_health_bundle(
    shapes: &Res<ShapeMeshes>,
    battle: &Battle,
    position: (f32, f32),
) -> MeshBundle {
    let material = shapes.mat_orange.clone();

    dbg!(format!(
        "create battle info at {};{}",
        position.0, position.1
    ));
    // TODO: prefer to put information in child: health points, flavour sprite...
    let mut transform = Transform::from_xyz(position.0, position.1, 16.0);
    transform.scale = Vec3::ONE * 3.0;
    let mesh = MeshBundle {
        mesh: shapes.quad2x2.clone(),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            shapes.pipeline_circle.clone(),
        )]),
        transform,
        ..Default::default()
    };
    mesh
}
