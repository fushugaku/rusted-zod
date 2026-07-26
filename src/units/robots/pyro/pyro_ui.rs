use bevy::prelude::Vec2;

use crate::components::CombatRng;
use crate::original::types::TeamType;

use crate::units::robots::SpecialProjectileKind;

pub(crate) const FLAME_PROJECTILE_SPEED: f32 = 300.0;
pub(crate) const FLAME_PROJECTILE_FRAME_COUNT: usize = 4;
pub(crate) const FIRE_IMPACT_FRAME_COUNTS: [usize; 5] = [4, 4, 4, 6, 6];
pub(crate) const FIRE_IMPACT_FRAME_TIME: f32 = 0.06;
pub(crate) const FIRE_IMPACT_Z: f32 = 34.4;
pub(crate) const FIRE_FRAME_COUNT: usize = 3;
pub(crate) const PORTRAIT_SOURCE_FACE_ID: u8 = 1;
pub(crate) const PORTRAIT_SHOULDERS_HEIGHT: f32 = 36.0;
#[cfg(test)]
pub(crate) const FIRE_ROTATIONS: [u16; 8] = [0, 45, 90, 135, 180, 225, 270, 315];
pub(crate) const SPECIAL_PROJECTILE_FRAME_TIME: f32 = 0.05;

#[cfg(test)]
const ATTACK_RESET_BASE_TIME: f32 = 0.07;
#[cfg(test)]
const ATTACK_FRAME_BASE_TIME: f32 = 0.05;
#[cfg(test)]
const ATTACK_JITTER_STEP: f32 = 0.0003;
#[cfg(test)]
const PROJECTILE_FIRE_FRAME: usize = 2;

pub(crate) fn portrait_frame_path(team: TeamType, frame: usize) -> Option<String> {
    (team != TeamType::Null).then(|| {
        let team = team.atlas_team().asset_name();
        format!(
            "other/hud/portraits/pyro_{team}/SHEADBI{PORTRAIT_SOURCE_FACE_ID}_{:04}.png",
            frame.min(39)
        )
    })
}

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

pub(crate) fn hud_name() -> &'static str {
    "pyro"
}

pub(crate) fn selected_reporting_voice_asset_path() -> &'static str {
    "sounds/ROB12.wav"
}

pub(crate) fn fire_atlas_frame_name(team: TeamType, rotation: u16, frame: usize) -> String {
    let team_name = team.atlas_team().asset_name();
    format!(
        "robot_pyro_fire_{team_name}_r{rotation:03}_n{:02}",
        frame % FIRE_FRAME_COUNT
    )
}

pub(crate) fn special_projectile_kind() -> SpecialProjectileKind {
    SpecialProjectileKind::Flame
}

pub(crate) fn special_projectile_frame_paths() -> Vec<String> {
    flame_projectile_frame_paths()
}

pub(crate) fn special_projectile_duration(start: Vec2, target: Vec2) -> f32 {
    flame_projectile_duration(start, target)
}

pub(crate) fn special_projectile_frame_time() -> f32 {
    SPECIAL_PROJECTILE_FRAME_TIME
}

pub(crate) fn special_projectile_entity_name() -> &'static str {
    "pyro_flame_projectile"
}

pub(crate) fn special_projectile_next_frame(frame_count: usize, rng: &mut CombatRng) -> usize {
    rng.index(frame_count)
}

pub(crate) fn special_projectile_spawns_fire_impact() -> bool {
    true
}

pub(crate) fn attack_marks_fire_damage() -> bool {
    true
}

pub(crate) fn flame_projectile_frame_paths() -> Vec<String> {
    (0..FLAME_PROJECTILE_FRAME_COUNT)
        .map(|frame| format!("units/robots/pyro/bullet_n{frame:02}.png"))
        .collect()
}

pub(crate) fn flame_projectile_duration(start: Vec2, target: Vec2) -> f32 {
    (start.distance(target) / FLAME_PROJECTILE_SPEED).max(0.02)
}

pub(crate) fn fire_impact_profile() -> FireImpactProfile {
    FireImpactProfile {
        family_count: FIRE_IMPACT_FRAME_COUNTS.len(),
        frame_counts: FIRE_IMPACT_FRAME_COUNTS,
        frame_time: FIRE_IMPACT_FRAME_TIME,
        z: FIRE_IMPACT_Z,
    }
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
pub(crate) fn fire_animation_next_delay(next_frame: usize, rng: &mut CombatRng) -> f32 {
    let jitter = rng.index(100) as f32 * ATTACK_JITTER_STEP;
    if next_frame % FIRE_FRAME_COUNT == 0 {
        ATTACK_RESET_BASE_TIME + jitter
    } else {
        ATTACK_FRAME_BASE_TIME + jitter
    }
}

#[cfg(test)]
pub(crate) fn fire_animation_shoots_projectile(frame: usize) -> bool {
    frame % FIRE_FRAME_COUNT == PROJECTILE_FIRE_FRAME
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::CombatRng, original::types::TeamType};
    use bevy::prelude::Vec2;

    #[test]
    fn pyro_flame_projectile_assets_and_speed_match_original_effect() {
        assert_eq!(
            flame_projectile_frame_paths(),
            vec![
                "units/robots/pyro/bullet_n00.png",
                "units/robots/pyro/bullet_n01.png",
                "units/robots/pyro/bullet_n02.png",
                "units/robots/pyro/bullet_n03.png",
            ]
        );
        assert_eq!(
            flame_projectile_duration(Vec2::ZERO, Vec2::new(300.0, 0.0)),
            1.0
        );
        assert_eq!(
            special_projectile_duration(Vec2::ZERO, Vec2::new(300.0, 0.0)),
            1.0
        );
        assert_eq!(special_projectile_frame_time(), 0.05);
        assert_eq!(special_projectile_entity_name(), "pyro_flame_projectile");
        assert!(special_projectile_spawns_fire_impact());
        assert!(attack_marks_fire_damage());
    }

    #[test]
    fn pyro_fire_impact_profile_matches_original_families() {
        assert_eq!(
            fire_impact_profile(),
            FireImpactProfile {
                family_count: 5,
                frame_counts: [4, 4, 4, 6, 6],
                frame_time: 0.06,
                z: 34.4,
            }
        );
        assert_eq!(
            fire_impact_frame_paths(0),
            vec![
                "other/fire/fire0_n00.png",
                "other/fire/fire0_n01.png",
                "other/fire/fire0_n02.png",
                "other/fire/fire0_n03.png",
            ]
        );
        assert_eq!(fire_impact_frame_paths(3).len(), 6);
        assert_eq!(
            fire_impact_frame_paths(4).last().unwrap(),
            "other/fire/fire4_n05.png"
        );
        assert_eq!(fire_impact_frame_paths(99), fire_impact_frame_paths(4));
    }

    #[test]
    fn pyro_fire_animation_assets_match_original_paths() {
        assert_eq!(
            fire_animation_frame_path(TeamType::Blue, 7, 2).unwrap(),
            "units/robots/pyro/fire_blue_r315_n02.png"
        );
        assert_eq!(
            fire_animation_frame_path(TeamType::White, 99, 99).unwrap(),
            "units/robots/pyro/fire_red_r315_n02.png"
        );
        assert!(fire_animation_frame_paths(TeamType::Null, 0).is_none());
    }

    #[test]
    fn pyro_fire_animation_delay_and_fire_frame_match_original_ranges() {
        let mut rng = CombatRng::default();
        assert!(!fire_animation_shoots_projectile(0));
        assert!(!fire_animation_shoots_projectile(1));
        assert!(fire_animation_shoots_projectile(2));
        assert!(fire_animation_shoots_projectile(5));

        for _ in 0..32 {
            let reset = fire_animation_next_delay(0, &mut rng);
            assert!((0.07..=0.0997).contains(&reset));

            let frame = fire_animation_next_delay(1, &mut rng);
            assert!((0.05..=0.0797).contains(&frame));
        }
    }
}
