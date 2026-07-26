use bevy::prelude::Vec2;

use crate::{
    components::{BridgeFootprint, CombatRng},
    original::objects::BuildingType,
};

use crate::units::buildings::{BridgeVisualState, BridgeVisualTileSpec, bridge_pixel_bounds};

pub(crate) const LEFT_INDEX: usize = 8;
pub(crate) const SECOND_INDEX: usize = 9;
pub(crate) const PENULTIMATE_INDEX: usize = 10;
pub(crate) const RIGHT_INDEX: usize = 11;
pub(crate) const FILL_LIVE_INDEX: usize = 12;
pub(crate) const FILL_DAMAGED_0_INDEX: usize = 13;
pub(crate) const FILL_DAMAGED_1_INDEX: usize = 14;
pub(crate) const FILL_DESTROYED_INDEX: usize = 15;
pub(crate) const TILE_FRAME_SIZE: Vec2 = Vec2::new(16.0, 64.0);

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn pathing_dimensions(extra_links: u16) -> (u16, u16) {
    (5u16.saturating_add(extra_links), 4)
}

pub(crate) fn edge_block_offsets(extra_links: u16) -> Vec<(u16, u16)> {
    let (width, _) = pathing_dimensions(extra_links);
    let mut offsets = Vec::with_capacity(width as usize * 2);
    for tx in 0..width {
        offsets.push((tx, 0));
        offsets.push((tx, 3));
    }
    offsets
}

pub(crate) fn center_offsets(extra_links: u16) -> Vec<(u16, u16)> {
    let (width, _) = pathing_dimensions(extra_links);
    let mut offsets = Vec::with_capacity(width as usize * 2);
    for tx in 0..width {
        offsets.push((tx, 1));
        offsets.push((tx, 2));
    }
    offsets
}

pub(crate) fn crane_repair_entrances(top_left: Vec2, width: f32, _height: f32) -> [Vec2; 2] {
    [
        Vec2::new(top_left.x - 31.0, top_left.y + 31.0),
        Vec2::new(top_left.x + width + 32.0, top_left.y + 32.0),
    ]
}

pub(crate) fn visual_tile_specs(
    extra_links: u16,
    state: BridgeVisualState,
) -> Vec<BridgeVisualTileSpec> {
    let total_cols = 5 + extra_links as usize;
    (0..total_cols)
        .map(|col| {
            let index = match col {
                0 => LEFT_INDEX,
                1 => SECOND_INDEX,
                c if c == total_cols - 2 => PENULTIMATE_INDEX,
                c if c == total_cols - 1 => RIGHT_INDEX,
                _ => fill_index(state, col),
            };
            BridgeVisualTileSpec {
                index,
                world_offset: Vec2::new(col as f32 * 16.0, 0.0),
                frame_size: TILE_FRAME_SIZE,
            }
        })
        .collect()
}

pub(crate) fn fill_index(state: BridgeVisualState, col: usize) -> usize {
    match state {
        BridgeVisualState::Live => FILL_LIVE_INDEX,
        BridgeVisualState::Damaged if col % 2 == 0 => FILL_DAMAGED_0_INDEX,
        BridgeVisualState::Damaged => FILL_DAMAGED_1_INDEX,
        BridgeVisualState::Destroyed => FILL_DESTROYED_INDEX,
    }
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

    #[test]
    fn bridge_horizontal_visual_tiles_match_original_indices_and_offsets() {
        let specs = visual_tile_specs(2, BridgeVisualState::Damaged);

        assert_eq!(specs.len(), 7);
        assert_eq!(specs[0].index, LEFT_INDEX);
        assert_eq!(specs[1].index, SECOND_INDEX);
        assert_eq!(specs[2].index, FILL_DAMAGED_0_INDEX);
        assert_eq!(specs[3].index, FILL_DAMAGED_1_INDEX);
        assert_eq!(specs[5].index, PENULTIMATE_INDEX);
        assert_eq!(specs[6].index, RIGHT_INDEX);
        assert_eq!(specs[3].world_offset, Vec2::new(48.0, 0.0));
        assert_eq!(specs[3].frame_size, Vec2::new(16.0, 64.0));
        assert_eq!(fill_index(BridgeVisualState::Live, 3), FILL_LIVE_INDEX);
    }
}
