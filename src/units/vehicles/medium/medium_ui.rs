use bevy::prelude::Vec2;

use crate::{
    components::DamageMissileVisual,
    original::types::{PlanetType, TeamType},
    units::{
        RocketImpactProfile,
        vehicles::{
            self, VehicleDamageProfile, VehicleDeathEffectBounds, VehicleDeathWreckProfile,
            VehicleDestroyedAssetProfile, VehicleMissileFrameProfile, VehicleTrackKind,
            VehicleTurrentProfile, vehicle_ui::VehicleAtlasFrameSpec,
        },
    },
};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn hud_name() -> &'static str {
    "medium"
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
        "medium",
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
        "medium", rotation,
    ))
}

pub(crate) fn lid_top_left_offset(body_direction: usize, _turrent_direction: usize) -> Vec2 {
    const UNIT_Y: [f32; 8] = [6.0, 0.0, 5.0, 0.0, 6.0, 0.0, 5.0, 0.0];
    const TURRENT_X: [f32; 8] = [0.0, 0.0, -1.0, -2.0, 0.0, 0.0, -1.0, -2.0];
    const TURRENT_Y: [f32; 8] = [0.0, 6.0, 0.0, 6.0, 0.0, 6.0, 0.0, 6.0];
    let body_direction = body_direction.min(7);
    Vec2::new(
        TURRENT_X[body_direction] + 12.0,
        TURRENT_Y[body_direction] - 5.0 + UNIT_Y[body_direction],
    )
}

pub(crate) fn damage_missile_visual() -> Option<DamageMissileVisual> {
    Some(DamageMissileVisual::LightRocket {
        extra_small: 0,
        extra_large: 1,
        xx_large: 0,
    })
}

#[cfg(test)]
pub(crate) fn damage_missile_frame_paths() -> Vec<String> {
    vehicles::light_ui::damage_missile_frame_paths()
}

pub(crate) fn rocket_impact_profile() -> RocketImpactProfile {
    vehicles::light_ui::rocket_impact_profile(0, 1, 0)
}

pub(crate) fn damage_profile() -> VehicleDamageProfile {
    VehicleDamageProfile {
        missile_visual: damage_missile_visual(),
        missile_frames: Some(VehicleMissileFrameProfile::LightRocketBullet),
        impact: Some(rocket_impact_profile()),
    }
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
    if team == TeamType::Null {
        return None;
    }
    let team_name = team.atlas_team().asset_name();
    let (rotation, frame) = vehicles::death_damaged_frame(rotation, frame);
    Some(format!(
        "units/vehicles/medium/base_damaged_{team_name}_r{rotation:03}_n{frame:02}.png"
    ))
}

pub(crate) fn destroyed_asset_path(_team: TeamType) -> Option<String> {
    None
}

pub(crate) fn turrent_profile() -> Option<VehicleTurrentProfile> {
    Some(VehicleTurrentProfile {
        frame_count: 8,
        team_colored: false,
        damage: 40,
        radius: 40,
    })
}

pub(crate) fn turrent_frame_paths(_team: TeamType) -> Option<Vec<String>> {
    Some(
        (0..8)
            .map(|frame| format!("units/vehicles/medium/top_pop_n{frame:02}.png"))
            .collect(),
    )
}

pub(crate) fn track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicles::tank_track_effect_frame_paths(planet, direction)
}
