use bevy::prelude::Vec2;

use crate::{
    components::{BridgeFootprint, CombatRng},
    original::objects::BuildingType,
    original::types::PlanetType,
};

use crate::units::buildings::{
    BRIDGE_TURRENT_MAX_DISTANCE, BridgeVisualState, BridgeVisualTileSpec, EffectTrajectory,
    bridge_pixel_bounds, building_ui::planet_asset_name,
};

pub(crate) const TOP_INDEX: usize = 0;
pub(crate) const SECOND_INDEX: usize = 1;
pub(crate) const PENULTIMATE_INDEX: usize = 2;
pub(crate) const BOTTOM_INDEX: usize = 3;
pub(crate) const FILL_LIVE_INDEX: usize = 4;
pub(crate) const FILL_DAMAGED_0_INDEX: usize = 5;
pub(crate) const FILL_DAMAGED_1_INDEX: usize = 6;
pub(crate) const FILL_DESTROYED_INDEX: usize = 7;
pub(crate) const TILE_FRAME_SIZE: Vec2 = Vec2::new(64.0, 16.0);

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn pathing_dimensions(extra_links: u16) -> (u16, u16) {
    (4, 5u16.saturating_add(extra_links))
}

pub(crate) fn edge_block_offsets(extra_links: u16) -> Vec<(u16, u16)> {
    let (_, height) = pathing_dimensions(extra_links);
    let mut offsets = Vec::with_capacity(height as usize * 2);
    for ty in 0..height {
        offsets.push((0, ty));
        offsets.push((3, ty));
    }
    offsets
}

pub(crate) fn center_offsets(extra_links: u16) -> Vec<(u16, u16)> {
    let (_, height) = pathing_dimensions(extra_links);
    let mut offsets = Vec::with_capacity(height as usize * 2);
    for ty in 0..height {
        offsets.push((1, ty));
        offsets.push((2, ty));
    }
    offsets
}

pub(crate) fn crane_repair_entrances(top_left: Vec2, _width: f32, height: f32) -> [Vec2; 2] {
    [
        Vec2::new(top_left.x + 32.0, top_left.y - 32.0),
        Vec2::new(top_left.x + 32.0, top_left.y + height + 32.0),
    ]
}

pub(crate) fn visual_tile_specs(
    extra_links: u16,
    state: BridgeVisualState,
) -> Vec<BridgeVisualTileSpec> {
    let total_rows = 5 + extra_links as usize;
    (0..total_rows)
        .map(|row| {
            let index = match row {
                0 => TOP_INDEX,
                1 => SECOND_INDEX,
                r if r == total_rows - 2 => PENULTIMATE_INDEX,
                r if r == total_rows - 1 => BOTTOM_INDEX,
                _ => fill_index(state, row),
            };
            BridgeVisualTileSpec {
                index,
                world_offset: Vec2::new(0.0, row as f32 * 16.0),
                frame_size: TILE_FRAME_SIZE,
            }
        })
        .collect()
}

pub(crate) fn fill_index(state: BridgeVisualState, row: usize) -> usize {
    match state {
        BridgeVisualState::Live => FILL_LIVE_INDEX,
        BridgeVisualState::Damaged if row % 2 == 0 => FILL_DAMAGED_0_INDEX,
        BridgeVisualState::Damaged => FILL_DAMAGED_1_INDEX,
        BridgeVisualState::Destroyed => FILL_DESTROYED_INDEX,
    }
}

pub(crate) fn turrent_spawn_points(bridge: BridgeFootprint, rng: &mut CombatRng) -> Vec<Vec2> {
    if bridge.building != BuildingType::BridgeVert {
        return Vec::new();
    }

    let Some((top_left, _width, height)) = bridge_pixel_bounds(bridge) else {
        return Vec::new();
    };
    let mut points = Vec::new();
    let x = top_left.x + 16.0;
    let mut y = top_left.y + 16.0 + 5.0 + rng.index(10) as f32;
    let max_y = top_left.y + height - 16.0;

    while y < max_y {
        points.push(Vec2::new(x + rng.index(32) as f32, y));
        y += 5.0 + rng.index(10) as f32;
    }

    points
}

pub(crate) fn turrent_frame_paths(planet: PlanetType) -> Vec<String> {
    let planet = planet_asset_name(planet);
    (0..12)
        .map(|frame| format!("planets/bridge_effects/debri_large_{planet}_n{frame:02}.png"))
        .collect()
}

pub(crate) fn rock_particle_frame_paths(planet: PlanetType) -> Vec<String> {
    let planet = planet_asset_name(planet);
    (0..16)
        .map(|frame| format!("planets/rock_effects/debri_small_{planet}_n{frame:02}.png"))
        .collect()
}

pub(crate) fn turrent_trajectory(
    anchor_map: Vec2,
    reversed: bool,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    let particle = anchor_map + Vec2::new(rng.index(8) as f32, rng.index(24) as f32);
    let start = particle - Vec2::new(8.0, 5.0);
    let end = particle
        + Vec2::new(
            BRIDGE_TURRENT_MAX_DISTANCE
                - rng.index((BRIDGE_TURRENT_MAX_DISTANCE * 2.0) as usize) as f32,
            BRIDGE_TURRENT_MAX_DISTANCE
                - rng.index((BRIDGE_TURRENT_MAX_DISTANCE * 2.0) as usize) as f32,
        );
    let (start, end) = if reversed { (end, start) } else { (start, end) };

    EffectTrajectory {
        start,
        end,
        final_time: 1.5 + rng.index(10) as f32 * 0.1,
        rise: turrent_rise(rng),
    }
}

pub(crate) fn turrent_rise(rng: &mut CombatRng) -> f32 {
    (1.1 + rng.index(200) as f32 * 0.01).trunc()
}

pub(crate) fn rock_particle_trajectory(anchor_map: Vec2, rng: &mut CombatRng) -> EffectTrajectory {
    particle_trajectory(anchor_map, 80.0, 60.0, 1.1, 10, 1.1, 30, rng)
}

pub(crate) fn end_particle_count(rng: &mut CombatRng) -> usize {
    12 + rng.index(6)
}

fn particle_trajectory(
    anchor_map: Vec2,
    max_horz: f32,
    max_vert: f32,
    lifetime_base: f32,
    lifetime_random: usize,
    rise_base: f32,
    rise_random: usize,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    let particle = anchor_map + Vec2::new(rng.index(8) as f32, rng.index(24) as f32);
    let start = particle - Vec2::new(8.0, 5.0);
    let end = particle
        + Vec2::new(
            max_horz - rng.index((max_horz * 2.0) as usize) as f32,
            max_vert - rng.index((max_vert * 2.0) as usize) as f32,
        );

    EffectTrajectory {
        start,
        end,
        final_time: lifetime_base + rng.index(lifetime_random) as f32 * 0.1,
        rise: rise_base + rng.index(rise_random) as f32 * 0.01,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::BuildingType;

    #[test]
    fn bridge_effect_assets_match_original_paths() {
        let bridge_frames = turrent_frame_paths(PlanetType::Desert);
        assert_eq!(bridge_frames.len(), 12);
        assert_eq!(
            bridge_frames.first().unwrap(),
            "planets/bridge_effects/debri_large_desert_n00.png"
        );
        assert_eq!(
            bridge_frames.last().unwrap(),
            "planets/bridge_effects/debri_large_desert_n11.png"
        );

        let rock_frames = rock_particle_frame_paths(PlanetType::Arctic);
        assert_eq!(rock_frames.len(), 16);
        assert_eq!(
            rock_frames.last().unwrap(),
            "planets/rock_effects/debri_small_arctic_n15.png"
        );
    }

    #[test]
    fn bridge_vertical_turrent_spawn_points_follow_original_axis_stride() {
        let mut rng = CombatRng::default();
        let bridge = BridgeFootprint {
            x: 2,
            y: 3,
            building: BuildingType::BridgeVert,
            extra_links: 2,
        };
        let points = turrent_spawn_points(bridge, &mut rng);

        assert!(!points.is_empty());
        for point in points {
            assert!((48.0..80.0).contains(&point.x));
            assert!((69.0..144.0).contains(&point.y));
        }
    }

    #[test]
    fn bridge_vertical_visual_tiles_match_original_indices_and_offsets() {
        let specs = visual_tile_specs(2, BridgeVisualState::Damaged);

        assert_eq!(specs.len(), 7);
        assert_eq!(specs[0].index, TOP_INDEX);
        assert_eq!(specs[1].index, SECOND_INDEX);
        assert_eq!(specs[2].index, FILL_DAMAGED_0_INDEX);
        assert_eq!(specs[3].index, FILL_DAMAGED_1_INDEX);
        assert_eq!(specs[5].index, PENULTIMATE_INDEX);
        assert_eq!(specs[6].index, BOTTOM_INDEX);
        assert_eq!(specs[3].world_offset, Vec2::new(0.0, 48.0));
        assert_eq!(specs[3].frame_size, Vec2::new(64.0, 16.0));
        assert_eq!(
            fill_index(BridgeVisualState::Destroyed, 3),
            FILL_DESTROYED_INDEX
        );
    }

    #[test]
    fn bridge_turrent_and_rock_particle_timing_matches_original_ranges() {
        let mut rng = CombatRng::default();
        for _ in 0..32 {
            let turrent = turrent_trajectory(Vec2::new(100.0, 50.0), false, &mut rng);
            assert!((1.5..=2.4).contains(&turrent.final_time));
            assert!([1.0, 2.0, 3.0].contains(&turrent.rise));
            assert!(turrent.start.distance(turrent.end) > 0.0);

            let rock = rock_particle_trajectory(Vec2::new(100.0, 50.0), &mut rng);
            assert!((1.1..=2.0).contains(&rock.final_time));
            assert!((1.1..=1.39).contains(&rock.rise));

            let count = end_particle_count(&mut rng);
            assert!((12..=17).contains(&count));
        }

        assert_eq!(
            crate::units::buildings::bridge_turrent_arc_size(1.1, 1.5, 0.0),
            1.0
        );
        assert!(crate::units::buildings::bridge_turrent_arc_size(1.1, 1.5, 0.75) > 1.0);
        assert_eq!(
            crate::units::buildings::bridge_rock_particle_arc_size(1.1, 1.1, 1.1),
            1.0
        );
        assert_eq!(crate::units::buildings::BRIDGE_TURRENT_FRAME_TIME, 0.07);
        assert_eq!(
            crate::units::buildings::BRIDGE_ROCK_PARTICLE_FRAME_TIME,
            0.07
        );
        assert_eq!(crate::units::buildings::BRIDGE_REVIVE_RERENDER_DELAY, 2.25);
    }
}
