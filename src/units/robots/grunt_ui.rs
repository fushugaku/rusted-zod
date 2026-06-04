use bevy::prelude::Vec2;

use crate::{
    components::CombatRng,
    original::types::TeamType,
};

pub(crate) const NORMAL_DEATH_FRAME_COUNTS: [usize; 4] = [10, 10, 10, 8];
pub(crate) const MELT_DEATH_FRAME_COUNT: usize = 17;
pub(crate) const TURRENT_DEATH_FRAME_COUNT: usize = 33;
pub(crate) const TURRENT_DEATH_AIR_FRAME_COUNT: usize = 8;
pub(crate) const TURRENT_DEATH_LAND_START_FRAME: usize = TURRENT_DEATH_AIR_FRAME_COUNT;
pub(crate) const TURRENT_DEATH_LAST_FRAME: usize = TURRENT_DEATH_FRAME_COUNT - 1;
pub(crate) const DEATH_Z: f32 = 34.0;
pub(crate) const DEATH_FRAME_TIME: f32 = 0.16;
pub(crate) const FLIP_AIR_FRAME_TIME: f32 = 0.05;
pub(crate) const FLIP_LAND_FRAME_TIME: f32 = 0.15;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TurrentDeathTrajectory {
    pub(crate) start_map: Vec2,
    pub(crate) end_map: Vec2,
    pub(crate) final_time: f32,
    pub(crate) rise: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RobotDeathAnimationProfile {
    pub(crate) death_frame_time: f32,
    pub(crate) death_z: f32,
    pub(crate) normal_frame_counts: [usize; 4],
    pub(crate) melt_frame_count: usize,
    pub(crate) turrent_frame_count: usize,
    pub(crate) turrent_air_frame_count: usize,
    pub(crate) turrent_land_start_frame: usize,
    pub(crate) turrent_last_frame: usize,
    pub(crate) flip_air_frame_time: f32,
    pub(crate) flip_land_frame_time: f32,
}

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(14.0)
}

pub(crate) fn death_animation_profile() -> RobotDeathAnimationProfile {
    RobotDeathAnimationProfile {
        death_frame_time: DEATH_FRAME_TIME,
        death_z: DEATH_Z,
        normal_frame_counts: NORMAL_DEATH_FRAME_COUNTS,
        melt_frame_count: MELT_DEATH_FRAME_COUNT,
        turrent_frame_count: TURRENT_DEATH_FRAME_COUNT,
        turrent_air_frame_count: TURRENT_DEATH_AIR_FRAME_COUNT,
        turrent_land_start_frame: TURRENT_DEATH_LAND_START_FRAME,
        turrent_last_frame: TURRENT_DEATH_LAST_FRAME,
        flip_air_frame_time: FLIP_AIR_FRAME_TIME,
        flip_land_frame_time: FLIP_LAND_FRAME_TIME,
    }
}

pub(crate) fn normal_death_frame_paths(team: TeamType, variant: usize) -> Option<Vec<String>> {
    if team == TeamType::Null {
        return None;
    }
    let variant = variant.min(3);
    let team_name = team.atlas_team().asset_name();
    Some(
        (0..normal_death_frame_count(variant))
            .map(|frame| {
                format!(
                    "units/robots/die{}_{}_n{frame:02}.png",
                    variant + 1,
                    team_name
                )
            })
            .collect(),
    )
}

pub(crate) fn melt_death_frame_paths(team: TeamType) -> Option<Vec<String>> {
    if team == TeamType::Null {
        return None;
    }
    let team_name = team.atlas_team().asset_name();
    Some(
        (0..MELT_DEATH_FRAME_COUNT)
            .map(|frame| format!("units/robots/melt_{team_name}_n{frame:02}.png"))
            .collect(),
    )
}

pub(crate) fn turrent_death_frame_paths(team: TeamType) -> Option<Vec<String>> {
    if team == TeamType::Null {
        return None;
    }
    let team_name = team.atlas_team().asset_name();
    Some(
        (0..TURRENT_DEATH_FRAME_COUNT)
            .map(|frame| format!("units/robots/die5_{team_name}_n{frame:02}.png"))
            .collect(),
    )
}

pub(crate) fn normal_death_frame_count(variant: usize) -> usize {
    NORMAL_DEATH_FRAME_COUNTS[variant.min(NORMAL_DEATH_FRAME_COUNTS.len() - 1)]
}

pub(crate) fn death_top_left_world(center: Vec2) -> Vec2 {
    Vec2::new(center.x - 8.0, center.y + 8.0)
}

pub(crate) fn turrent_death_trajectory(
    center_map: Vec2,
    rng: &mut CombatRng,
) -> TurrentDeathTrajectory {
    let start_map = center_map + Vec2::new(2.0 - rng.index(5) as f32, 2.0 - rng.index(5) as f32);
    TurrentDeathTrajectory {
        start_map,
        end_map: start_map
            + Vec2::new(100.0 - rng.index(200) as f32, 100.0 - rng.index(200) as f32),
        final_time: 3.5 + rng.index(10) as f32 * 0.1,
        rise: 1.3 + rng.index(200) as f32 * 0.01,
    }
}

pub(crate) fn turrent_death_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    -(rise / final_time) * (t * t) + rise * t + 1.0
}
