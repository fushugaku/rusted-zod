use bevy::prelude::Vec2;

use crate::{
    original::settings::UnitSettings,
    units::{
        RocketImpactProfile, UnitAttackSound, run_time,
        vehicles::{self, VehicleMissileFrameProfile, missile_launcher_ui},
    },
};

pub(crate) use missile_launcher_ui::{
    damage_missile_frame_paths, damage_missile_visual, death_effect_bounds, death_profile,
    death_wreck_asset_path, destroyed_asset_path, left_offset, movement_profile, right_offset,
    track_effect_frame_paths,
};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 6.0,
        attack_radius: 160.0,
        attack_damage: 62.0 / 74.0,
        attack_damage_chance: 0.0,
        attack_damage_radius: 80.0,
        attack_missile_speed: 70.0,
        attack_speed: 4.454,
        attack_snipe_chance: 0.0,
        health_ratio: 50.0 / 74.0,
        build_time: 373.0,
        max_run_time: run_time(160.0, 6.0) * 0.5,
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::MobileMissile)
}

pub(crate) fn damage_profile() -> vehicles::VehicleDamageProfile {
    vehicles::VehicleDamageProfile {
        missile_visual: damage_missile_visual(),
        missile_frames: Some(VehicleMissileFrameProfile::MissileLauncherBullet),
        impact: Some(rocket_impact_profile()),
    }
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
