use bevy::prelude::Vec2;

use crate::{
    components::ObjectStats,
    original::types::{PlanetType, TeamType},
    render::atlas::RepairOverlayKind,
};

use super::{BuildingDeathProfile, BuildingEffectBox};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/repair/base_destroyed_{}.png",
        super::planet_asset_name(planet)
    )
}

pub(crate) fn overlay_should_be_visible(
    kind: RepairOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    repairing_unit: bool,
) -> bool {
    if stats.destroyed() {
        return false;
    }

    if owner == TeamType::Null {
        return matches!(kind, RepairOverlayKind::SmokeStack);
    }

    match kind {
        RepairOverlayKind::SmokeStack => repairing_unit,
        RepairOverlayKind::FrontLight
        | RepairOverlayKind::SideLight
        | RepairOverlayKind::Bulb
        | RepairOverlayKind::TextBox => true,
    }
}

pub(crate) fn overlay_forced_frame(
    kind: RepairOverlayKind,
    owner: TeamType,
    repairing_unit: bool,
) -> Option<usize> {
    match kind {
        RepairOverlayKind::FrontLight | RepairOverlayKind::SideLight => Some(1),
        RepairOverlayKind::Bulb | RepairOverlayKind::SmokeStack
            if owner == TeamType::Null || !repairing_unit =>
        {
            Some(0)
        }
        _ => None,
    }
}

pub(crate) fn death_profile() -> BuildingDeathProfile {
    BuildingDeathProfile {
        effect_box: BuildingEffectBox {
            x: 8.0,
            y: 8.0,
            width: 56,
            height: 40,
        },
        width_pix: 80.0,
        height_pix: 64.0,
        max_effects_base: 6,
        max_effects_random: 4,
        fireball_base: 6,
        fireball_random: 3,
        piece_base: 4,
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
                    x: 8.0,
                    y: 8.0,
                    width: 56,
                    height: 40
                },
                width_pix: 80.0,
                height_pix: 64.0,
                max_effects_base: 6,
                max_effects_random: 4,
                fireball_base: 6,
                fireball_random: 3,
                piece_base: 4,
                piece_random: 3,
                piece_variants: 2,
                piece_flight_base: 1.5
            }
        );
    }

    #[test]
    fn destroyed_asset_matches_original_path() {
        assert_eq!(
            destroyed_asset_path(PlanetType::Desert),
            "buildings/repair/base_destroyed_desert.png"
        );
    }

    #[test]
    fn overlay_visibility_matches_original_owner_destroyed_and_busy_rules() {
        let live = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Repair), 100);
        let destroyed = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Repair), 0);

        assert!(overlay_should_be_visible(
            RepairOverlayKind::SmokeStack,
            TeamType::Null,
            live,
            false
        ));
        assert!(!overlay_should_be_visible(
            RepairOverlayKind::TextBox,
            TeamType::Null,
            live,
            false
        ));
        assert!(overlay_should_be_visible(
            RepairOverlayKind::TextBox,
            TeamType::Red,
            live,
            false
        ));
        assert!(!overlay_should_be_visible(
            RepairOverlayKind::SmokeStack,
            TeamType::Red,
            live,
            false
        ));
        assert!(overlay_should_be_visible(
            RepairOverlayKind::SmokeStack,
            TeamType::Red,
            live,
            true
        ));
        assert!(!overlay_should_be_visible(
            RepairOverlayKind::FrontLight,
            TeamType::Red,
            destroyed,
            true
        ));
    }

    #[test]
    fn overlay_forced_frames_match_original_after_effects() {
        assert_eq!(
            overlay_forced_frame(RepairOverlayKind::FrontLight, TeamType::Red, true),
            Some(1)
        );
        assert_eq!(
            overlay_forced_frame(RepairOverlayKind::SideLight, TeamType::Red, false),
            Some(1)
        );
        assert_eq!(
            overlay_forced_frame(RepairOverlayKind::Bulb, TeamType::Red, false),
            Some(0)
        );
        assert_eq!(
            overlay_forced_frame(RepairOverlayKind::SmokeStack, TeamType::Null, false),
            Some(0)
        );
        assert_eq!(
            overlay_forced_frame(RepairOverlayKind::TextBox, TeamType::Red, false),
            None
        );
        assert_eq!(
            overlay_forced_frame(RepairOverlayKind::Bulb, TeamType::Red, true),
            None
        );
    }
}
