use bevy::{prelude::*, render::pipeline::RenderPipeline};

use crate::{
    map_graph::RoomEntity,
    shapes::{self, ShapeMeshes},
};

pub struct CombatPlugin;

pub struct Battle {
    pub hp: f32,
    pub attack: f32,
}
pub struct IsDirty;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(update_battle_room.system());
    }
}

// TODO: react to WillMove -> decrease hp of tile then move if battle.hp = 0

fn update_battle_room(
    mut commands: Commands,
    shapes: Res<ShapeMeshes>,
    q_r: Query<(Entity, &Battle, &RoomEntity), With<IsDirty>>,
) {
    for (e, b, r) in q_r.iter() {
        let mut command_entity = commands.entity(e);
        command_entity.remove::<IsDirty>();
        if b.hp <= 0f32 {
            command_entity
                .remove::<MeshBundle>()
                .remove::<Handle<ColorMaterial>>();
            continue;
        }
        let bundle = create_health_bundle(&shapes, b, r.position);
        command_entity.insert(bundle.0).insert(bundle.1);
    }
}
fn create_health_bundle(
    shapes: &Res<ShapeMeshes>,
    battle: &Battle,
    position: (f32, f32),
) -> (MeshBundle, Handle<shapes::ColorMaterial>) {
    let material = shapes.mat_orange.clone();

    // TODO: prefer to information in child: health points, flavour sprite...
    let mut transform = Transform::from_xyz(position.0, position.1, 20.0);
    transform.scale = Vec3::ONE * 5.0;
    let mesh = MeshBundle {
        mesh: shapes.quad2x2.clone(),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            shapes.pipeline_triangle.clone(),
        )]),
        transform,
        ..Default::default()
    };
    (mesh, material)
}
