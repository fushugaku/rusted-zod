use bevy::prelude::Vec2;

use crate::{
    components::ProductionLevel,
    original::objects::{ObjectKind, RobotType},
    original::types::PlanetType,
};

use super::ProductionPlacement;
use super::{BuildingDeathProfile, BuildingEffectBox};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/fort/fort_{}_back_destroyed.png",
        super::planet_asset_name(planet)
    )
}

pub(crate) fn default_production_unit() -> ObjectKind {
    ObjectKind::Robot(RobotType::Grunt)
}

pub(crate) fn default_build_list(level: ProductionLevel) -> Vec<ObjectKind> {
    super::fort_front::default_build_list(level)
}

pub(crate) fn production_placement() -> ProductionPlacement {
    ProductionPlacement {
        create_offset: Vec2::new(80.0, 32.0),
        move_offset: Vec2::new(80.0, -16.0),
    }
}

pub(crate) fn death_profile(_planet: PlanetType) -> BuildingDeathProfile {
    BuildingDeathProfile {
        effect_box: BuildingEffectBox {
            x: 18.0,
            y: 18.0,
            width: 136,
            height: 118,
        },
        width_pix: 160.0,
        height_pix: 176.0,
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
                height_pix: 176.0,
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
    }

    #[test]
    fn destroyed_asset_matches_original_path() {
        assert_eq!(
            destroyed_asset_path(PlanetType::City),
            "buildings/fort/fort_city_back_destroyed.png"
        );
    }
}
