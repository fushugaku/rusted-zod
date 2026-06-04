use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, DamageMissileVisual},
    original::{
        objects::{ObjectKind, RobotType},
        settings::UnitSettings,
        types::TeamType,
    },
    units::UnitAttackSound,
};

pub(crate) mod grunt;
pub(crate) mod laser;
pub(crate) mod psycho;
pub(crate) mod pyro;
pub(crate) mod sniper;
pub(crate) mod tough;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SpecialProjectileKind {
    Laser,
    Flame,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeathEffectChoice {
    None,
    Normal,
    Melt,
    Turrent,
}

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

pub(crate) fn default_selection_size(robot: RobotType) -> Vec2 {
    match robot {
        RobotType::Grunt => grunt::default_selection_size(),
        RobotType::Psycho => psycho::default_selection_size(),
        RobotType::Sniper => sniper::default_selection_size(),
        RobotType::Tough => tough::default_selection_size(),
        RobotType::Pyro => pyro::default_selection_size(),
        RobotType::Laser => laser::default_selection_size(),
    }
}

pub(crate) fn settings(robot: RobotType) -> UnitSettings {
    match robot {
        RobotType::Grunt => grunt::settings(),
        RobotType::Psycho => psycho::settings(),
        RobotType::Sniper => sniper::settings(),
        RobotType::Tough => tough::settings(),
        RobotType::Pyro => pyro::settings(),
        RobotType::Laser => laser::settings(),
    }
}

pub(crate) fn attack_sound(robot: RobotType) -> Option<UnitAttackSound> {
    match robot {
        RobotType::Grunt => grunt::attack_sound(),
        RobotType::Psycho => psycho::attack_sound(),
        RobotType::Sniper => sniper::attack_sound(),
        RobotType::Tough => tough::attack_sound(),
        RobotType::Pyro => pyro::attack_sound(),
        RobotType::Laser => laser::attack_sound(),
    }
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

pub(crate) fn special_projectile_kind(robot: RobotType) -> Option<SpecialProjectileKind> {
    match robot {
        RobotType::Pyro => Some(pyro::special_projectile_kind()),
        RobotType::Laser => Some(laser::special_projectile_kind()),
        RobotType::Grunt | RobotType::Psycho | RobotType::Sniper | RobotType::Tough => None,
    }
}

pub(crate) fn special_projectile_frame_paths(kind: SpecialProjectileKind) -> Vec<String> {
    match kind {
        SpecialProjectileKind::Laser => laser::special_projectile_frame_paths(),
        SpecialProjectileKind::Flame => pyro::special_projectile_frame_paths(),
    }
}

pub(crate) fn pyro_fire_impact_profile() -> pyro::FireImpactProfile {
    pyro::fire_impact_profile()
}

pub(crate) fn pyro_fire_impact_family_count() -> usize {
    pyro::fire_impact_family_count()
}

pub(crate) fn pyro_fire_impact_frame_paths(family: usize) -> Vec<String> {
    pyro::fire_impact_frame_paths(family)
}

pub(crate) fn damage_missile_visual(robot: RobotType) -> Option<DamageMissileVisual> {
    match robot {
        RobotType::Tough => tough::damage_missile_visual(),
        RobotType::Grunt
        | RobotType::Psycho
        | RobotType::Sniper
        | RobotType::Pyro
        | RobotType::Laser => None,
    }
}

pub(crate) fn tough_mushroom_effect_profile() -> tough::MushroomEffectProfile {
    tough::mushroom_effect_profile()
}

pub(crate) fn tough_mushroom_frame_paths() -> Vec<String> {
    tough::mushroom_frame_paths()
}

pub(crate) fn tough_mushroom_base_top_left(map_center: Vec2, scale: f32) -> Vec2 {
    tough::mushroom_base_top_left(map_center, scale)
}

pub(crate) fn tough_mushroom_frame_offsets(scale: f32) -> Vec<Vec2> {
    tough::mushroom_frame_offsets(scale)
}

pub(crate) fn tough_smoke_effect_profile() -> tough::SmokeEffectProfile {
    tough::smoke_effect_profile()
}

pub(crate) fn tough_smoke_frame_paths() -> Vec<String> {
    tough::smoke_frame_paths()
}

pub(crate) fn tough_rocket_smoke_positions(
    start: Vec2,
    target: Vec2,
    total_time: f32,
    smoke_time_cursor: &mut f32,
    elapsed: f32,
    offsets: &[Vec2],
) -> Vec<Vec2> {
    tough::rocket_smoke_positions(
        start,
        target,
        total_time,
        smoke_time_cursor,
        elapsed,
        offsets,
    )
}

pub(crate) fn death_effect_choice(
    kind: ObjectKind,
    team: TeamType,
    do_fire_death: bool,
    do_missile_death: bool,
) -> DeathEffectChoice {
    if !matches!(kind, ObjectKind::Robot(_)) || team == TeamType::Null {
        return DeathEffectChoice::None;
    }
    if do_missile_death {
        DeathEffectChoice::Turrent
    } else if do_fire_death {
        DeathEffectChoice::Melt
    } else {
        DeathEffectChoice::Normal
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::VehicleType;

    #[test]
    fn robot_death_effect_choice_matches_original_priority() {
        let robot = ObjectKind::Robot(RobotType::Grunt);

        assert_eq!(
            death_effect_choice(robot, TeamType::Red, true, true),
            DeathEffectChoice::Turrent
        );
        assert_eq!(
            death_effect_choice(robot, TeamType::Red, true, false),
            DeathEffectChoice::Melt
        );
        assert_eq!(
            death_effect_choice(robot, TeamType::Red, false, false),
            DeathEffectChoice::Normal
        );
        assert_eq!(
            death_effect_choice(robot, TeamType::Null, true, true),
            DeathEffectChoice::None
        );
        assert_eq!(
            death_effect_choice(
                ObjectKind::Vehicle(VehicleType::Jeep),
                TeamType::Red,
                true,
                true
            ),
            DeathEffectChoice::None
        );
    }

    #[test]
    fn robot_death_effect_frame_paths_match_original_assets() {
        let die1 = normal_death_frame_paths(TeamType::Red, 0).unwrap();
        let die4 = normal_death_frame_paths(TeamType::Red, 3).unwrap();
        let die_clamped = normal_death_frame_paths(TeamType::Red, 99).unwrap();
        let melt = melt_death_frame_paths(TeamType::Blue).unwrap();
        let turrent = turrent_death_frame_paths(TeamType::Green).unwrap();

        assert_eq!(die1.len(), 10);
        assert_eq!(die1.first().unwrap(), "units/robots/die1_red_n00.png");
        assert_eq!(die4.len(), 8);
        assert_eq!(die4.last().unwrap(), "units/robots/die4_red_n07.png");
        assert_eq!(die_clamped, die4);
        assert_eq!(melt.len(), 17);
        assert_eq!(melt.last().unwrap(), "units/robots/melt_blue_n16.png");
        assert_eq!(turrent.len(), 33);
        assert_eq!(turrent.last().unwrap(), "units/robots/die5_green_n32.png");
        assert!(normal_death_frame_paths(TeamType::Null, 0).is_none());
        assert!(melt_death_frame_paths(TeamType::Null).is_none());
        assert!(turrent_death_frame_paths(TeamType::Null).is_none());
    }

    #[test]
    fn robot_death_top_left_converts_from_center_like_original_robot_loc() {
        assert_eq!(
            death_top_left_world(Vec2::new(100.0, -50.0)),
            Vec2::new(92.0, -42.0)
        );
    }

    #[test]
    fn robot_death_animation_profile_matches_original_ranges() {
        assert_eq!(
            death_animation_profile(),
            RobotDeathAnimationProfile {
                death_frame_time: 0.16,
                death_z: 34.0,
                normal_frame_counts: [10, 10, 10, 8],
                melt_frame_count: 17,
                turrent_frame_count: 33,
                turrent_air_frame_count: 8,
                turrent_land_start_frame: 8,
                turrent_last_frame: 32,
                flip_air_frame_time: 0.05,
                flip_land_frame_time: 0.15,
            }
        );
        assert_eq!(turrent_death_arc_size(2.0, 4.0, 0.0), 1.0);
        assert!(turrent_death_arc_size(2.0, 4.0, 1.0) > 1.0);

        let mut rng = CombatRng::default();
        for _ in 0..32 {
            let trajectory = turrent_death_trajectory(Vec2::new(10.0, 20.0), &mut rng);
            assert!((3.5..=4.4).contains(&trajectory.final_time));
            assert!((1.3..=3.29).contains(&trajectory.rise));
            assert!(
                (trajectory.start_map.x - 10.0) >= -2.0 && (trajectory.start_map.x - 10.0) <= 2.0
            );
            assert!(
                (trajectory.end_map.x - trajectory.start_map.x) > -100.0
                    && (trajectory.end_map.x - trajectory.start_map.x) <= 100.0
            );
        }
    }

    #[test]
    fn robot_effect_wrappers_expose_unit_side_assets_to_main() {
        assert_eq!(
            pyro_fire_impact_profile(),
            pyro::FireImpactProfile {
                family_count: 5,
                frame_counts: [4, 4, 4, 6, 6],
                frame_time: 0.06,
                z: 34.4,
            }
        );
        assert_eq!(pyro_fire_impact_family_count(), 5);
        assert_eq!(
            pyro_fire_impact_frame_paths(4).last().unwrap(),
            "other/fire/fire4_n05.png"
        );

        assert_eq!(tough_mushroom_effect_profile().frame_time, 0.08);
        assert_eq!(tough_smoke_effect_profile().frame_time, 0.12);
        assert_eq!(
            tough_mushroom_frame_paths().last().unwrap(),
            "units/robots/tough/mushroom_n11.png"
        );
        assert_eq!(
            tough_smoke_frame_paths().last().unwrap(),
            "units/robots/tough/smoke_n07.png"
        );
        assert_eq!(
            tough_mushroom_base_top_left(Vec2::new(100.0, 50.0), 1.5),
            Vec2::new(76.0, 2.0)
        );
        assert_eq!(tough_mushroom_frame_offsets(1.5)[0], Vec2::new(0.0, 21.0));

        let mut cursor = 0.0;
        let smoke = tough_rocket_smoke_positions(
            Vec2::ZERO,
            Vec2::new(150.0, 0.0),
            1.0,
            &mut cursor,
            0.06,
            &[Vec2::ZERO],
        );
        assert_eq!(smoke.len(), 1);
        assert!((smoke[0].x + 6.0).abs() < 0.0001);
    }
}
