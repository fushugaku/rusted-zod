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
    "jeep"
}

pub(crate) fn movement_profile() -> vehicles::VehicleMovementProfile {
    vehicles::VehicleMovementProfile {
        base_frame_count: 2,
        base_frame_time: vehicles::VEHICLE_BASE_FRAME_TIME,
        track_kind: VehicleTrackKind::Jeep,
        drops_damage_effects: true,
    }
}

pub(crate) fn base_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> VehicleAtlasFrameSpec {
    vehicles::vehicle_ui::base_atlas_frame_spec_for_folder(
        "jeep",
        movement_profile().base_frame_count,
        team,
        rotation,
        frame,
    )
}

pub(crate) fn top_atlas_frame_spec(
    _team: TeamType,
    _rotation: u16,
) -> Option<VehicleAtlasFrameSpec> {
    None
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
        y: 14,
        width: 22,
        height: 10,
    }
}

pub(crate) fn death_profile() -> vehicles::VehicleDeathProfile {
    vehicles::VehicleDeathProfile {
        effect_bounds: death_effect_bounds(),
        wreck: VehicleDeathWreckProfile::Static,
        destroyed_asset: VehicleDestroyedAssetProfile::Static,
        turrent: None,
    }
}

pub(crate) fn death_wreck_asset_path() -> String {
    "units/vehicles/jeep/wasted.png".to_string()
}

pub(crate) fn destroyed_asset_path(_team: TeamType) -> Option<String> {
    Some(death_wreck_asset_path())
}

pub(crate) fn track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicles::jeep_track_effect_frame_paths(planet, direction)
}
