use bevy::prelude::Vec2;

use crate::{
    components::CombatRng,
    original::types::TeamType,
};

pub(crate) const FLAME_PROJECTILE_FRAME_COUNT: usize = 4;
pub(crate) const FIRE_IMPACT_FRAME_COUNTS: [usize; 5] = [4, 4, 4, 6, 6];
pub(crate) const FIRE_IMPACT_FRAME_TIME: f32 = 0.06;
pub(crate) const FIRE_IMPACT_Z: f32 = 34.4;
pub(crate) const FIRE_FRAME_COUNT: usize = 3;
pub(crate) const FIRE_ROTATIONS: [u16; 8] = [0, 45, 90, 135, 180, 225, 270, 315];

const ATTACK_RESET_BASE_TIME: f32 = 0.07;
const ATTACK_FRAME_BASE_TIME: f32 = 0.05;
const ATTACK_JITTER_STEP: f32 = 0.0003;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FireImpactProfile {
    pub(crate) family_count: usize,
    pub(crate) frame_counts: [usize; 5],
    pub(crate) frame_time: f32,
    pub(crate) z: f32,
}

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(14.0)
}

pub(crate) fn special_projectile_frame_paths() -> Vec<String> {
    flame_projectile_frame_paths()
}

pub(crate) fn flame_projectile_frame_paths() -> Vec<String> {
    (0..FLAME_PROJECTILE_FRAME_COUNT)
        .map(|frame| format!("units/robots/pyro/bullet_n{frame:02}.png"))
        .collect()
}

pub(crate) fn fire_impact_profile() -> FireImpactProfile {
    FireImpactProfile {
        family_count: FIRE_IMPACT_FRAME_COUNTS.len(),
        frame_counts: FIRE_IMPACT_FRAME_COUNTS,
        frame_time: FIRE_IMPACT_FRAME_TIME,
        z: FIRE_IMPACT_Z,
    }
}

pub(crate) fn fire_impact_family_count() -> usize {
    FIRE_IMPACT_FRAME_COUNTS.len()
}

pub(crate) fn fire_impact_frame_count(family: usize) -> usize {
    FIRE_IMPACT_FRAME_COUNTS[family.min(FIRE_IMPACT_FRAME_COUNTS.len() - 1)]
}

pub(crate) fn fire_impact_frame_paths(family: usize) -> Vec<String> {
    let family = family.min(FIRE_IMPACT_FRAME_COUNTS.len() - 1);
    (0..fire_impact_frame_count(family))
        .map(|frame| format!("other/fire/fire{family}_n{frame:02}.png"))
        .collect()
}

pub(crate) fn fire_animation_frame_path(
    team: TeamType,
    direction: usize,
    frame: usize,
) -> Option<String> {
    if team == TeamType::Null {
        return None;
    }

    let team_name = team.atlas_team().asset_name();
    let rotation = FIRE_ROTATIONS[direction.min(FIRE_ROTATIONS.len() - 1)];
    let frame = frame.min(FIRE_FRAME_COUNT - 1);
    Some(format!(
        "units/robots/pyro/fire_{team_name}_r{rotation:03}_n{frame:02}.png"
    ))
}

pub(crate) fn fire_animation_frame_paths(team: TeamType, direction: usize) -> Option<Vec<String>> {
    (team != TeamType::Null).then(|| {
        (0..FIRE_FRAME_COUNT)
            .map(|frame| {
                fire_animation_frame_path(team, direction, frame)
                    .expect("non-null team has pyro fire frame")
            })
            .collect()
    })
}

pub(crate) fn fire_animation_next_delay(next_frame: usize, rng: &mut CombatRng) -> f32 {
    let jitter = rng.index(100) as f32 * ATTACK_JITTER_STEP;
    if next_frame % FIRE_FRAME_COUNT == 0 {
        ATTACK_RESET_BASE_TIME + jitter
    } else {
        ATTACK_FRAME_BASE_TIME + jitter
    }
}
