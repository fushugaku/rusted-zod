use bevy::prelude::Vec2;

use crate::{
    components::CombatRng,
    original::{settings::UnitSettings, types::TeamType},
    units::{UnitAttackSound, robots::SpecialProjectileKind, run_time},
};

pub(crate) const FLAME_PROJECTILE_SPEED: f32 = 300.0;

const PROJECTILE_FIRE_FRAME: usize = 2;

pub(crate) use super::pyro_ui::FireImpactProfile;

pub(crate) fn default_selection_size() -> Vec2 {
    super::pyro_ui::default_selection_size()
}

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 4,
        move_speed: 12.0,
        attack_radius: 120.0,
        attack_damage: 0.010486,
        attack_damage_chance: 0.7,
        attack_damage_radius: 0.0,
        attack_missile_speed: 0.0,
        attack_speed: 0.1,
        attack_snipe_chance: 0.0,
        health_ratio: 20.0 / 74.0,
        build_time: 161.0,
        max_run_time: run_time(120.0, 12.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Pyro)
}

pub(crate) fn special_projectile_kind() -> SpecialProjectileKind {
    SpecialProjectileKind::Flame
}

pub(crate) fn special_projectile_frame_paths() -> Vec<String> {
    super::pyro_ui::special_projectile_frame_paths()
}

pub(crate) fn flame_projectile_frame_paths() -> Vec<String> {
    super::pyro_ui::flame_projectile_frame_paths()
}

pub(crate) fn flame_projectile_duration(start: Vec2, target: Vec2) -> f32 {
    (start.distance(target) / FLAME_PROJECTILE_SPEED).max(0.02)
}

pub(crate) fn fire_impact_profile() -> FireImpactProfile {
    super::pyro_ui::fire_impact_profile()
}

pub(crate) fn fire_impact_family_count() -> usize {
    super::pyro_ui::fire_impact_family_count()
}

pub(crate) fn fire_impact_frame_count(family: usize) -> usize {
    super::pyro_ui::fire_impact_frame_count(family)
}

pub(crate) fn fire_impact_frame_paths(family: usize) -> Vec<String> {
    super::pyro_ui::fire_impact_frame_paths(family)
}

pub(crate) fn fire_animation_frame_path(
    team: TeamType,
    direction: usize,
    frame: usize,
) -> Option<String> {
    super::pyro_ui::fire_animation_frame_path(team, direction, frame)
}

pub(crate) fn fire_animation_frame_paths(team: TeamType, direction: usize) -> Option<Vec<String>> {
    super::pyro_ui::fire_animation_frame_paths(team, direction)
}

pub(crate) fn fire_animation_next_delay(next_frame: usize, rng: &mut CombatRng) -> f32 {
    super::pyro_ui::fire_animation_next_delay(next_frame, rng)
}

pub(crate) fn fire_animation_shoots_projectile(frame: usize) -> bool {
    frame % super::pyro_ui::FIRE_FRAME_COUNT == PROJECTILE_FIRE_FRAME
}

#[cfg(test)]
mod tests {
    use super::*;

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
