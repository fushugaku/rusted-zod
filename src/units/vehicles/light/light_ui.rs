use bevy::prelude::Vec2;

use crate::{
    components::{DamageCrater, DamageMissileVisual},
    original::types::{PlanetType, TeamType},
    units::{
        DamageMissileLaunchEffectProfile, RocketImpactProfile,
        vehicles::{
            self, VehicleDamageProfile, VehicleDeathEffectBounds, VehicleDeathWreckProfile,
            VehicleDestroyedAssetProfile, VehicleMissileFrameProfile, VehicleTrackKind,
            VehicleTurrentProfile, vehicle_ui::VehicleAtlasFrameSpec,
        },
    },
};

pub(crate) const INIT_FIRE_FRAME_COUNT: usize = 4;
pub(crate) const INIT_FIRE_FRAME_TIME: f32 = 0.02;
pub(crate) const INIT_FIRE_TOP_LEFT_OFFSET: Vec2 = Vec2::new(-8.0, -7.0);
pub(crate) const INIT_FIRE_Z: f32 = 34.8;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn hud_name() -> &'static str {
    "light"
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
        "light",
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
        "light", rotation,
    ))
}

pub(crate) fn lid_top_left_offset(body_direction: usize, turrent_direction: usize) -> Vec2 {
    const TURRENT_X: [f32; 8] = [2.0, 0.0, -2.0, 0.0, 2.0, 0.0, -2.0, 0.0];
    const LID_SHIFT_X: [f32; 8] = [11.0, 11.0, 12.0, 12.0, 12.0, 12.0, 12.0, 11.0];
    const LID_SHIFT_Y: [f32; 8] = [3.0, 4.0, 5.0, 4.0, 3.0, 3.0, 4.0, 3.0];
    let body_direction = body_direction.min(7);
    let turrent_direction = turrent_direction.min(7);
    Vec2::new(
        TURRENT_X[body_direction] + LID_SHIFT_X[turrent_direction],
        LID_SHIFT_Y[turrent_direction],
    )
}

pub(crate) fn damage_missile_visual() -> Option<DamageMissileVisual> {
    Some(DamageMissileVisual::LightRocket {
        extra_small: 0,
        extra_large: 0,
        xx_large: 0,
    })
}

pub(crate) fn damage_missile_frame_paths() -> Vec<String> {
    vec!["units/vehicles/light/bullet.png".to_string()]
}

pub(crate) fn rocket_impact_profile(
    extra_small: u8,
    extra_large: u8,
    xx_large: u8,
) -> RocketImpactProfile {
    RocketImpactProfile {
        xx_large_mushrooms: xx_large,
        large_mushrooms: extra_large,
        small_mushrooms: extra_small,
        unit_particle_radius: 40.0,
        unit_particle_amount: 7
            + usize::from(extra_small) * 2
            + usize::from(extra_large) * 3
            + usize::from(xx_large) * 4,
    }
}

pub(crate) fn big_crater_chance(extra_large: u8, xx_large: u8) -> Option<f32> {
    if xx_large > 0 {
        Some(0.35)
    } else if extra_large > 0 {
        Some(0.15)
    } else {
        None
    }
}

pub(crate) fn damage_crater(extra_large: u8, xx_large: u8) -> DamageCrater {
    DamageCrater {
        is_big: false,
        chance: 0.75,
        big_chance: big_crater_chance(extra_large, xx_large),
    }
}

pub(crate) fn init_fire_frame_path(frame: usize) -> String {
    format!(
        "units/vehicles/light/initfire_n{:02}.png",
        frame.min(INIT_FIRE_FRAME_COUNT - 1)
    )
}

pub(crate) fn launch_effect_profile(frame: usize) -> DamageMissileLaunchEffectProfile {
    DamageMissileLaunchEffectProfile {
        frame_path: init_fire_frame_path(frame),
        map_top_left_offset: INIT_FIRE_TOP_LEFT_OFFSET,
        z: INIT_FIRE_Z,
        frame_time: INIT_FIRE_FRAME_TIME,
        name: "light_rocket_init_fire",
    }
}

pub(crate) fn damage_profile() -> VehicleDamageProfile {
    VehicleDamageProfile {
        missile_visual: damage_missile_visual(),
        missile_frames: Some(VehicleMissileFrameProfile::LightRocketBullet),
        impact: Some(rocket_impact_profile(0, 0, 0)),
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
        "units/vehicles/light/base_damaged_{team_name}_r{rotation:03}_n{frame:02}.png"
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
            .map(|frame| format!("units/vehicles/light/top_pop_n{frame:02}.png"))
            .collect(),
    )
}

pub(crate) fn track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicles::tank_track_effect_frame_paths(planet, direction)
}
