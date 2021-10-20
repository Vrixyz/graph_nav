use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::delayed_destroy::DelayedDestroy;

pub struct TextFeedbackSpawn {
    pub text: String,
    pub pos: Vec2,
}
pub struct TextFeedback {
    pub text: String,
}

pub fn spawn_text_feedback(
    mut commands: Commands,
    time: Res<Time>,
    q: Query<(Entity, &TextFeedbackSpawn)>,
) {
    for (e, s) in q.iter() {
        commands
            .spawn()
            .insert(TextFeedback {
                text: s.text.clone(),
            })
            .insert(Transform::from_translation(s.pos.extend(0f32)))
            .insert(DelayedDestroy {
                time_to_destroy: time.time_since_startup().as_secs_f32() + 0.5f32,
            });
        commands.entity(e).despawn();
    }
}

pub fn show_text_feedback(
    window: Res<Windows>,
    egui_context: ResMut<EguiContext>,
    q: Query<(&Transform, &TextFeedback)>,
) {
    for (t, f) in q.iter() {
        let win = window.get_primary().expect("no primary window");
        if let Some(pos) = win.cursor_position() {
            egui::Window::new("Hello")
                .fixed_size((150f32, 50f32))
                .title_bar(false)
                .fixed_pos((pos.x - 75f32, win.physical_height() as f32 - pos.y - 55f32))
                .show(egui_context.ctx(), |ui| {
                    ui.label(&f.text);
                });
        }
    }
}
