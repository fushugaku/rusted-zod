use bevy::prelude::Vec2;

use crate::{
    original::{
        objects::{ObjectKind, RobotType},
        settings::UnitSettings,
    },
    units::{
        UnitAttackSound, run_time,
        vehicles::{self, apc_ui},
    },
};

pub(crate) use apc_ui::{
    death_effect_bounds, death_profile, death_wreck_asset_path, destroyed_asset_path,
    movement_profile, track_effect_frame_paths,
};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 14.0,
        attack_radius: 0.0,
        attack_damage: 0.0,
        attack_damage_chance: 0.0,
        attack_damage_radius: 0.0,
        attack_missile_speed: 0.0,
        attack_speed: 0.0,
        attack_snipe_chance: 0.0,
        health_ratio: 50.0 / 74.0,
        build_time: 118.0,
        max_run_time: run_time(120.0, 14.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    None
}

pub(crate) fn damage_profile() -> vehicles::VehicleDamageProfile {
    vehicles::VehicleDamageProfile {
        missile_visual: None,
        missile_frames: None,
        impact: None,
    }
}

pub(crate) fn driver_attack_sound(effective_kind: ObjectKind) -> Option<UnitAttackSound> {
    match effective_kind {
        ObjectKind::Robot(RobotType::Grunt)
        | ObjectKind::Robot(RobotType::Psycho)
        | ObjectKind::Robot(RobotType::Sniper) => Some(UnitAttackSound::Rifle),
        ObjectKind::Robot(RobotType::Pyro) => Some(UnitAttackSound::Pyro),
        ObjectKind::Robot(RobotType::Laser) => Some(UnitAttackSound::Laser),
        ObjectKind::Robot(RobotType::Tough) => Some(UnitAttackSound::Tough),
        _ => None,
    }
}
