use crate::{
    components::{DriverHealth, ObjectStats},
    original::{
        objects::{CannonType, ObjectKind, RobotType, VehicleType},
        types::TeamType,
    },
    settings_sync::SourceSettingsState,
    units::{
        UnitAttackSound, cannons,
        items::grenades::{
            GRENADE_ATTACK_SPEED, GRENADE_DAMAGE, GRENADE_DAMAGE_RADIUS, GRENADE_MISSILE_SPEED,
            GRENADE_SCATTER_HALF_EXTENT, can_have_grenades,
        },
        robots, vehicles,
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AttackDelivery {
    pub(crate) damage: f32,
    pub(crate) damage_chance: f32,
    pub(crate) radius: f32,
    pub(crate) missile_speed: f32,
    pub(crate) cooldown: f32,
    pub(crate) scatter_half_extent: f32,
    pub(crate) consumes_grenade: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GrenadeAttackSource {
    Own,
    GroupLeader(u32),
}

pub(crate) fn grenade_attack_source(
    kind: ObjectKind,
    ref_id: u32,
    own_amount: Option<u8>,
    leader_ref_id: Option<u32>,
    leader_amount: Option<u8>,
) -> Option<GrenadeAttackSource> {
    if !can_have_grenades(kind) {
        return None;
    }
    if own_amount.unwrap_or(0) > 0 {
        return Some(GrenadeAttackSource::Own);
    }

    let leader_ref_id = leader_ref_id.filter(|leader_ref_id| *leader_ref_id != ref_id)?;
    (leader_amount.unwrap_or(0) > 0).then_some(GrenadeAttackSource::GroupLeader(leader_ref_id))
}

pub(crate) fn grenade_attack_amount_for_source(source: Option<GrenadeAttackSource>) -> u8 {
    u8::from(source.is_some())
}

pub(crate) fn attacker_has_explosives(stats: ObjectStats, grenade_amount: u8) -> bool {
    stats.has_explosive_damage() || grenade_amount > 0
}

pub(crate) fn can_attack_target_with_grenades(
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    attacker_grenade_amount: u8,
    target_team: TeamType,
    target_stats: ObjectStats,
    distance: f32,
) -> bool {
    if !can_attack_target_identity(
        attacker_team,
        attacker_stats,
        attacker_grenade_amount,
        target_team,
        target_stats,
    ) {
        return false;
    }

    distance <= attacker_stats.attack_radius
}

pub(crate) fn can_attack_target_identity(
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    attacker_grenade_amount: u8,
    target_team: TeamType,
    target_stats: ObjectStats,
) -> bool {
    if !attacker_stats.can_attack() || target_stats.destroyed() {
        return false;
    }
    if attacker_team == target_team {
        return false;
    }
    if target_stats.attacked_only_by_explosives
        && !attacker_has_explosives(attacker_stats, attacker_grenade_amount)
    {
        return false;
    }

    true
}

pub(crate) fn should_snipe_driver(
    attacker_kind: ObjectKind,
    attacker_stats: ObjectStats,
    target_kind: ObjectKind,
    driver: &DriverHealth,
    target_can_be_sniped: bool,
    roll: f32,
) -> bool {
    can_snipe(attacker_kind, attacker_stats)
        && can_be_sniped(target_kind, driver, target_can_be_sniped)
        && roll <= attacker_stats.snipe_chance
}

fn can_snipe(kind: ObjectKind, stats: ObjectStats) -> bool {
    stats.snipe_chance > 0.0 && can_snipe_flag(kind)
}

pub(crate) fn can_snipe_flag(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Robot(RobotType::Grunt)
            | ObjectKind::Robot(RobotType::Psycho)
            | ObjectKind::Robot(RobotType::Sniper)
            | ObjectKind::Robot(RobotType::Pyro)
            | ObjectKind::Robot(RobotType::Laser)
            | ObjectKind::Vehicle(VehicleType::Jeep)
            | ObjectKind::Cannon(CannonType::Gatling)
    )
}

pub(crate) fn can_be_sniped(
    kind: ObjectKind,
    driver: &DriverHealth,
    target_can_be_sniped: bool,
) -> bool {
    driver.lead_health() > 0.0
        && target_can_be_sniped
        && matches!(
            kind,
            ObjectKind::Cannon(_)
                | ObjectKind::Vehicle(VehicleType::Jeep)
                | ObjectKind::Vehicle(VehicleType::Light)
                | ObjectKind::Vehicle(VehicleType::Medium)
                | ObjectKind::Vehicle(VehicleType::Heavy)
        )
}

pub(crate) fn target_kind_can_be_sniped(target_kind: ObjectKind, lid_open: bool) -> bool {
    match target_kind {
        ObjectKind::Cannon(_) | ObjectKind::Vehicle(VehicleType::Jeep) => true,
        ObjectKind::Vehicle(VehicleType::Light)
        | ObjectKind::Vehicle(VehicleType::Medium)
        | ObjectKind::Vehicle(VehicleType::Heavy) => lid_open,
        _ => false,
    }
}

pub(crate) fn attack_delivery(stats: ObjectStats, grenade_amount: u8) -> AttackDelivery {
    if grenade_amount > 0 {
        return AttackDelivery {
            damage: GRENADE_DAMAGE,
            damage_chance: 0.0,
            radius: GRENADE_DAMAGE_RADIUS,
            missile_speed: GRENADE_MISSILE_SPEED,
            cooldown: GRENADE_ATTACK_SPEED,
            scatter_half_extent: GRENADE_SCATTER_HALF_EXTENT,
            consumes_grenade: true,
        };
    }

    AttackDelivery {
        damage: stats.attack_damage,
        damage_chance: stats.damage_chance,
        radius: stats.damage_radius,
        missile_speed: stats.missile_speed,
        cooldown: stats.attack_speed,
        scatter_half_extent: 16.0,
        consumes_grenade: false,
    }
}

pub(crate) fn attack_delivery_with_settings(
    stats: ObjectStats,
    grenade_amount: u8,
    settings: &SourceSettingsState,
) -> AttackDelivery {
    if grenade_amount == 0 {
        return attack_delivery(stats, 0);
    }

    AttackDelivery {
        damage: settings.grenade_damage(),
        damage_chance: 0.0,
        radius: settings.grenade_damage_radius(),
        missile_speed: settings.grenade_missile_speed(),
        cooldown: settings.grenade_attack_speed(),
        scatter_half_extent: GRENADE_SCATTER_HALF_EXTENT,
        consumes_grenade: true,
    }
}

#[cfg(test)]
pub(crate) fn effective_attack_stats(
    kind: ObjectKind,
    mut stats: ObjectStats,
    driver: Option<&DriverHealth>,
) -> ObjectStats {
    if let (ObjectKind::Vehicle(VehicleType::Apc), Some(driver)) = (kind, driver) {
        let driver_stats = ObjectStats::from_kind(ObjectKind::Robot(driver.driver_kind), 100);
        stats.attack_radius = driver_stats.attack_radius;
        stats.attack_damage = driver_stats.attack_damage;
        stats.damage_chance = driver_stats.damage_chance;
        stats.damage_radius = driver_stats.damage_radius;
        stats.missile_speed = driver_stats.missile_speed;
        stats.attack_speed = driver_stats.attack_speed;
        stats.snipe_chance = driver_stats.snipe_chance;
    }
    stats
}

pub(crate) fn effective_attack_stats_with_driver_stats(
    kind: ObjectKind,
    mut stats: ObjectStats,
    driver: Option<&DriverHealth>,
    driver_stats: Option<ObjectStats>,
) -> ObjectStats {
    if matches!(kind, ObjectKind::Vehicle(VehicleType::Apc))
        && driver.is_some()
        && let Some(driver_stats) = driver_stats
    {
        stats.attack_radius = driver_stats.attack_radius;
        stats.attack_damage = driver_stats.attack_damage;
        stats.damage_chance = driver_stats.damage_chance;
        stats.damage_radius = driver_stats.damage_radius;
        stats.missile_speed = driver_stats.missile_speed;
        stats.attack_speed = driver_stats.attack_speed;
        stats.snipe_chance = driver_stats.snipe_chance;
    }
    stats
}

pub(crate) fn effective_attack_kind(kind: ObjectKind, driver: Option<&DriverHealth>) -> ObjectKind {
    if matches!(kind, ObjectKind::Vehicle(VehicleType::Apc))
        && let Some(driver) = driver
    {
        return ObjectKind::Robot(driver.driver_kind);
    }
    kind
}

pub(crate) fn driver_attack_damage_multiplier(
    kind: ObjectKind,
    driver: Option<&DriverHealth>,
) -> f32 {
    if matches!(kind, ObjectKind::Vehicle(VehicleType::Apc)) {
        driver.map_or(1.0, |driver| driver.driver_count().max(1) as f32)
    } else {
        1.0
    }
}

pub(crate) fn attack_sound_for_kind(
    kind: ObjectKind,
    consumes_grenade: bool,
) -> Option<UnitAttackSound> {
    if consumes_grenade {
        return Some(UnitAttackSound::ThrowGrenade);
    }

    match kind {
        ObjectKind::Robot(robot) => robots::attack_sound(robot),
        ObjectKind::Vehicle(vehicle) => vehicles::attack_sound(vehicle),
        ObjectKind::Cannon(cannon) => cannons::attack_sound(cannon),
        _ => None,
    }
}

pub(crate) fn attack_sound_for_attack(
    source_kind: ObjectKind,
    effective_kind: ObjectKind,
    consumes_grenade: bool,
) -> Option<UnitAttackSound> {
    if consumes_grenade {
        return Some(UnitAttackSound::ThrowGrenade);
    }

    if matches!(source_kind, ObjectKind::Vehicle(VehicleType::Apc)) {
        return vehicles::apc::driver_attack_sound(effective_kind);
    }

    attack_sound_for_kind(effective_kind, false)
}

pub(crate) fn unit_rating_will_die(attacker: ObjectKind, victim: ObjectKind) -> bool {
    use CannonType::{Gatling, Gun, Howitzer, MissileCannon};
    use ObjectKind::{Cannon, Robot, Vehicle};
    use RobotType::{Grunt, Laser, Psycho, Pyro, Sniper, Tough};
    use VehicleType::{Heavy, Jeep, Light, Medium, MissileLauncher};

    matches!(
        (attacker, victim),
        (Robot(Grunt), Robot(Psycho))
            | (Robot(Grunt), Robot(Sniper))
            | (Robot(Grunt), Robot(Tough))
            | (Robot(Grunt), Robot(Pyro))
            | (Robot(Grunt), Robot(Laser))
            | (Robot(Grunt), Cannon(Gatling))
            | (Robot(Grunt), Cannon(Gun))
            | (Robot(Grunt), Cannon(Howitzer))
            | (Robot(Grunt), Cannon(MissileCannon))
            | (Robot(Grunt), Vehicle(Jeep))
            | (Robot(Grunt), Vehicle(Light))
            | (Robot(Grunt), Vehicle(Medium))
            | (Robot(Grunt), Vehicle(Heavy))
            | (Robot(Grunt), Vehicle(MissileLauncher))
            | (Robot(Psycho), Robot(Tough))
            | (Robot(Psycho), Robot(Pyro))
            | (Robot(Psycho), Robot(Laser))
            | (Robot(Psycho), Cannon(MissileCannon))
            | (Robot(Psycho), Vehicle(Medium))
            | (Robot(Psycho), Vehicle(Heavy))
            | (Robot(Psycho), Vehicle(MissileLauncher))
            | (Robot(Sniper), Robot(Tough))
            | (Robot(Sniper), Robot(Pyro))
            | (Robot(Sniper), Robot(Laser))
            | (Robot(Sniper), Cannon(MissileCannon))
            | (Robot(Sniper), Vehicle(MissileLauncher))
            | (Robot(Tough), Cannon(MissileCannon))
            | (Robot(Tough), Vehicle(MissileLauncher))
            | (Robot(Pyro), Cannon(MissileCannon))
            | (Robot(Pyro), Vehicle(MissileLauncher))
            | (Robot(Laser), Cannon(MissileCannon))
            | (Robot(Laser), Vehicle(MissileLauncher))
            | (Vehicle(Jeep), Robot(Tough))
            | (Vehicle(Jeep), Robot(Pyro))
            | (Vehicle(Jeep), Robot(Laser))
            | (Vehicle(Jeep), Cannon(Gun))
            | (Vehicle(Jeep), Cannon(Howitzer))
            | (Vehicle(Jeep), Cannon(MissileCannon))
            | (Vehicle(Jeep), Vehicle(Light))
            | (Vehicle(Jeep), Vehicle(Medium))
            | (Vehicle(Jeep), Vehicle(Heavy))
            | (Vehicle(Jeep), Vehicle(MissileLauncher))
            | (Vehicle(Light), Cannon(MissileCannon))
            | (Vehicle(Light), Vehicle(MissileLauncher))
            | (Vehicle(Medium), Vehicle(MissileLauncher))
            | (Vehicle(MissileLauncher), Cannon(Howitzer))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_sniping_policy_matches_original_driver_windows() {
        assert!(target_kind_can_be_sniped(
            ObjectKind::Cannon(CannonType::Gun),
            false
        ));
        assert!(target_kind_can_be_sniped(
            ObjectKind::Vehicle(VehicleType::Jeep),
            false
        ));
        assert!(!target_kind_can_be_sniped(
            ObjectKind::Vehicle(VehicleType::Heavy),
            false
        ));
        assert!(target_kind_can_be_sniped(
            ObjectKind::Vehicle(VehicleType::Heavy),
            true
        ));
        assert!(!target_kind_can_be_sniped(
            ObjectKind::Vehicle(VehicleType::Heavy),
            false
        ));
        assert!(!target_kind_can_be_sniped(
            ObjectKind::Robot(RobotType::Grunt),
            true
        ));
    }

    #[test]
    fn unit_rating_will_die_matches_original_cross_reference_direction() {
        assert!(unit_rating_will_die(
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Heavy)
        ));
        assert!(!unit_rating_will_die(
            ObjectKind::Vehicle(VehicleType::Heavy),
            ObjectKind::Robot(RobotType::Grunt)
        ));
        assert!(unit_rating_will_die(
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Howitzer)
        ));
        assert!(!unit_rating_will_die(
            ObjectKind::Vehicle(VehicleType::MissileLauncher),
            ObjectKind::Vehicle(VehicleType::Jeep)
        ));
    }

    #[test]
    fn grenade_attack_source_matches_original_leader_fallback() {
        let grunt = ObjectKind::Robot(RobotType::Grunt);
        assert_eq!(
            grenade_attack_source(grunt, 11, Some(3), Some(10), Some(9)),
            Some(GrenadeAttackSource::Own)
        );
        assert_eq!(
            grenade_attack_source(grunt, 11, Some(0), Some(10), Some(9)),
            Some(GrenadeAttackSource::GroupLeader(10))
        );
        assert_eq!(
            grenade_attack_source(grunt, 10, Some(0), Some(10), Some(9)),
            None
        );
        assert_eq!(
            grenade_attack_source(
                ObjectKind::Robot(RobotType::Tough),
                11,
                Some(0),
                Some(10),
                Some(9),
            ),
            None
        );
        assert_eq!(
            grenade_attack_amount_for_source(Some(GrenadeAttackSource::GroupLeader(10))),
            1
        );
    }
}
