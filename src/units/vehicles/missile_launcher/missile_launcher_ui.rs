use bevy::prelude::Vec2;

use crate::{
    components::{DamageCrater, DamageMissileVisual},
    original::types::{PlanetType, TeamType},
    units::{
        DamageMissileVisualGeometry, RocketImpactProfile,
        vehicles::{
            self, VehicleDamageProfile, VehicleDeathEffectBounds, VehicleDeathWreckProfile,
            VehicleDestroyedAssetProfile, VehicleMissileFrameProfile, VehicleTrackKind,
            vehicle_ui::VehicleAtlasFrameSpec,
        },
    },
};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn hud_name() -> &'static str {
    "missile_launcher"
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
        "missile_launcher",
        movement_profile().base_frame_count,
        team,
        rotation,
        frame,
    )
}

pub(crate) fn top_atlas_frame_spec(team: TeamType, rotation: u16) -> Option<VehicleAtlasFrameSpec> {
    Some(vehicles::vehicle_ui::team_top_atlas_frame_spec(
        "missile_launcher",
        team,
        rotation,
    ))
}

pub(crate) fn damage_missile_visual() -> Option<DamageMissileVisual> {
    Some(DamageMissileVisual::MissileLauncher)
}

pub(crate) fn damage_missile_frame_paths() -> Vec<String> {
    vec!["units/vehicles/missile_launcher/bullet.png".to_string()]
}

pub(crate) fn rocket_impact_profile() -> RocketImpactProfile {
    RocketImpactProfile {
        xx_large_mushrooms: 3,
        large_mushrooms: 0,
        small_mushrooms: 2,
        unit_particle_radius: 80.0,
        unit_particle_amount: 23,
    }
}

pub(crate) fn damage_crater() -> DamageCrater {
    DamageCrater {
        is_big: true,
        chance: 0.75,
        big_chance: None,
    }
}

pub(crate) fn left_offset(direction: Vec2) -> Vec2 {
    Vec2::new(-direction.y * 8.0, direction.x * 8.0)
}

pub(crate) fn right_offset(direction: Vec2) -> Vec2 {
    Vec2::new(direction.y * 8.0, -direction.x * 8.0)
}

pub(crate) fn visual_geometry(direction: Vec2) -> DamageMissileVisualGeometry {
    let left = left_offset(direction);
    let right = right_offset(direction);
    DamageMissileVisualGeometry {
        primary_offset: Vec2::ZERO,
        replica_offsets: vec![left, right],
        smoke_offsets: vec![Vec2::ZERO, left, right],
    }
}

pub(crate) fn damage_profile() -> VehicleDamageProfile {
    VehicleDamageProfile {
        missile_visual: damage_missile_visual(),
        missile_frames: Some(VehicleMissileFrameProfile::MissileLauncherBullet),
        impact: Some(rocket_impact_profile()),
    }
}

pub(crate) fn death_effect_bounds() -> VehicleDeathEffectBounds {
    VehicleDeathEffectBounds {
        x: 5,
        y: 10,
        width: 21,
        height: 19,
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
    "units/vehicles/missile_launcher/wasted.png".to_string()
}

pub(crate) fn destroyed_asset_path(team: TeamType) -> Option<String> {
    let team = team.atlas_team().asset_name();
    Some(format!("units/vehicles/missile_launcher/wasted_{team}.png"))
}

pub(crate) fn track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicles::tank_track_effect_frame_paths(planet, direction)
}
