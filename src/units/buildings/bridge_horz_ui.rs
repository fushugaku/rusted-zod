use bevy::prelude::Vec2;

use crate::{
    components::{BridgeFootprint, CombatRng},
    original::objects::BuildingType,
};

use super::bridge_pixel_bounds;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn turrent_spawn_points(bridge: BridgeFootprint, rng: &mut CombatRng) -> Vec<Vec2> {
    if bridge.building != BuildingType::BridgeHorz {
        return Vec::new();
    }

    let Some((top_left, width, _height)) = bridge_pixel_bounds(bridge) else {
        return Vec::new();
    };
    let mut points = Vec::new();
    let mut x = top_left.x + 16.0 + 5.0 + rng.index(10) as f32;
    let y = top_left.y + 16.0;
    let max_x = top_left.x + width - 16.0;

    while x < max_x {
        points.push(Vec2::new(x, y + rng.index(32) as f32));
        x += 5.0 + rng.index(10) as f32;
    }

    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_horizontal_turrent_spawn_points_follow_original_axis_stride() {
        let mut rng = CombatRng::default();
        let bridge = BridgeFootprint {
            x: 2,
            y: 3,
            building: BuildingType::BridgeHorz,
            extra_links: 2,
        };
        let points = turrent_spawn_points(bridge, &mut rng);

        assert!(!points.is_empty());
        for point in points {
            assert!((53.0..128.0).contains(&point.x));
            assert!((64.0..96.0).contains(&point.y));
        }
    }
}
