use bevy::prelude::Vec2;

use crate::{
    components::ObjectStats,
    original::types::{PlanetType, TeamType},
    render::atlas::RepairOverlayKind,
};

use crate::units::buildings::{
    BuildingAtlasFrameSpec, BuildingDeathProfile, BuildingEffectBox, BuildingOverlayFrameSpec,
    BuildingPathingRect, BuildingPathingSpec, building_ui::planet_asset_name,
};

pub(crate) const OVERLAY_FRAME_TIME: f32 = 0.35;
pub(crate) const OVERLAY_KINDS: [RepairOverlayKind; 5] = [
    RepairOverlayKind::TextBox,
    RepairOverlayKind::Bulb,
    RepairOverlayKind::SmokeStack,
    RepairOverlayKind::FrontLight,
    RepairOverlayKind::SideLight,
];

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn pathing_spec() -> BuildingPathingSpec {
    BuildingPathingSpec {
        blocked_rects: vec![BuildingPathingRect {
            dx: 0,
            dy: 0,
            width: 5,
            height: 4,
        }],
        blocked_masks: Vec::new(),
        unblocked_tiles: Vec::new(),
    }
}

pub(crate) fn crane_repair_local_points(size: Vec2) -> (Vec2, Vec2) {
    (Vec2::new(32.0, size.y + 32.0), Vec2::new(32.0, 32.0))
}

pub(crate) fn unit_repair_local_points(size: Vec2) -> (Vec2, Vec2) {
    (Vec2::new(32.0, size.y + 32.0), Vec2::new(32.0, 32.0))
}

pub(crate) fn atlas_layer_specs(team: TeamType, planet: PlanetType) -> Vec<BuildingAtlasFrameSpec> {
    let team = team.atlas_team();
    vec![
        BuildingAtlasFrameSpec {
            atlas_team: TeamType::Red,
            frame_name: base_atlas_frame_name(planet),
            world_offset: Vec2::ZERO,
            animation_frame_names: Vec::new(),
        },
        BuildingAtlasFrameSpec {
            atlas_team: team,
            frame_name: team_overlay_frame_name(team),
            world_offset: team_overlay_world_offset(),
            animation_frame_names: Vec::new(),
        },
    ]
}

pub(crate) fn base_atlas_frame_name(planet: PlanetType) -> String {
    format!("building_repair_base_{}", planet_asset_name(planet))
}

pub(crate) fn team_overlay_frame_name(team: TeamType) -> String {
    format!("building_repair_{}", team.atlas_team().asset_name())
}

pub(crate) fn team_overlay_world_offset() -> Vec2 {
    Vec2::new(0.0, 48.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/repair/base_destroyed_{}.png",
        planet_asset_name(planet)
    )
}

pub(crate) fn overlay_frame_specs(kind: RepairOverlayKind) -> Vec<BuildingOverlayFrameSpec> {
    overlay_frame_names(kind)
        .into_iter()
        .zip(overlay_offsets(kind))
        .map(|(frame_name, world_offset)| BuildingOverlayFrameSpec {
            frame_name,
            world_offset,
        })
        .collect()
}

pub(crate) fn overlay_frame_names(kind: RepairOverlayKind) -> Vec<String> {
    let (stem, count) = match kind {
        RepairOverlayKind::FrontLight => ("front_light", 2),
        RepairOverlayKind::SideLight => ("side_light", 2),
        RepairOverlayKind::Bulb => ("bulb", 2),
        RepairOverlayKind::SmokeStack => ("smoke_stack", 5),
        RepairOverlayKind::TextBox => ("text_box", 3),
    };

    (0..count)
        .map(|frame| format!("building_repair_{stem}_{frame}"))
        .collect()
}

pub(crate) fn overlay_offsets(kind: RepairOverlayKind) -> Vec<Vec2> {
    match kind {
        RepairOverlayKind::FrontLight => vec![Vec2::new(6.0, 16.0); 2],
        RepairOverlayKind::SideLight => vec![Vec2::new(18.0, 6.0); 2],
        RepairOverlayKind::Bulb => vec![Vec2::new(32.0, 0.0); 2],
        RepairOverlayKind::SmokeStack => vec![Vec2::new(61.0, 0.0); 5],
        RepairOverlayKind::TextBox => vec![Vec2::new(16.0, 32.0); 3],
    }
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

pub(crate) fn overlay_initial_frame(
    kind: RepairOverlayKind,
    owner: TeamType,
    repairing_unit: bool,
) -> usize {
    overlay_forced_frame(kind, owner, repairing_unit).unwrap_or(0)
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
    fn atlas_layer_specs_match_original_base_and_team_overlay() {
        let specs = atlas_layer_specs(TeamType::Green, PlanetType::Volcanic);

        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].atlas_team, TeamType::Red);
        assert_eq!(specs[0].frame_name, "building_repair_base_volcanic");
        assert_eq!(specs[1].atlas_team, TeamType::Green);
        assert_eq!(specs[1].frame_name, "building_repair_green");
        assert_eq!(specs[1].world_offset, Vec2::new(0.0, 48.0));
    }

    #[test]
    fn overlay_frame_specs_match_original_assets_and_offsets() {
        assert_eq!(
            OVERLAY_KINDS,
            [
                RepairOverlayKind::TextBox,
                RepairOverlayKind::Bulb,
                RepairOverlayKind::SmokeStack,
                RepairOverlayKind::FrontLight,
                RepairOverlayKind::SideLight
            ]
        );
        assert_eq!(OVERLAY_FRAME_TIME, 0.35);
        assert_eq!(
            overlay_frame_names(RepairOverlayKind::TextBox),
            vec![
                "building_repair_text_box_0".to_string(),
                "building_repair_text_box_1".to_string(),
                "building_repair_text_box_2".to_string()
            ]
        );
        assert_eq!(
            overlay_offsets(RepairOverlayKind::SmokeStack),
            vec![Vec2::new(61.0, 0.0); 5]
        );

        let specs = overlay_frame_specs(RepairOverlayKind::Bulb);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].frame_name, "building_repair_bulb_0");
        assert_eq!(specs[0].world_offset, Vec2::new(32.0, 0.0));
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
        assert_eq!(
            overlay_initial_frame(RepairOverlayKind::FrontLight, TeamType::Red, true),
            1
        );
        assert_eq!(
            overlay_initial_frame(RepairOverlayKind::TextBox, TeamType::Red, true),
            0
        );
    }
}
