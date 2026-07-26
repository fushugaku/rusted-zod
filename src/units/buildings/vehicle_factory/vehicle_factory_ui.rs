use bevy::prelude::Vec2;

use crate::{
    components::ObjectStats,
    original::types::{PlanetType, TeamType},
    render::atlas::FactoryOverlayKind,
};

use crate::units::buildings::{
    BuildingAtlasFrameSpec, BuildingDeathProfile, BuildingEffectBox, BuildingOverlayFrameSpec,
    BuildingPathingRect, BuildingPathingSpec, ProductionPlacement, building_ui::planet_asset_name,
};

#[cfg(test)]
pub(crate) const OVERLAY_FRAME_TIME: f32 = 0.25;
pub(crate) const OVERLAY_KINDS: [FactoryOverlayKind; 7] = [
    FactoryOverlayKind::VehicleExhaust,
    FactoryOverlayKind::VehicleTank,
    FactoryOverlayKind::VehicleVent,
    FactoryOverlayKind::VehicleBulb,
    FactoryOverlayKind::VehicleLight0,
    FactoryOverlayKind::VehicleLight1,
    FactoryOverlayKind::VehicleSpin,
];

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn pathing_spec() -> BuildingPathingSpec {
    BuildingPathingSpec {
        blocked_rects: vec![BuildingPathingRect {
            dx: 0,
            dy: 0,
            width: 4,
            height: 5,
        }],
        blocked_masks: Vec::new(),
        unblocked_tiles: Vec::new(),
    }
}

pub(crate) fn crane_repair_local_points(size: Vec2) -> (Vec2, Vec2) {
    (Vec2::new(31.0, size.y + 32.0), Vec2::new(31.0, 32.0))
}

pub(crate) fn production_placement() -> ProductionPlacement {
    ProductionPlacement {
        create_offset: Vec2::new(32.0, 48.0),
        move_offset: Vec2::new(32.0, 96.0),
    }
}

pub(crate) fn production_label_asset_path() -> &'static str {
    "other/production_gui/vehicle_factory_label.png"
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
    format!("building_vehicle_base_{}", planet_asset_name(planet))
}

pub(crate) fn team_overlay_frame_name(team: TeamType) -> String {
    format!("building_vehicle_{}", team.atlas_team().asset_name())
}

pub(crate) fn team_overlay_world_offset() -> Vec2 {
    Vec2::new(32.0, 48.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/vehicle/base_destroyed_{}.png",
        planet_asset_name(planet)
    )
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

pub(crate) fn overlay_frame_specs(kind: FactoryOverlayKind) -> Vec<BuildingOverlayFrameSpec> {
    overlay_frame_names(kind)
        .into_iter()
        .zip(overlay_offsets(kind))
        .map(|(frame_name, world_offset)| BuildingOverlayFrameSpec {
            frame_name,
            world_offset,
        })
        .collect()
}

pub(crate) fn overlay_frame_names(kind: FactoryOverlayKind) -> Vec<String> {
    match kind {
        FactoryOverlayKind::VehicleExhaust => {
            (0..13).map(|frame| format!("exhaust_{frame}")).collect()
        }
        FactoryOverlayKind::VehicleTank => (0..2)
            .map(|frame| format!("building_vehicle_tank_{frame}"))
            .collect(),
        FactoryOverlayKind::VehicleVent => (0..4)
            .map(|frame| format!("building_vehicle_vent_{frame}"))
            .collect(),
        FactoryOverlayKind::VehicleBulb => (0..2)
            .map(|frame| format!("building_vehicle_bulb_{frame}"))
            .collect(),
        FactoryOverlayKind::VehicleLight0 | FactoryOverlayKind::VehicleLight1 => {
            vec!["building_vehicle_lights_1".to_string()]
        }
        FactoryOverlayKind::VehicleSpin => (0..8)
            .map(|frame| format!("building_vehicle_spin_{frame}"))
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn overlay_offsets(kind: FactoryOverlayKind) -> Vec<Vec2> {
    match kind {
        FactoryOverlayKind::VehicleExhaust => (0..13)
            .map(|frame| Vec2::new(28.0, -22.0 - frame as f32 * 2.0))
            .collect(),
        FactoryOverlayKind::VehicleTank => vec![Vec2::new(16.0, 48.0); 2],
        FactoryOverlayKind::VehicleVent => vec![Vec2::new(16.0, 32.0); 4],
        FactoryOverlayKind::VehicleBulb => vec![Vec2::new(24.0, 39.0); 2],
        FactoryOverlayKind::VehicleLight0 => vec![Vec2::new(13.0, 47.0)],
        FactoryOverlayKind::VehicleLight1 => vec![Vec2::new(42.0, 47.0)],
        FactoryOverlayKind::VehicleSpin => vec![Vec2::new(9.0, -2.0); 8],
        _ => Vec::new(),
    }
}

pub(crate) fn overlay_initial_frame_visible(kind: FactoryOverlayKind) -> bool {
    overlay_kind_is_vehicle(kind)
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
    use crate::original::objects::{BuildingType, ObjectKind};

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
    fn atlas_layer_and_production_label_specs_match_original_assets() {
        assert_eq!(
            production_label_asset_path(),
            "other/production_gui/vehicle_factory_label.png"
        );

        let specs = atlas_layer_specs(TeamType::Yellow, PlanetType::Arctic);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].atlas_team, TeamType::Red);
        assert_eq!(specs[0].frame_name, "building_vehicle_base_arctic");
        assert_eq!(specs[1].atlas_team, TeamType::Yellow);
        assert_eq!(specs[1].frame_name, "building_vehicle_yellow");
        assert_eq!(specs[1].world_offset, Vec2::new(32.0, 48.0));
    }

    #[test]
    fn overlay_frame_specs_match_original_assets_and_offsets() {
        assert_eq!(OVERLAY_KINDS.len(), 7);
        assert_eq!(OVERLAY_FRAME_TIME, 0.25);
        assert_eq!(
            overlay_frame_names(FactoryOverlayKind::VehicleVent).len(),
            4
        );
        assert_eq!(
            overlay_frame_names(FactoryOverlayKind::VehicleLight1),
            vec!["building_vehicle_lights_1".to_string()]
        );
        assert_eq!(
            overlay_offsets(FactoryOverlayKind::VehicleExhaust)
                .last()
                .copied(),
            Some(Vec2::new(28.0, -46.0))
        );
        assert_eq!(
            overlay_offsets(FactoryOverlayKind::VehicleLight1),
            vec![Vec2::new(42.0, 47.0)]
        );
        assert!(overlay_initial_frame_visible(
            FactoryOverlayKind::VehicleLight1
        ));
        assert!(!overlay_initial_frame_visible(
            FactoryOverlayKind::RobotSingleLight0
        ));

        let specs = overlay_frame_specs(FactoryOverlayKind::VehicleBulb);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].frame_name, "building_vehicle_bulb_0");
        assert_eq!(specs[0].world_offset, Vec2::new(24.0, 39.0));
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
