use bevy::prelude::Vec2;

use crate::{
    components::ProductionLevel,
    original::objects::{CannonType, ObjectKind, RobotType, VehicleType},
    original::types::PlanetType,
};

use super::ProductionPlacement;
use super::{BuildingDeathProfile, BuildingEffectBox};

pub(crate) const DESTROYED_OVERLAY_PATH: &str = "buildings/fort/destroyed_overlay.png";
pub(crate) const DESTROYED_OVERLAY_ALPHA_MIN: u8 = 1;
pub(crate) const DESTROYED_OVERLAY_ALPHA_MAX: u8 = 254;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/fort/fort_{}_front_destroyed.png",
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
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
        ],
        Level1 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
        ],
        Level2 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Cannon(CannonType::Howitzer),
        ],
        Level3 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Vehicle(VehicleType::Apc),
        ],
        Level4 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Robot(RobotType::Laser),
            ObjectKind::Vehicle(VehicleType::Heavy),
            ObjectKind::Cannon(CannonType::MissileCannon),
        ],
        Level5 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Robot(RobotType::Laser),
            ObjectKind::Vehicle(VehicleType::Heavy),
            ObjectKind::Cannon(CannonType::MissileCannon),
            ObjectKind::Vehicle(VehicleType::MissileLauncher),
        ],
    }
}

pub(crate) fn production_placement() -> ProductionPlacement {
    ProductionPlacement {
        create_offset: Vec2::new(80.0, 128.0),
        move_offset: Vec2::new(80.0, 208.0),
    }
}

pub(crate) fn death_profile(planet: PlanetType) -> BuildingDeathProfile {
    BuildingDeathProfile {
        effect_box: BuildingEffectBox {
            x: 18.0,
            y: 18.0,
            width: 136,
            height: 118,
        },
        width_pix: 160.0,
        height_pix: match planet {
            PlanetType::Jungle => 176.0,
            PlanetType::Desert | PlanetType::Volcanic | PlanetType::Arctic | PlanetType::City => {
                192.0
            }
        },
        max_effects_base: 20,
        max_effects_random: 8,
        fireball_base: 12,
        fireball_random: 6,
        piece_base: 16,
        piece_random: 6,
        piece_variants: 5,
        piece_flight_base: 3.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn death_profile_matches_original_do_death_effect() {
        assert_eq!(
            death_profile(PlanetType::Desert),
            BuildingDeathProfile {
                effect_box: BuildingEffectBox {
                    x: 18.0,
                    y: 18.0,
                    width: 136,
                    height: 118
                },
                width_pix: 160.0,
                height_pix: 192.0,
                max_effects_base: 20,
                max_effects_random: 8,
                fireball_base: 12,
                fireball_random: 6,
                piece_base: 16,
                piece_random: 6,
                piece_variants: 5,
                piece_flight_base: 3.0
            }
        );
        assert_eq!(death_profile(PlanetType::Jungle).height_pix, 176.0);
    }

    #[test]
    fn destroyed_assets_and_overlay_match_original_paths() {
        assert_eq!(
            destroyed_asset_path(PlanetType::Desert),
            "buildings/fort/fort_desert_front_destroyed.png"
        );
        assert_eq!(
            DESTROYED_OVERLAY_PATH,
            "buildings/fort/destroyed_overlay.png"
        );
        assert_eq!(DESTROYED_OVERLAY_ALPHA_MIN, 1);
        assert_eq!(DESTROYED_OVERLAY_ALPHA_MAX, 254);
    }
}
