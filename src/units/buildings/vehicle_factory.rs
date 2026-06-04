use bevy::prelude::Vec2;

use crate::{
    components::{ObjectStats, ProductionLevel},
    original::objects::{CannonType, ObjectKind, VehicleType},
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
        "buildings/vehicle/base_destroyed_{}.png",
        super::planet_asset_name(planet)
    )
}

pub(crate) fn default_production_unit() -> ObjectKind {
    ObjectKind::Vehicle(VehicleType::Jeep)
}

pub(crate) fn default_build_list(level: ProductionLevel) -> Vec<ObjectKind> {
    use ProductionLevel::*;

    match level {
        Level0 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
        ],
        Level1 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
        ],
        Level2 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Vehicle(VehicleType::Medium),
        ],
        Level3 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Cannon(CannonType::Howitzer),
        ],
        Level4 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Vehicle(VehicleType::Heavy),
        ],
        Level5 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Vehicle(VehicleType::Heavy),
            ObjectKind::Vehicle(VehicleType::MissileLauncher),
            ObjectKind::Cannon(CannonType::MissileCannon),
        ],
    }
}

pub(crate) fn production_placement() -> ProductionPlacement {
    ProductionPlacement {
        create_offset: Vec2::new(32.0, 48.0),
        move_offset: Vec2::new(32.0, 96.0),
    }
}

pub(crate) fn overlay_kind_is_vehicle(kind: FactoryOverlayKind) -> bool {
    matches!(
        kind,
        FactoryOverlayKind::VehicleExhaust
            | FactoryOverlayKind::VehicleTank
            | FactoryOverlayKind::VehicleVent
            | FactoryOverlayKind::VehicleBulb
            | FactoryOverlayKind::VehicleLight0
            | FactoryOverlayKind::VehicleLight1
            | FactoryOverlayKind::VehicleSpin
    )
}

pub(crate) fn overlay_should_be_visible(
    kind: FactoryOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    production_active: bool,
) -> bool {
    if stats.destroyed() || !overlay_kind_is_vehicle(kind) {
        return false;
    }

    if owner == TeamType::Null || !production_active {
        return matches!(kind, FactoryOverlayKind::VehicleTank);
    }

    true
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
            destroyed_asset_path(PlanetType::Volcanic),
            "buildings/vehicle/base_destroyed_volcanic.png"
        );
    }

    #[test]
    fn overlay_visibility_matches_original_owner_building_and_destroyed_rules() {
        let live = ObjectStats::from_kind(ObjectKind::Building(BuildingType::VehicleFactory), 100);
        let destroyed =
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::VehicleFactory), 0);

        assert!(overlay_should_be_visible(
            FactoryOverlayKind::VehicleTank,
            TeamType::Red,
            live,
            false
        ));
        assert!(!overlay_should_be_visible(
            FactoryOverlayKind::VehicleVent,
            TeamType::Red,
            live,
            false
        ));
        assert!(overlay_should_be_visible(
            FactoryOverlayKind::VehicleVent,
            TeamType::Red,
            live,
            true
        ));
        assert!(!overlay_should_be_visible(
            FactoryOverlayKind::VehicleTank,
            TeamType::Red,
            destroyed,
            true
        ));
        assert!(!overlay_should_be_visible(
            FactoryOverlayKind::RobotBody,
            TeamType::Red,
            live,
            true
        ));
    }
}
