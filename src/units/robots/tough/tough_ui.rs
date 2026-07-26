use bevy::prelude::Vec2;

#[cfg(test)]
use crate::components::CombatRng;
use crate::components::{DamageCrater, DamageMissileVisual};
use crate::original::types::TeamType;

pub(crate) const ROCKET_FRAME_COUNT: usize = 2;
pub(crate) const MUSHROOM_FRAME_COUNT: usize = 12;
pub(crate) const MUSHROOM_FRAME_TIME: f32 = 0.08;
pub(crate) const MUSHROOM_Z: f32 = 34.0;
pub(crate) const SMOKE_FRAME_COUNT: usize = 8;
pub(crate) const SMOKE_FRAME_TIME: f32 = 0.12;
pub(crate) const SMOKE_Z: f32 = 34.2;
pub(crate) const FIRE_FRAME_COUNT: usize = 3;
pub(crate) const PORTRAIT_SOURCE_FACE_ID: u8 = 0;
pub(crate) const PORTRAIT_SHOULDERS_HEIGHT: f32 = 26.0;
#[cfg(test)]
pub(crate) const FIRE_ROTATIONS: [u16; 8] = [0, 45, 90, 135, 180, 225, 270, 315];

const MUSHROOM_SHIFT_Y: [f32; MUSHROOM_FRAME_COUNT] =
    [14.0, 9.0, 2.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

pub(crate) fn portrait_frame_path(team: TeamType, frame: usize) -> Option<String> {
    (team != TeamType::Null).then(|| {
        let team = team.atlas_team().asset_name();
        format!(
            "other/hud/portraits/tough_{team}/SHEADBI{PORTRAIT_SOURCE_FACE_ID}_{:04}.png",
            frame.min(39)
        )
    })
}
#[cfg(test)]
const ATTACK_RESET_BASE_TIME: f32 = 0.7;
#[cfg(test)]
const ATTACK_RESET_JITTER_STEP: f32 = 0.003;
#[cfg(test)]
const ATTACK_FRAME_BASE_TIME: f32 = 0.05;
#[cfg(test)]
const ATTACK_FRAME_JITTER_STEP: f32 = 0.0003;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MushroomEffectProfile {
    pub(crate) frame_count: usize,
    pub(crate) frame_time: f32,
    pub(crate) z: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SmokeEffectProfile {
    pub(crate) frame_count: usize,
    pub(crate) frame_time: f32,
    pub(crate) z: f32,
}

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(14.0)
}

pub(crate) fn hud_name() -> &'static str {
    "tough"
}

pub(crate) fn selected_reporting_voice_asset_path() -> &'static str {
    "sounds/ROB10.wav"
}

pub(crate) fn fire_atlas_frame_name(team: TeamType, rotation: u16, frame: usize) -> String {
    let team_name = team.atlas_team().asset_name();
    format!(
        "robot_tough_fire_{team_name}_r{rotation:03}_n{:02}",
        frame % FIRE_FRAME_COUNT
    )
}

pub(crate) fn damage_missile_visual() -> Option<DamageMissileVisual> {
    Some(DamageMissileVisual::ToughRocket)
}

pub(crate) fn rocket_frame_paths() -> Vec<String> {
    (0..ROCKET_FRAME_COUNT)
        .map(|frame| format!("units/robots/tough/bullet_n{frame:02}.png"))
        .collect()
}

pub(crate) fn damage_missile_frame_paths() -> Vec<String> {
    rocket_frame_paths()
}

pub(crate) fn damage_crater() -> DamageCrater {
    DamageCrater {
        is_big: false,
        chance: 0.35,
        big_chance: None,
    }
}

pub(crate) fn rocket_muzzle_offset(direction: usize) -> Vec2 {
    const SX: [f32; 8] = [8.0, 8.0, 0.0, -8.0, -8.0, -8.0, 0.0, 8.0];
    const SY: [f32; 8] = [0.0, -8.0, -8.0, -8.0, 0.0, 8.0, 8.0, 8.0];
    let direction = direction.min(7);
    Vec2::new(SX[direction], -SY[direction])
}

pub(crate) fn mushroom_effect_profile() -> MushroomEffectProfile {
    MushroomEffectProfile {
        frame_count: MUSHROOM_FRAME_COUNT,
        frame_time: MUSHROOM_FRAME_TIME,
        z: MUSHROOM_Z,
    }
}

pub(crate) fn mushroom_frame_paths() -> Vec<String> {
    (0..MUSHROOM_FRAME_COUNT)
        .map(|frame| format!("units/robots/tough/mushroom_n{frame:02}.png"))
        .collect()
}

pub(crate) fn mushroom_base_top_left(map_center: Vec2, scale: f32) -> Vec2 {
    map_center - Vec2::new(16.0 * scale, 32.0 * scale)
}

pub(crate) fn mushroom_frame_offsets(scale: f32) -> Vec<Vec2> {
    MUSHROOM_SHIFT_Y
        .into_iter()
        .map(|shift_y| Vec2::new(0.0, shift_y * scale))
        .collect()
}

pub(crate) fn smoke_effect_profile() -> SmokeEffectProfile {
    SmokeEffectProfile {
        frame_count: SMOKE_FRAME_COUNT,
        frame_time: SMOKE_FRAME_TIME,
        z: SMOKE_Z,
    }
}

pub(crate) fn smoke_frame_paths() -> Vec<String> {
    (0..SMOKE_FRAME_COUNT)
        .map(|frame| format!("units/robots/tough/smoke_n{frame:02}.png"))
        .collect()
}

pub(crate) fn rocket_smoke_positions(
    start: Vec2,
    target: Vec2,
    total_time: f32,
    smoke_time_cursor: &mut f32,
    elapsed: f32,
    offsets: &[Vec2],
) -> Vec<Vec2> {
    let speed = rocket_speed(start, target, total_time);
    if speed <= f32::EPSILON || total_time <= f32::EPSILON || offsets.is_empty() {
        return Vec::new();
    }

    let smoke_back_time = rocket_smoke_back_time(speed);
    let smoke_interval = rocket_smoke_interval(speed);
    let mut positions = Vec::new();
    while elapsed - *smoke_time_cursor > smoke_interval {
        let smoke_elapsed = *smoke_time_cursor - smoke_back_time;
        let progress = smoke_elapsed / total_time;
        let base = start.lerp(target, progress);
        positions.extend(offsets.iter().map(|offset| base + *offset));
        *smoke_time_cursor += smoke_interval;
    }
    positions
}

pub(crate) fn rocket_smoke_back_time(speed: f32) -> f32 {
    if speed <= f32::EPSILON {
        0.0
    } else {
        6.0 / speed
    }
}

pub(crate) fn rocket_smoke_interval(speed: f32) -> f32 {
    if speed <= f32::EPSILON {
        0.0
    } else {
        8.0 / speed
    }
}

pub(crate) fn rocket_speed(start: Vec2, target: Vec2, total_time: f32) -> f32 {
    if total_time <= f32::EPSILON {
        0.0
    } else {
        start.distance(target) / total_time
    }
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
        "units/robots/tough/fire_{team_name}_r{rotation:03}_n{frame:02}.png"
    ))
}

#[cfg(test)]
pub(crate) fn fire_animation_frame_paths(team: TeamType, direction: usize) -> Option<Vec<String>> {
    (team != TeamType::Null).then(|| {
        (0..FIRE_FRAME_COUNT)
            .map(|frame| {
                fire_animation_frame_path(team, direction, frame)
                    .expect("non-null team has tough fire frame")
            })
            .collect()
    })
}

#[cfg(test)]
pub(crate) fn fire_animation_next_delay(next_frame: usize, rng: &mut CombatRng) -> f32 {
    let jitter = rng.index(100) as f32;
    if next_frame % FIRE_FRAME_COUNT == 0 {
        ATTACK_RESET_BASE_TIME + jitter * ATTACK_RESET_JITTER_STEP
    } else {
        ATTACK_FRAME_BASE_TIME + jitter * ATTACK_FRAME_JITTER_STEP
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::CombatRng, original::types::TeamType};
    use bevy::prelude::Vec2;

    #[test]
    fn tough_projectile_and_fire_assets_match_original_paths() {
        assert_eq!(
            rocket_frame_paths(),
            vec![
                "units/robots/tough/bullet_n00.png",
                "units/robots/tough/bullet_n01.png",
            ]
        );
        assert_eq!(
            fire_animation_frame_path(TeamType::Red, 2, 1).unwrap(),
            "units/robots/tough/fire_red_r090_n01.png"
        );
        assert_eq!(
            fire_animation_frame_path(TeamType::Purple, 99, 99).unwrap(),
            "units/robots/tough/fire_red_r315_n02.png"
        );
        assert!(fire_animation_frame_paths(TeamType::Null, 0).is_none());
    }

    #[test]
    fn tough_mushroom_effect_profile_matches_original_frames_and_offsets() {
        assert_eq!(
            mushroom_effect_profile(),
            MushroomEffectProfile {
                frame_count: 12,
                frame_time: 0.08,
                z: 34.0,
            }
        );
        assert_eq!(mushroom_frame_paths().len(), 12);
        assert_eq!(
            mushroom_frame_paths().last().unwrap(),
            "units/robots/tough/mushroom_n11.png"
        );
        assert_eq!(
            mushroom_base_top_left(Vec2::new(100.0, 50.0), 1.5),
            Vec2::new(76.0, 2.0)
        );
        assert_eq!(mushroom_frame_offsets(1.5)[0], Vec2::new(0.0, 21.0));
        assert_eq!(mushroom_frame_offsets(1.5)[11], Vec2::new(0.0, 9.0));
    }

    #[test]
    fn tough_smoke_effect_profile_matches_original_frames_and_timing() {
        assert_eq!(
            smoke_effect_profile(),
            SmokeEffectProfile {
                frame_count: 8,
                frame_time: 0.12,
                z: 34.2,
            }
        );
        assert_eq!(smoke_frame_paths().len(), 8);
        assert_eq!(
            smoke_frame_paths().last().unwrap(),
            "units/robots/tough/smoke_n07.png"
        );
        assert_eq!(rocket_smoke_back_time(150.0), 6.0 / 150.0);
        assert_eq!(rocket_smoke_interval(150.0), 8.0 / 150.0);
    }

    #[test]
    fn tough_rocket_smoke_positions_match_original_back_offset() {
        let start = Vec2::ZERO;
        let target = Vec2::new(150.0, 0.0);
        let total_time = 1.0;
        let mut cursor = 0.0;

        assert!(
            rocket_smoke_positions(start, target, total_time, &mut cursor, 0.05, &[Vec2::ZERO])
                .is_empty()
        );
        let smoke =
            rocket_smoke_positions(start, target, total_time, &mut cursor, 0.06, &[Vec2::ZERO]);

        assert_eq!(smoke.len(), 1);
        assert_eq!(cursor, 8.0 / 150.0);
        assert!((smoke[0].x + 6.0).abs() < 0.0001);
        assert_eq!(smoke[0].y, 0.0);
    }

    #[test]
    fn tough_fire_animation_delay_matches_original_ranges() {
        let mut rng = CombatRng::default();
        for _ in 0..32 {
            let reset = fire_animation_next_delay(0, &mut rng);
            assert!((0.7..=0.997).contains(&reset));

            let frame = fire_animation_next_delay(1, &mut rng);
            assert!((0.05..=0.0797).contains(&frame));
        }
    }
}
