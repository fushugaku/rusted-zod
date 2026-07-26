use crate::units::{UnitAttackSound, UnitSettings, run_time};

pub(crate) const REQUIRES_ACTIVATION: bool = true;

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 4,
        move_speed: 14.0,
        attack_radius: 136.0,
        attack_damage: 0.017799,
        attack_damage_chance: 0.7,
        attack_damage_radius: 0.0,
        attack_missile_speed: 0.0,
        attack_speed: 0.4,
        attack_snipe_chance: 0.6,
        health_ratio: 15.0 / 74.0,
        build_time: 179.0,
        max_run_time: run_time(136.0, 14.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Laser)
}
