use bevy::prelude::Vec2;

use crate::{
    components::ObjectStats,
    original::types::{PlanetType, TeamType},
    render::atlas::RadarOverlayKind,
};

use super::{BuildingDeathProfile, BuildingEffectBox};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/radar/base_destroyed_{}.png",
        super::planet_asset_name(planet)
    )
}

pub(crate) fn overlay_should_be_visible(
    kind: RadarOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
) -> bool {
    !stats.destroyed() && (owner != TeamType::Null || matches!(kind, RadarOverlayKind::FrontLight))
}

pub(crate) fn death_profile() -> BuildingDeathProfile {
    BuildingDeathProfile {
        effect_box: BuildingEffectBox {
            x: 1.0,
            y: 6.0,
            width: 44,
            height: 30,
        },
        width_pix: 64.0,
        height_pix: 48.0,
        max_effects_base: 6,
        max_effects_random: 3,
        fireball_base: 4,
        fireball_random: 3,
        piece_base: 3,
        piece_random: 3,
        piece_variants: 2,
        piece_flight_base: 1.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{BuildingType, ObjectKind};

    #[test]
    fn death_profile_matches_original_do_death_effect() {
        assert_eq!(
            death_profile(),
            BuildingDeathProfile {
                effect_box: BuildingEffectBox {
                    x: 1.0,
                    y: 6.0,
                    width: 44,
                    height: 30
                },
                width_pix: 64.0,
                height_pix: 48.0,
                max_effects_base: 6,
                max_effects_random: 3,
                fireball_base: 4,
                fireball_random: 3,
                piece_base: 3,
                piece_random: 3,
                piece_variants: 2,
                piece_flight_base: 1.5
            }
        );
    }

    #[test]
    fn destroyed_asset_matches_original_path() {
        assert_eq!(
            destroyed_asset_path(PlanetType::Arctic),
            "buildings/radar/base_destroyed_arctic.png"
        );
    }

    #[test]
    fn overlay_visibility_matches_original_owner_and_destroyed_rules() {
        let live = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 100);
        let destroyed = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 0);

        assert!(overlay_should_be_visible(
            RadarOverlayKind::FrontLight,
            TeamType::Null,
            live
        ));
        assert!(!overlay_should_be_visible(
            RadarOverlayKind::Dish,
            TeamType::Null,
            live
        ));
        assert!(overlay_should_be_visible(
            RadarOverlayKind::BoxSpinner,
            TeamType::Red,
            live
        ));
        assert!(!overlay_should_be_visible(
            RadarOverlayKind::FrontLight,
            TeamType::Red,
            destroyed
        ));
    }
}
