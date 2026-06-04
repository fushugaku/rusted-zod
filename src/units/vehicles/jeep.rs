use bevy::prelude::Vec2;

use crate::{
    original::{
        settings::UnitSettings,
    },
    units::{
        UnitAttackSound, run_time,
        vehicles::{self, jeep_ui},
    },
};

pub(crate) use jeep_ui::{
    death_effect_bounds, death_profile, death_wreck_asset_path, destroyed_asset_path,
    movement_profile, track_effect_frame_paths,
};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 17.0,
        attack_radius: 120.0,
        attack_damage: 0.0027067,
        attack_damage_chance: 0.65,
        attack_damage_radius: 0.0,
        attack_missile_speed: 0.0,
        attack_speed: 0.1,
        attack_snipe_chance: 0.4,
        health_ratio: 13.0 / 74.0,
        build_time: 81.0,
        max_run_time: run_time(120.0, 17.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Jeep)
}

pub(crate) fn damage_profile() -> vehicles::VehicleDamageProfile {
    vehicles::VehicleDamageProfile {
        missile_visual: None,
        missile_frames: None,
        impact: None,
    }
}
