use bevy::prelude::Vec2;

use crate::{
    original::settings::UnitSettings,
    units::{
        RocketImpactProfile, UnitAttackSound, run_time,
        vehicles::{self, VehicleMissileFrameProfile, light_ui},
    },
};

pub(crate) use light_ui::{
    damage_missile_frame_paths, damage_missile_visual, death_effect_bounds, death_profile,
    death_wreck_asset_path, destroyed_asset_path, init_fire_frame_path, movement_profile,
    track_effect_frame_paths, turrent_frame_paths, turrent_profile,
};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 14.0,
        attack_radius: 120.0,
        attack_damage: 50.0 / 240.0,
        attack_damage_chance: 0.0,
        attack_damage_radius: 40.0,
        attack_missile_speed: 225.0,
        attack_speed: 1.128,
        attack_snipe_chance: 0.0,
        health_ratio: 25.0 / 74.0,
        build_time: 137.0,
        max_run_time: run_time(120.0, 14.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Light)
}

pub(crate) fn damage_profile() -> vehicles::VehicleDamageProfile {
    vehicles::VehicleDamageProfile {
        missile_visual: damage_missile_visual(),
        missile_frames: Some(VehicleMissileFrameProfile::LightRocketBullet),
        impact: Some(rocket_impact_profile(0, 0, 0)),
    }
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
