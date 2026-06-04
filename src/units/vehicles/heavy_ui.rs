use crate::{
    components::DamageMissileVisual,
    original::{
        objects::VehicleType,
        types::{PlanetType, TeamType},
    },
    units::vehicles::{
        self, VehicleDeathEffectBounds, VehicleDeathWreckProfile, VehicleDestroyedAssetProfile,
        VehicleTrackKind, VehicleTurrentProfile,
    },
};

pub(crate) fn movement_profile() -> vehicles::VehicleMovementProfile {
    vehicles::VehicleMovementProfile {
        base_frame_count: 3,
        base_frame_time: vehicles::VEHICLE_BASE_FRAME_TIME,
        track_kind: VehicleTrackKind::Tank,
        drops_damage_effects: true,
    }
}

pub(crate) fn damage_missile_visual() -> Option<DamageMissileVisual> {
    Some(DamageMissileVisual::LightRocket {
        extra_small: 0,
        extra_large: 1,
        xx_large: 1,
    })
}

pub(crate) fn damage_missile_frame_paths() -> Vec<String> {
    vehicles::light::damage_missile_frame_paths()
}

pub(crate) fn death_effect_bounds() -> VehicleDeathEffectBounds {
    VehicleDeathEffectBounds {
        x: 8,
        y: 8,
        width: 16,
        height: 16,
    }
}

pub(crate) fn death_profile() -> vehicles::VehicleDeathProfile {
    vehicles::VehicleDeathProfile {
        effect_bounds: death_effect_bounds(),
        wreck: VehicleDeathWreckProfile::DamagedFrames,
        destroyed_asset: VehicleDestroyedAssetProfile::None,
        turrent: turrent_profile(),
    }
}

pub(crate) fn death_wreck_asset_path(
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> Option<String> {
    vehicles::damaged_wreck_asset_path(VehicleType::Heavy, team, rotation, frame)
}

pub(crate) fn destroyed_asset_path(_team: TeamType) -> Option<String> {
    None
}

pub(crate) fn turrent_profile() -> Option<VehicleTurrentProfile> {
    Some(VehicleTurrentProfile {
        frame_count: 8,
        team_colored: true,
    })
}

pub(crate) fn turrent_frame_paths(team: TeamType) -> Option<Vec<String>> {
    if team == TeamType::Null {
        return None;
    }
    let team = team.atlas_team().asset_name();
    Some(
        (0..8)
            .map(|frame| format!("units/vehicles/heavy/top_pop_{team}_n{frame:02}.png"))
            .collect(),
    )
}

pub(crate) fn track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicles::tank_track_effect_frame_paths(planet, direction)
}
