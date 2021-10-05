use bevy::math::Vec3;

pub fn move_towards(current: Vec3, target: Vec3, max_distance_delta: f32) -> Vec3 {
    let to_vector = target - current;

    let sqdist = target.distance_squared(current);

    if sqdist == 0.0 || (max_distance_delta >= 0.0 && sqdist <= max_distance_delta.powf(2.0)) {
        return target;
    }
    let dist = sqdist.sqrt();
    current + to_vector / dist * max_distance_delta
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn move_towards_done() {
        let dest = Vec3::new(1.0, 0.0, 0.0);
        assert_eq!(move_towards(Vec3::new(0.0, 0.0, 0.0), dest, 1.0), dest);
    }
    #[test]
    fn move_towards_not_done() {
        let target = Vec3::new(3.0, 0.0, 0.0);
        let actual_dest = Vec3::new(2.0, 0.0, 0.0);
        assert_eq!(
            move_towards(Vec3::new(0.0, 0.0, 0.0), target, 2.0),
            actual_dest
        );
    }
}
