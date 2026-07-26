use crate::{
    components::ObjectStats,
    original::objects::{ObjectKind, RobotType, VehicleType},
    units::{UnitAttackSound, UnitSettings, run_time},
};

pub(crate) const REQUIRES_ACTIVATION: bool = true;

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

pub(crate) fn removes_group_members_on_enter(target_kind: ObjectKind) -> bool {
    matches!(target_kind, ObjectKind::Vehicle(VehicleType::Apc))
}

pub(crate) fn enter_removes_group_member(
    target_kind: ObjectKind,
    entrant_ref_id: u32,
    member_ref_id: u32,
    member_leader_ref_id: u32,
    member_destroyed: bool,
    member_health: f32,
) -> bool {
    removes_group_members_on_enter(target_kind)
        && member_ref_id != entrant_ref_id
        && member_leader_ref_id == entrant_ref_id
        && !member_destroyed
        && member_health > 0.0
}

pub(crate) fn apply_driver_attack_stats(stats: &mut ObjectStats, robot_kind: RobotType) {
    let driver_stats = ObjectStats::from_kind(ObjectKind::Robot(robot_kind), 100);
    stats.attack_radius = driver_stats.attack_radius;
    stats.attack_damage = driver_stats.attack_damage;
    stats.damage_chance = driver_stats.damage_chance;
    stats.damage_radius = driver_stats.damage_radius;
    stats.missile_speed = driver_stats.missile_speed;
    stats.attack_speed = driver_stats.attack_speed;
    stats.snipe_chance = driver_stats.snipe_chance;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apc_enter_removes_live_group_members() {
        let apc = ObjectKind::Vehicle(VehicleType::Apc);
        assert!(enter_removes_group_member(apc, 10, 11, 10, false, 5.0));
        assert!(!enter_removes_group_member(apc, 10, 10, 10, false, 5.0));
        assert!(!enter_removes_group_member(apc, 10, 11, 99, false, 5.0));
        assert!(!enter_removes_group_member(apc, 10, 11, 10, true, 5.0));
        assert!(!enter_removes_group_member(apc, 10, 11, 10, false, 0.0));
        assert!(!enter_removes_group_member(
            ObjectKind::Vehicle(VehicleType::Jeep),
            10,
            11,
            10,
            false,
            5.0
        ));
    }

    #[test]
    fn apc_driver_attack_stats_follow_entering_robot() {
        let mut stats = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Apc), 100);
        apply_driver_attack_stats(&mut stats, RobotType::Tough);
        let tough = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Tough), 100);
        assert_eq!(stats.attack_radius, tough.attack_radius);
        assert_eq!(stats.attack_damage, tough.attack_damage);
        assert_eq!(stats.damage_radius, tough.damage_radius);
        assert_eq!(stats.missile_speed, tough.missile_speed);
    }
}
