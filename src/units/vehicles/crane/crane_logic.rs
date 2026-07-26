use crate::{
    components::ObjectStats,
    original::{
        objects::{ObjectKind, VehicleType},
        types::TeamType,
    },
    units::{UnitAttackSound, UnitSettings, buildings},
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
        health_ratio: 1.0,
        build_time: 97.0,
        max_run_time: 0.0,
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    None
}

pub(crate) fn can_issue_repair_command(
    kind: ObjectKind,
    team: TeamType,
    stats: ObjectStats,
) -> bool {
    matches!(kind, ObjectKind::Vehicle(VehicleType::Crane))
        && team != TeamType::Null
        && !stats.destroyed()
        && stats.move_speed > 0.0
}

pub(crate) fn can_repair_target(
    kind: ObjectKind,
    target_team: TeamType,
    repairer_team: TeamType,
    stats: ObjectStats,
) -> bool {
    buildings::auto_repairable_after_destroy(kind)
        && (target_team == TeamType::Null || target_team == repairer_team)
        && stats.destroyed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::BuildingType;

    #[test]
    fn crane_repair_rules_match_original_building_gate() {
        let mut stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 0);
        assert!(can_repair_target(
            ObjectKind::Building(BuildingType::Radar),
            TeamType::Red,
            TeamType::Red,
            stats
        ));
        assert!(can_repair_target(
            ObjectKind::Building(BuildingType::Repair),
            TeamType::Null,
            TeamType::Red,
            stats
        ));
        assert!(!can_repair_target(
            ObjectKind::Building(BuildingType::FortFront),
            TeamType::Red,
            TeamType::Red,
            stats
        ));
        assert!(can_repair_target(
            ObjectKind::Bridge(BuildingType::BridgeVert),
            TeamType::Null,
            TeamType::Red,
            stats
        ));
        stats.health = stats.max_health;
        assert!(!can_repair_target(
            ObjectKind::Building(BuildingType::Radar),
            TeamType::Red,
            TeamType::Red,
            stats
        ));
    }
}
