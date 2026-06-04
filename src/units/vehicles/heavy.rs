use bevy::prelude::Vec2;

use crate::{
    original::settings::UnitSettings,
    units::{
        RocketImpactProfile, UnitAttackSound, run_time,
        vehicles::{self, VehicleMissileFrameProfile, heavy_ui},
    },
};

pub(crate) use heavy_ui::{
    damage_missile_frame_paths, damage_missile_visual, death_effect_bounds, death_profile,
    death_wreck_asset_path, destroyed_asset_path, movement_profile, track_effect_frame_paths,
    turrent_frame_paths, turrent_profile,
};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 9.0,
        attack_radius: 144.0,
        attack_damage: 120.0 / 240.0,
        attack_damage_chance: 0.0,
        attack_damage_radius: 50.0,
        attack_missile_speed: 135.0,
        attack_speed: 4.088,
        attack_snipe_chance: 0.0,
        health_ratio: 62.0 / 74.0,
        build_time: 309.0,
        max_run_time: run_time(144.0, 9.0) * 0.7,
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Heavy)
}

pub(crate) fn damage_profile() -> vehicles::VehicleDamageProfile {
    vehicles::VehicleDamageProfile {
        missile_visual: damage_missile_visual(),
        missile_frames: Some(VehicleMissileFrameProfile::LightRocketBullet),
        impact: Some(rocket_impact_profile()),
    }
}

pub(crate) fn rocket_impact_profile() -> RocketImpactProfile {
    vehicles::light::rocket_impact_profile(0, 1, 1)
}
