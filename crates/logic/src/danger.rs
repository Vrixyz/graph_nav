use bevy::{prelude::*, render::pipeline::RenderPipeline};
use bevy_prototype_lyon::{prelude::*, shapes::Circle};

use crate::{map_graph::PlayerPositionDisplay, shapes::ShapeMeshes, AppState};

pub struct DangerZone {
    pub size: f32,
}

pub struct GrowDangerZone {
    pub radius_increase_per_second: f32,
}
pub struct DangerSpeedModifier {
    pub multiplier: f32,
}

pub struct SpawnDangerZoneCommand {
    pub position: Vec2,
    pub radius_increase_per_second: f32,
}

pub fn SpawnDangerZone(
    mut commands: Commands,
    shapes: Res<ShapeMeshes>,
    q: Query<(Entity, &SpawnDangerZoneCommand)>,
) {
    for (e, s) in q.iter() {
        let mesh = MeshBundle {
            mesh: shapes.quad2x2.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                shapes.pipeline_circle.clone(),
            )]),
            transform: Transform::from_xyz(s.position.x, s.position.y, 15.0),
            ..Default::default()
        };
        commands
            .spawn()
            .insert_bundle(mesh)
            .insert(DangerZone { size: 1f32 })
            .insert(GrowDangerZone {
                radius_increase_per_second: s.radius_increase_per_second,
            });
        commands.entity(e).despawn();
    }
}

pub fn check_player_death(
    mut state: ResMut<State<AppState>>,
    position: Query<&Transform, With<PlayerPositionDisplay>>,
    dangers: Query<(&Transform, &DangerZone)>,
) {
    for player_transform in position.iter() {
        for (danger_transform, danger) in dangers.iter() {
            let distance = danger_transform
                .translation
                .distance(player_transform.translation);
            if distance < danger.size {
                state.set(AppState::Menu);
            }
        }
    }
}

pub fn danger_zone_grow_speedup(
    time: Res<Time>,
    mut danger_speed_modifier: ResMut<DangerSpeedModifier>,
) {
    danger_speed_modifier.multiplier += time.delta_seconds() * 0.1f32;
}

pub fn grow_danger_zone(
    time: Res<Time>,
    danger_speed_modifier: Res<DangerSpeedModifier>,
    mut dangers: Query<(&GrowDangerZone, &mut DangerZone)>,
) {
    for (grow, mut d) in dangers.iter_mut() {
        d.size += time.delta_seconds()
            * grow.radius_increase_per_second
            * danger_speed_modifier.multiplier;
    }
}

pub fn update_danger_visual(mut q: Query<(Entity, &mut Transform, &DangerZone)>) {
    for (e, mut t, d) in q.iter_mut() {
        t.scale = Vec3::ONE * d.size;
    }
}
