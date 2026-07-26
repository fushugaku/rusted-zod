use bevy::prelude::Vec2;

use crate::{
    original::types::{PlanetType, TeamType},
    units::vehicles::{
        self, VehicleDamageProfile, VehicleDeathEffectBounds, VehicleDeathWreckProfile,
        VehicleDestroyedAssetProfile, VehicleTrackKind, vehicle_ui::VehicleAtlasFrameSpec,
    },
};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn hud_name() -> &'static str {
    "apc"
}

pub(crate) fn movement_profile() -> vehicles::VehicleMovementProfile {
    vehicles::VehicleMovementProfile {
        base_frame_count: 3,
        base_frame_time: vehicles::VEHICLE_BASE_FRAME_TIME,
        track_kind: VehicleTrackKind::Tank,
        drops_damage_effects: true,
    }
}

pub(crate) fn base_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> VehicleAtlasFrameSpec {
    vehicles::vehicle_ui::base_atlas_frame_spec_for_folder(
        "apc",
        movement_profile().base_frame_count,
        team,
        rotation,
        frame,
    )
}

pub(crate) fn top_atlas_frame_spec(
    _team: TeamType,
    rotation: u16,
) -> Option<VehicleAtlasFrameSpec> {
    Some(vehicles::vehicle_ui::red_top_atlas_frame_spec(
        "apc", rotation,
    ))
}

pub(crate) fn damage_profile() -> VehicleDamageProfile {
    VehicleDamageProfile {
        missile_visual: None,
        missile_frames: None,
        impact: None,
    }
}

pub(crate) fn death_effect_bounds() -> VehicleDeathEffectBounds {
    VehicleDeathEffectBounds {
        x: 5,
        y: 8,
        width: 18,
        height: 20,
    }
}

pub(crate) fn death_profile() -> vehicles::VehicleDeathProfile {
    vehicles::VehicleDeathProfile {
        effect_bounds: death_effect_bounds(),
        wreck: VehicleDeathWreckProfile::Static,
        destroyed_asset: VehicleDestroyedAssetProfile::TeamStatic,
        turrent: None,
    }
}

pub(crate) fn death_wreck_asset_path() -> String {
    "units/vehicles/apc/wasted.png".to_string()
}

pub(crate) fn destroyed_asset_path(team: TeamType) -> Option<String> {
    let team = team.atlas_team().asset_name();
    Some(format!("units/vehicles/apc/wasted_{team}.png"))
}

pub(crate) fn track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicles::tank_track_effect_frame_paths(planet, direction)
}
