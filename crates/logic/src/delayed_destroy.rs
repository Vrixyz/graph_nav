use bevy::prelude::*;

pub struct DelayedDestroy {
    pub(crate) time_to_destroy: f32,
}

pub(super) fn destroy_after(
    mut commands: Commands,
    time: Res<Time>,
    q_destroy: Query<(Entity, &DelayedDestroy)>,
) {
    for (e, d) in q_destroy.iter() {
        if d.time_to_destroy < time.time_since_startup().as_secs_f32() {
            commands.entity(e).despawn();
        }
    }
}
