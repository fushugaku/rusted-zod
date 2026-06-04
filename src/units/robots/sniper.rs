use crate::{
    original::settings::UnitSettings,
    units::{UnitAttackSound, run_time},
};

pub(crate) fn default_selection_size() -> bevy::prelude::Vec2 {
    super::sniper_ui::default_selection_size()
}

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 3,
        move_speed: 14.0,
        attack_radius: 144.0,
        attack_damage: 0.007008,
        attack_damage_chance: 0.8,
        attack_damage_radius: 0.0,
        attack_missile_speed: 0.0,
        attack_speed: 0.4,
        attack_snipe_chance: 0.8,
        health_ratio: 13.0 / 74.0,
        build_time: 148.0,
        max_run_time: run_time(144.0, 14.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Rifle)
}
