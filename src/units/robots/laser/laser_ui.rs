use bevy::prelude::Vec2;

use crate::components::CombatRng;
use crate::original::types::TeamType;

use crate::units::robots::SpecialProjectileKind;

pub(crate) const FIRE_FRAME_COUNT: usize = 3;
pub(crate) const PORTRAIT_SOURCE_FACE_ID: u8 = 1;
pub(crate) const PORTRAIT_SHOULDERS_HEIGHT: f32 = 36.0;
pub(crate) const LASER_PROJECTILE_SPEED: f32 = 300.0;
pub(crate) const LASER_PROJECTILE_FRAME_COUNT: usize = 2;
pub(crate) const SPECIAL_PROJECTILE_FRAME_TIME: f32 = 0.05;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(14.0)
}

pub(crate) fn hud_name() -> &'static str {
    "laser"
}

pub(crate) fn portrait_frame_path(team: TeamType, frame: usize) -> Option<String> {
    (team != TeamType::Null).then(|| {
        let team = team.atlas_team().asset_name();
        format!(
            "other/hud/portraits/laser_{team}/SHEADBI{PORTRAIT_SOURCE_FACE_ID}_{:04}.png",
            frame.min(39)
        )
    })
}

pub(crate) fn selected_reporting_voice_asset_path() -> &'static str {
    "sounds/ROB11.wav"
}

pub(crate) fn fire_atlas_frame_name(team: TeamType, rotation: u16, frame: usize) -> String {
    let team_name = team.atlas_team().asset_name();
    format!(
        "robot_laser_fire_{team_name}_r{rotation:03}_n{:02}",
        frame % FIRE_FRAME_COUNT
    )
}

pub(crate) fn special_projectile_kind() -> SpecialProjectileKind {
    SpecialProjectileKind::Laser
}

pub(crate) fn special_projectile_frame_paths() -> Vec<String> {
    (0..LASER_PROJECTILE_FRAME_COUNT)
        .map(|frame| format!("units/robots/laser/bullet_n{frame:02}.png"))
        .collect()
}

pub(crate) fn special_projectile_duration(start: Vec2, target: Vec2) -> f32 {
    (start.distance(target) / LASER_PROJECTILE_SPEED).max(0.02)
}

pub(crate) fn special_projectile_frame_time() -> f32 {
    SPECIAL_PROJECTILE_FRAME_TIME
}

pub(crate) fn special_projectile_entity_name() -> &'static str {
    "laser_projectile"
}

pub(crate) fn special_projectile_next_frame(current: usize, _rng: &mut CombatRng) -> usize {
    current
}

pub(crate) fn special_projectile_spawns_fire_impact() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::CombatRng;

    #[test]
    fn laser_special_projectile_profile_matches_original_effect() {
        assert_eq!(
            special_projectile_frame_paths(),
            vec![
                "units/robots/laser/bullet_n00.png",
                "units/robots/laser/bullet_n01.png",
            ]
        );
        assert_eq!(
            special_projectile_duration(Vec2::ZERO, Vec2::new(300.0, 0.0)),
            1.0
        );
        assert_eq!(special_projectile_frame_time(), 0.05);
        assert_eq!(special_projectile_entity_name(), "laser_projectile");
        assert!(!special_projectile_spawns_fire_impact());

        let mut rng = CombatRng::default();
        assert_eq!(special_projectile_next_frame(1, &mut rng), 1);
    }
}
