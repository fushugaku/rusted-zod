use std::collections::HashMap;

use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, ObjectStats, ProductionLevel},
    original::objects::{CannonType, ObjectKind, RobotType},
    original::types::{PlanetType, TeamType},
    render::atlas::FactoryOverlayKind,
};

use super::ProductionPlacement;
use super::{BuildingDeathProfile, BuildingEffectBox};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/robot/base_destroyed_{}.png",
        super::planet_asset_name(planet)
    )
}

pub(crate) fn default_production_unit() -> ObjectKind {
    ObjectKind::Robot(RobotType::Grunt)
}

pub(crate) fn default_build_list(level: ProductionLevel) -> Vec<ObjectKind> {
    use ProductionLevel::*;

    match level {
        Level0 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Cannon(CannonType::Gatling),
        ],
        Level1 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
        ],
        Level2 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Cannon(CannonType::Gun),
        ],
        Level3 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Cannon(CannonType::Howitzer),
        ],
        Level4 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Robot(RobotType::Laser),
        ],
        Level5 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Robot(RobotType::Laser),
            ObjectKind::Cannon(CannonType::MissileCannon),
        ],
    }
}

pub(crate) fn production_placement() -> ProductionPlacement {
    ProductionPlacement {
        create_offset: Vec2::new(43.0, 53.0),
        move_offset: Vec2::new(43.0, 96.0),
    }
}

pub(crate) fn overlay_kind_is_robot(kind: FactoryOverlayKind) -> bool {
    matches!(
        kind,
        FactoryOverlayKind::RobotExhaust
            | FactoryOverlayKind::RobotGreenBox
            | FactoryOverlayKind::RobotSingleLight0
            | FactoryOverlayKind::RobotSingleLight1
            | FactoryOverlayKind::RobotSingleLight2
            | FactoryOverlayKind::RobotDoubleLight
            | FactoryOverlayKind::RobotBody
            | FactoryOverlayKind::RobotSpin
    )
}

pub(crate) fn overlay_should_be_visible(
    kind: FactoryOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    production_active: bool,
    current: usize,
) -> bool {
    if stats.destroyed() || !overlay_kind_is_robot(kind) {
        return false;
    }

    if owner == TeamType::Null || !production_active {
        return matches!(kind, FactoryOverlayKind::RobotBody);
    }

    if overlay_is_single_light(kind) && current == 0 {
        return false;
    }

    true
}

pub(crate) fn overlay_next_frame(
    ref_id: u32,
    kind: FactoryOverlayKind,
    current: usize,
    frame_count: usize,
    rng: &mut CombatRng,
    robot_light_updates: &mut HashMap<u32, Option<[usize; 3]>>,
) -> usize {
    if let Some(light_index) = overlay_single_light_index(kind) {
        let update = *robot_light_updates
            .entry(ref_id)
            .or_insert_with(|| robot_single_light_update(rng));
        return update.map(|states| states[light_index]).unwrap_or(current);
    }

    (current + 1) % frame_count
}

pub(crate) fn robot_single_light_update(rng: &mut CombatRng) -> Option<[usize; 3]> {
    (rng.index(3) == 0).then(|| [rng.index(2), rng.index(2), rng.index(2)])
}

pub(crate) fn overlay_is_single_light(kind: FactoryOverlayKind) -> bool {
    overlay_single_light_index(kind).is_some()
}

pub(crate) fn overlay_single_light_index(kind: FactoryOverlayKind) -> Option<usize> {
    match kind {
        FactoryOverlayKind::RobotSingleLight0 => Some(0),
        FactoryOverlayKind::RobotSingleLight1 => Some(1),
        FactoryOverlayKind::RobotSingleLight2 => Some(2),
        _ => None,
    }
}

pub(crate) fn death_profile() -> BuildingDeathProfile {
    BuildingDeathProfile {
        effect_box: BuildingEffectBox {
            x: 8.0,
            y: 8.0,
            width: 40,
            height: 56,
        },
        width_pix: 64.0,
        height_pix: 80.0,
        max_effects_base: 8,
        max_effects_random: 4,
        fireball_base: 8,
        fireball_random: 3,
        piece_base: 6,
        piece_random: 3,
        piece_variants: 2,
        piece_flight_base: 1.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::BuildingType;

    #[test]
    fn death_profile_matches_original_do_death_effect() {
        assert_eq!(
            death_profile(),
            BuildingDeathProfile {
                effect_box: BuildingEffectBox {
                    x: 8.0,
                    y: 8.0,
                    width: 40,
                    height: 56
                },
                width_pix: 64.0,
                height_pix: 80.0,
                max_effects_base: 8,
                max_effects_random: 4,
                fireball_base: 8,
                fireball_random: 3,
                piece_base: 6,
                piece_random: 3,
                piece_variants: 2,
                piece_flight_base: 1.5
            }
        );
    }

    #[test]
    fn destroyed_asset_matches_original_path() {
        assert_eq!(
            destroyed_asset_path(PlanetType::Jungle),
            "buildings/robot/base_destroyed_jungle.png"
        );
    }

    #[test]
    fn overlay_visibility_matches_original_owner_building_and_destroyed_rules() {
        let live = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let destroyed = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 0);

        assert!(overlay_should_be_visible(
            FactoryOverlayKind::RobotBody,
            TeamType::Null,
            live,
            false,
            0
        ));
        assert!(!overlay_should_be_visible(
            FactoryOverlayKind::RobotSpin,
            TeamType::Null,
            live,
            false,
            0
        ));
        assert!(overlay_should_be_visible(
            FactoryOverlayKind::RobotSpin,
            TeamType::Red,
            live,
            true,
            0
        ));
        assert!(!overlay_should_be_visible(
            FactoryOverlayKind::RobotBody,
            TeamType::Red,
            destroyed,
            true,
            0
        ));
        assert!(!overlay_should_be_visible(
            FactoryOverlayKind::RobotSingleLight0,
            TeamType::Red,
            live,
            true,
            0
        ));
        assert!(overlay_should_be_visible(
            FactoryOverlayKind::RobotSingleLight0,
            TeamType::Red,
            live,
            true,
            1
        ));
    }

    #[test]
    fn robot_single_lights_use_original_random_on_skip_behavior() {
        let mut no_reroll = CombatRng::default();
        let mut no_updates = HashMap::new();
        assert_eq!(
            overlay_next_frame(
                10,
                FactoryOverlayKind::RobotSingleLight0,
                0,
                2,
                &mut no_reroll,
                &mut no_updates,
            ),
            0
        );
        assert_eq!(
            overlay_next_frame(
                10,
                FactoryOverlayKind::RobotSingleLight1,
                1,
                2,
                &mut no_reroll,
                &mut no_updates,
            ),
            1
        );

        let mut reroll_on = CombatRng(3);
        let mut light_updates = HashMap::new();
        assert_eq!(
            overlay_next_frame(
                20,
                FactoryOverlayKind::RobotSingleLight0,
                0,
                2,
                &mut reroll_on,
                &mut light_updates,
            ),
            1
        );
        assert_eq!(
            overlay_next_frame(
                20,
                FactoryOverlayKind::RobotSingleLight1,
                0,
                2,
                &mut reroll_on,
                &mut light_updates,
            ),
            1
        );
        assert_eq!(
            overlay_next_frame(
                20,
                FactoryOverlayKind::RobotSingleLight2,
                0,
                2,
                &mut reroll_on,
                &mut light_updates,
            ),
            1
        );

        let mut non_light = CombatRng::default();
        let mut non_light_updates = HashMap::new();
        assert_eq!(
            overlay_next_frame(
                30,
                FactoryOverlayKind::RobotBody,
                1,
                2,
                &mut non_light,
                &mut non_light_updates,
            ),
            0
        );
    }
}
