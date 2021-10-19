use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes::Circle};

use crate::{map_graph::PlayerPositionDisplay, AppState};

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

pub fn SpawnDangerZone(mut commands: Commands, q: Query<(Entity, &SpawnDangerZoneCommand)>) {
    for (e, s) in q.iter() {
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
            Transform::from_xyz(s.position.x, s.position.y, 0.0),
        );
        commands
            .spawn()
            .insert_bundle(circle)
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

pub fn update_danger_visual(
    time: Res<Time>,
    mut timer: Local<f32>,
    mut commands: Commands,
    q: Query<(Entity, &Transform, &DangerZone)>,
) {
    *timer += time.delta_seconds();
    if *timer < 0.1f32 {
        return;
    }
    *timer = 0f32;
    for (e, t, d) in q.iter() {
        let color = *Color::RED.set_r(0.7f32);
        let circle = GeometryBuilder::build_as(
            &Circle {
                radius: d.size,
                center: Vec2::ZERO,
            },
            ShapeColors::outlined(color, color),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(2.0),
            },
            Transform::from_translation(t.translation),
        );
        commands.entity(e).insert_bundle(circle);
    }
}
