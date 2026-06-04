use bevy::prelude::Vec2;

use crate::{
    components::CombatRng,
    original::{
        objects::{ItemType, ObjectKind},
        settings::{MAP_ITEM_TURRENT_DELAY, MAP_ITEM_TURRENT_DELAY_RANDOM},
    },
};

pub(crate) const TURRENT_MAX_DISTANCE: f32 = 300.0;
const TURRENT_MAX_INDEX: u8 = 21;
pub(crate) const UNIT_PARTICLE_FRAME_TIME: f32 = 0.03;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EffectTrajectory {
    pub(crate) start: Vec2,
    pub(crate) end: Vec2,
    pub(crate) final_time: f32,
    pub(crate) rise: f32,
}

pub(crate) fn asset_path(object_i: u8) -> String {
    format!(
        "other/map_items/map_object{}.png",
        object_i.min(TURRENT_MAX_INDEX)
    )
}

pub(crate) fn turrent_frame_paths(object_i: u8) -> Vec<String> {
    vec![format!(
        "other/map_items/no_shadow{}.png",
        object_i.min(TURRENT_MAX_INDEX)
    )]
}

pub(crate) fn turrent_object_index(kind: ObjectKind) -> Option<u8> {
    match kind {
        ObjectKind::MapItem(id) if id >= ItemType::MapObjectStart as u8 => {
            Some((id - ItemType::MapObjectStart as u8).min(TURRENT_MAX_INDEX))
        }
        _ => None,
    }
}

pub(crate) fn turrent_visual_offset(object_i: u8) -> Vec2 {
    Vec2::new(0.0, turrent_image_height(object_i) - 16.0)
}

pub(crate) fn turrent_image_height(object_i: u8) -> f32 {
    match object_i.min(TURRENT_MAX_INDEX) {
        5 | 10 => 16.0,
        _ => 32.0,
    }
}

pub(crate) fn turrent_start(top_left: Vec2, rng: &mut CombatRng) -> Vec2 {
    top_left + Vec2::new(5.0 - rng.index(10) as f32, 5.0 - rng.index(10) as f32)
}

pub(crate) fn turrent_target(top_left: Vec2, rng: &mut CombatRng) -> Vec2 {
    top_left
        + Vec2::splat(16.0)
        + Vec2::new(
            TURRENT_MAX_DISTANCE - rng.index((TURRENT_MAX_DISTANCE * 2.0) as usize) as f32,
            TURRENT_MAX_DISTANCE - rng.index((TURRENT_MAX_DISTANCE * 2.0) as usize) as f32,
        )
}

pub(crate) fn turrent_delay(rng: &mut CombatRng) -> f32 {
    MAP_ITEM_TURRENT_DELAY + rng.index(MAP_ITEM_TURRENT_DELAY_RANDOM) as f32 * 0.01
}

pub(crate) fn turrent_rise(rng: &mut CombatRng) -> f32 {
    0.5 + rng.index(100) as f32 * 0.01
}

pub(crate) fn turrent_spin_degrees_per_sec(rng: &mut CombatRng) -> f32 {
    let mut angle_degrees_per_sec = 240.0 - rng.index(480) as f32;
    if angle_degrees_per_sec >= 0.0 {
        angle_degrees_per_sec += 100.0;
    } else {
        angle_degrees_per_sec -= 100.0;
    }
    angle_degrees_per_sec
}

pub(crate) fn turrent_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    if final_time <= 0.0 {
        0.0
    } else {
        -(rise / final_time) * (t * t) + rise * t
    }
}

pub(crate) fn unit_particle_frame_paths() -> Vec<String> {
    (0..20)
        .map(|frame| format!("other/particles/unit_particle_n{frame:02}.png"))
        .collect()
}

pub(crate) fn death_unit_particle_count(rng: &mut CombatRng) -> usize {
    10 + rng.index(8)
}

pub(crate) fn death_spark_count(rng: &mut CombatRng) -> usize {
    20 + rng.index(20)
}

pub(crate) fn death_unit_particle_trajectory(
    anchor_map: Vec2,
    max_horz: f32,
    max_vert: f32,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    particle_trajectory(anchor_map, max_horz, max_vert, 1.4, 10, 1.1, 30, rng)
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

    #[test]
    fn asset_paths_match_original_map_object_names() {
        assert_eq!(asset_path(0), "other/map_items/map_object0.png");
        assert_eq!(asset_path(99), "other/map_items/map_object21.png");
        assert_eq!(
            turrent_frame_paths(12),
            vec!["other/map_items/no_shadow12.png"]
        );
        assert_eq!(
            turrent_frame_paths(99),
            vec!["other/map_items/no_shadow21.png"]
        );

        let unit_frames = unit_particle_frame_paths();
        assert_eq!(unit_frames.len(), 20);
        assert_eq!(unit_frames[0], "other/particles/unit_particle_n00.png");
        assert_eq!(unit_frames[19], "other/particles/unit_particle_n19.png");
        assert_eq!(UNIT_PARTICLE_FRAME_TIME, 0.03);
    }

    #[test]
    fn turrent_profile_matches_original_ranges() {
        assert_eq!(
            turrent_object_index(ObjectKind::MapItem(ItemType::MapObjectStart as u8)),
            Some(0)
        );
        assert_eq!(
            turrent_object_index(ObjectKind::MapItem(ItemType::MapObjectStart as u8 + 21)),
            Some(21)
        );
        assert_eq!(
            turrent_object_index(ObjectKind::MapItem(ItemType::MapObjectStart as u8 + 99)),
            Some(21)
        );
        assert_eq!(
            turrent_object_index(ObjectKind::MapItem(ItemType::Hut as u8)),
            None
        );
        assert_eq!(turrent_image_height(0), 32.0);
        assert_eq!(turrent_image_height(5), 16.0);
        assert_eq!(turrent_image_height(10), 16.0);
        assert_eq!(turrent_image_height(99), 32.0);
        assert_eq!(turrent_visual_offset(0), Vec2::new(0.0, 16.0));
        assert_eq!(turrent_visual_offset(5), Vec2::ZERO);

        let mut rng = CombatRng::default();
        let top_left = Vec2::new(100.0, 50.0);
        for _ in 0..32 {
            let start = turrent_start(top_left, &mut rng);
            assert!(start.x >= top_left.x - 4.0 && start.x <= top_left.x + 5.0);
            assert!(start.y >= top_left.y - 4.0 && start.y <= top_left.y + 5.0);

            let target = turrent_target(top_left, &mut rng);
            let offset = target - (top_left + Vec2::splat(16.0));
            assert!(offset.x > -TURRENT_MAX_DISTANCE && offset.x <= TURRENT_MAX_DISTANCE);
            assert!(offset.y > -TURRENT_MAX_DISTANCE && offset.y <= TURRENT_MAX_DISTANCE);
            assert!((3.0..=3.99).contains(&turrent_delay(&mut rng)));
            assert!((0.5..=1.49).contains(&turrent_rise(&mut rng)));
            let spin = turrent_spin_degrees_per_sec(&mut rng);
            assert!((-339.0..=-100.0).contains(&spin) || (100.0..=340.0).contains(&spin));
        }

        assert_eq!(turrent_arc_size(1.0, 3.0, 0.0), 0.0);
        assert!(turrent_arc_size(1.0, 3.0, 1.0) > 0.0);
    }

    #[test]
    fn death_particles_match_original_ranges() {
        let mut rng = CombatRng::default();
        for _ in 0..32 {
            assert!((10..=17).contains(&death_unit_particle_count(&mut rng)));
            assert!((20..=39).contains(&death_spark_count(&mut rng)));
        }

        let trajectory = death_unit_particle_trajectory(Vec2::ZERO, 65.0, 55.0, &mut rng);
        assert!((1.4..=2.3).contains(&trajectory.final_time));
        assert!((1.1..=1.39).contains(&trajectory.rise));
        assert!(trajectory.start.distance(trajectory.end) > 0.0);
    }
}
