use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes::Circle};

use crate::map_graph::PlayerPositionDisplay;

pub struct DangerZone {
    pub size: f32,
}

pub struct GrowDangerZone {
    pub radius_increase_per_second: f32,
}

pub fn check_player_death(
    position: Query<&Transform, With<PlayerPositionDisplay>>,
    dangers: Query<(&Transform, &DangerZone)>,
) {
    for player_transform in position.iter() {
        for (danger_transform, danger) in dangers.iter() {
            let distance = danger_transform
                .translation
                .distance(player_transform.translation);
            if distance < danger.size {
                dbg!("die");
            }
        }
    }
}

pub fn grow_danger_zone(time: Res<Time>, mut dangers: Query<(&GrowDangerZone, &mut DangerZone)>) {
    for (grow, mut d) in dangers.iter_mut() {
        d.size += time.delta_seconds() * grow.radius_increase_per_second;
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
        let circle = GeometryBuilder::build_as(
            &Circle {
                radius: d.size,
                center: Vec2::ZERO,
            },
            ShapeColors::outlined(Color::NONE, Color::RED),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(2.0),
            },
            Transform::from_translation(t.translation),
        );
        commands.entity(e).insert_bundle(circle);
    }
}
