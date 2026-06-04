use bevy::prelude::Vec2;

use crate::{
    components::{DamageCrater, DamageMissileVisual},
    original::{
        objects::{ObjectKind, VehicleType},
        settings::UnitSettings,
    },
};

pub(crate) mod buildings;
pub(crate) mod cannons;
pub(crate) mod items;
pub(crate) mod robots;
pub(crate) mod vehicles;

const RUN_PAST_RADIUS: f32 = 1.3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnitAttackSound {
    Rifle,
    Psycho,
    Tough,
    Pyro,
    Laser,
    Gun,
    Gatling,
    Jeep,
    Light,
    Medium,
    Heavy,
    MobileMissile,
    ThrowGrenade,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DamageMissileVisualGeometry {
    pub(crate) primary_offset: Vec2,
    pub(crate) replica_offsets: Vec<Vec2>,
    pub(crate) smoke_offsets: Vec<Vec2>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RocketImpactProfile {
    pub(crate) xx_large_mushrooms: u8,
    pub(crate) large_mushrooms: u8,
    pub(crate) small_mushrooms: u8,
    pub(crate) unit_particle_radius: f32,
    pub(crate) unit_particle_amount: usize,
}

pub(crate) fn combat_object_default_size(kind: ObjectKind) -> Vec2 {
    match kind {
        ObjectKind::Robot(robot) => robots::default_selection_size(robot),
        ObjectKind::Vehicle(vehicle) => vehicles::default_selection_size(vehicle),
        ObjectKind::Cannon(cannon) => cannons::default_selection_size(cannon),
        ObjectKind::Building(building) | ObjectKind::Bridge(building) => {
            buildings::default_selection_size(building)
        }
        ObjectKind::Rock => items::rock::default_selection_size(),
        ObjectKind::Animal(_) => items::animal::default_selection_size(),
        ObjectKind::MapItem(item_id) => items::default_selection_size(item_id),
    }
}

pub(crate) fn unit_settings(kind: ObjectKind) -> Option<UnitSettings> {
    match kind {
        ObjectKind::Robot(robot) => Some(robots::settings(robot)),
        ObjectKind::Vehicle(vehicle) => Some(vehicles::settings(vehicle)),
        ObjectKind::Cannon(cannon) => Some(cannons::settings(cannon)),
        _ => None,
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

pub(crate) fn special_projectile_kind_for_attack(
    kind: ObjectKind,
) -> Option<robots::SpecialProjectileKind> {
    match kind {
        ObjectKind::Robot(robot) => robots::special_projectile_kind(robot),
        _ => None,
    }
}

pub(crate) fn damage_missile_visual_for_attack(
    attacker: ObjectKind,
    consumes_grenade: bool,
) -> DamageMissileVisual {
    if consumes_grenade {
        return DamageMissileVisual::Grenade;
    }

    match attacker {
        ObjectKind::Robot(robot) => robots::damage_missile_visual(robot),
        ObjectKind::Vehicle(vehicle) => vehicles::damage_missile_visual(vehicle),
        ObjectKind::Cannon(cannon) => cannons::damage_missile_visual(cannon),
        _ => None,
    }
    .unwrap_or(DamageMissileVisual::Generic)
}

pub(crate) fn damage_crater_for_attack(
    attacker: ObjectKind,
    consumes_grenade: bool,
) -> Option<DamageCrater> {
    if consumes_grenade {
        return Some(DamageCrater {
            is_big: false,
            chance: 0.35,
            big_chance: None,
        });
    }

    damage_crater_for_visual(damage_missile_visual_for_attack(attacker, false))
}

pub(crate) fn damage_crater_for_visual(visual: DamageMissileVisual) -> Option<DamageCrater> {
    match visual {
        DamageMissileVisual::MissileLauncher => Some(DamageCrater {
            is_big: true,
            chance: 0.75,
            big_chance: None,
        }),
        DamageMissileVisual::MissileCannon => Some(DamageCrater {
            is_big: false,
            chance: 1.0,
            big_chance: None,
        }),
        DamageMissileVisual::ToughRocket => Some(DamageCrater {
            is_big: false,
            chance: 0.35,
            big_chance: None,
        }),
        DamageMissileVisual::LightRocket {
            extra_large,
            xx_large,
            ..
        } => Some(DamageCrater {
            is_big: false,
            chance: 0.75,
            big_chance: light_rocket_big_crater_chance(extra_large, xx_large),
        }),
        _ => Some(DamageCrater {
            is_big: false,
            chance: 0.75,
            big_chance: None,
        }),
    }
}

pub(crate) fn light_rocket_big_crater_chance(extra_large: u8, xx_large: u8) -> Option<f32> {
    if xx_large > 0 {
        Some(0.35)
    } else if extra_large > 0 {
        Some(0.15)
    } else {
        None
    }
}

pub(crate) fn damage_missile_frame_paths(visual: DamageMissileVisual) -> Vec<String> {
    match visual {
        DamageMissileVisual::Generic => Vec::new(),
        DamageMissileVisual::Grenade => items::grenades::projectile_frame_paths(),
        DamageMissileVisual::ToughRocket => robots::tough::damage_missile_frame_paths(),
        DamageMissileVisual::LightRocket { .. } => vehicles::light::damage_missile_frame_paths(),
        DamageMissileVisual::MissileCannon => cannons::missile_cannon::damage_missile_frame_paths(),
        DamageMissileVisual::MissileLauncher => {
            vehicles::missile_launcher::damage_missile_frame_paths()
        }
        DamageMissileVisual::MapObjectTurrent(object_i) => {
            items::map_object::turrent_frame_paths(object_i)
        }
    }
}

pub(crate) fn light_rocket_init_fire_frame_path(frame: usize) -> String {
    vehicles::light::init_fire_frame_path(frame)
}

pub(crate) fn damage_missile_visual_geometry(
    visual: DamageMissileVisual,
    start: Vec2,
    target: Vec2,
) -> DamageMissileVisualGeometry {
    let direction = missile_direction(start, target);
    match visual {
        DamageMissileVisual::ToughRocket => DamageMissileVisualGeometry {
            primary_offset: Vec2::ZERO,
            replica_offsets: Vec::new(),
            smoke_offsets: vec![Vec2::ZERO],
        },
        DamageMissileVisual::MissileCannon => {
            let primary_offset = cannons::missile_cannon::primary_offset(direction);
            let other_offset = primary_offset + cannons::missile_cannon::other_offset(direction);
            DamageMissileVisualGeometry {
                primary_offset,
                replica_offsets: vec![other_offset],
                smoke_offsets: vec![primary_offset, other_offset],
            }
        }
        DamageMissileVisual::MissileLauncher => {
            let left = vehicles::missile_launcher::left_offset(direction);
            let right = vehicles::missile_launcher::right_offset(direction);
            DamageMissileVisualGeometry {
                primary_offset: Vec2::ZERO,
                replica_offsets: vec![left, right],
                smoke_offsets: vec![Vec2::ZERO, left, right],
            }
        }
        _ => DamageMissileVisualGeometry {
            primary_offset: Vec2::ZERO,
            replica_offsets: Vec::new(),
            smoke_offsets: Vec::new(),
        },
    }
}

pub(crate) fn damage_missile_muzzle_offset(
    visual: DamageMissileVisual,
    direction: usize,
) -> Option<Vec2> {
    match visual {
        DamageMissileVisual::ToughRocket => Some(robots::tough::rocket_muzzle_offset(direction)),
        DamageMissileVisual::LightRocket { .. }
        | DamageMissileVisual::MissileCannon
        | DamageMissileVisual::MissileLauncher => Some(vehicles::rocket_muzzle_offset(direction)),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn vehicle_rocket_muzzle_offset(direction: usize) -> Vec2 {
    vehicles::rocket_muzzle_offset(direction)
}

#[cfg(test)]
pub(crate) fn tough_rocket_muzzle_offset(direction: usize) -> Vec2 {
    robots::tough::rocket_muzzle_offset(direction)
}

pub(crate) fn rocket_impact_profile(visual: DamageMissileVisual) -> Option<RocketImpactProfile> {
    match visual {
        DamageMissileVisual::LightRocket {
            extra_small,
            extra_large,
            xx_large,
        } => Some(vehicles::light::rocket_impact_profile(
            extra_small,
            extra_large,
            xx_large,
        )),
        DamageMissileVisual::MissileCannon => {
            Some(cannons::missile_cannon::rocket_impact_profile())
        }
        DamageMissileVisual::MissileLauncher => {
            Some(vehicles::missile_launcher::rocket_impact_profile())
        }
        _ => None,
    }
}

fn missile_direction(start: Vec2, target: Vec2) -> Vec2 {
    let delta = target - start;
    let mag = delta.length();
    if mag <= f32::EPSILON {
        Vec2::X
    } else {
        delta / mag
    }
}

pub(crate) fn produced_object_count(kind: ObjectKind) -> u8 {
    match kind {
        ObjectKind::Robot(_) => unit_settings(kind)
            .map(|settings| settings.group_amount)
            .unwrap_or(0),
        ObjectKind::Vehicle(_) => 1,
        ObjectKind::Cannon(_) => 0,
        _ => 0,
    }
}

pub(crate) fn run_time(radius: f32, move_speed: f32) -> f32 {
    if move_speed <= 0.0 {
        0.0
    } else {
        RUN_PAST_RADIUS * radius / move_speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{BuildingType, CannonType, ItemType, RobotType, VehicleType};

    #[test]
    fn combat_object_default_sizes_keep_original_rust_port_groups() {
        for robot in [
            RobotType::Grunt,
            RobotType::Psycho,
            RobotType::Sniper,
            RobotType::Tough,
            RobotType::Pyro,
            RobotType::Laser,
        ] {
            assert_eq!(
                combat_object_default_size(ObjectKind::Robot(robot)),
                Vec2::splat(14.0)
            );
        }

        for vehicle in [
            VehicleType::Jeep,
            VehicleType::Light,
            VehicleType::Medium,
            VehicleType::Heavy,
            VehicleType::Apc,
            VehicleType::MissileLauncher,
            VehicleType::Crane,
        ] {
            assert_eq!(
                combat_object_default_size(ObjectKind::Vehicle(vehicle)),
                Vec2::splat(32.0)
            );
        }

        for cannon in [
            CannonType::Gatling,
            CannonType::Gun,
            CannonType::Howitzer,
            CannonType::MissileCannon,
        ] {
            assert_eq!(
                combat_object_default_size(ObjectKind::Cannon(cannon)),
                Vec2::splat(32.0)
            );
        }

        for building in [
            BuildingType::FortFront,
            BuildingType::FortBack,
            BuildingType::Radar,
            BuildingType::Repair,
            BuildingType::RobotFactory,
            BuildingType::VehicleFactory,
            BuildingType::BridgeVert,
            BuildingType::BridgeHorz,
        ] {
            assert_eq!(
                combat_object_default_size(ObjectKind::Building(building)),
                Vec2::splat(64.0)
            );
            assert_eq!(
                combat_object_default_size(ObjectKind::Bridge(building)),
                Vec2::splat(64.0)
            );
        }

        assert_eq!(
            combat_object_default_size(ObjectKind::Rock),
            Vec2::splat(16.0)
        );
        assert_eq!(
            combat_object_default_size(ObjectKind::Animal(0)),
            Vec2::splat(16.0)
        );
        assert_eq!(
            combat_object_default_size(ObjectKind::MapItem(ItemType::Flag as u8)),
            Vec2::splat(16.0)
        );
        assert_eq!(
            combat_object_default_size(ObjectKind::MapItem(ItemType::MapObjectStart as u8 + 3)),
            Vec2::splat(16.0)
        );
    }

    #[test]
    fn produced_object_count_uses_unit_files() {
        assert_eq!(
            produced_object_count(ObjectKind::Robot(RobotType::Grunt)),
            3
        );
        assert_eq!(
            produced_object_count(ObjectKind::Robot(RobotType::Tough)),
            2
        );
        assert_eq!(
            produced_object_count(ObjectKind::Robot(RobotType::Laser)),
            4
        );
        assert_eq!(
            produced_object_count(ObjectKind::Vehicle(VehicleType::Jeep)),
            1
        );
        assert_eq!(
            produced_object_count(ObjectKind::Cannon(CannonType::Gatling)),
            0
        );
    }

    #[test]
    fn attack_sounds_live_on_unit_profiles() {
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Grunt), false),
            Some(UnitAttackSound::Rifle)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Psycho), false),
            Some(UnitAttackSound::Psycho)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Vehicle(VehicleType::MissileLauncher), false),
            Some(UnitAttackSound::MobileMissile)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Cannon(CannonType::Howitzer), false),
            Some(UnitAttackSound::Heavy)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Vehicle(VehicleType::Crane), false),
            None
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Grunt), true),
            Some(UnitAttackSound::ThrowGrenade)
        );
    }

    #[test]
    fn apc_driver_attack_sounds_keep_original_overrides() {
        assert_eq!(
            attack_sound_for_attack(
                ObjectKind::Vehicle(VehicleType::Apc),
                ObjectKind::Robot(RobotType::Psycho),
                false,
            ),
            Some(UnitAttackSound::Rifle)
        );
        assert_eq!(
            attack_sound_for_attack(
                ObjectKind::Vehicle(VehicleType::Apc),
                ObjectKind::Robot(RobotType::Tough),
                false,
            ),
            Some(UnitAttackSound::Tough)
        );
    }

    #[test]
    fn special_projectile_profiles_live_on_robot_units() {
        assert_eq!(
            special_projectile_kind_for_attack(ObjectKind::Robot(RobotType::Laser)),
            Some(robots::SpecialProjectileKind::Laser)
        );
        assert_eq!(
            special_projectile_kind_for_attack(ObjectKind::Robot(RobotType::Pyro)),
            Some(robots::SpecialProjectileKind::Flame)
        );
        assert_eq!(
            special_projectile_kind_for_attack(ObjectKind::Robot(RobotType::Grunt)),
            None
        );
        assert_eq!(
            robots::special_projectile_frame_paths(robots::SpecialProjectileKind::Laser),
            vec![
                "units/robots/laser/bullet_n00.png",
                "units/robots/laser/bullet_n01.png",
            ]
        );
        assert_eq!(
            robots::special_projectile_frame_paths(robots::SpecialProjectileKind::Flame),
            vec![
                "units/robots/pyro/bullet_n00.png",
                "units/robots/pyro/bullet_n01.png",
                "units/robots/pyro/bullet_n02.png",
                "units/robots/pyro/bullet_n03.png",
            ]
        );
    }

    #[test]
    fn damage_missile_visual_profiles_live_on_units() {
        assert_eq!(
            damage_missile_visual_for_attack(ObjectKind::Robot(RobotType::Grunt), true),
            DamageMissileVisual::Grenade
        );
        assert_eq!(
            damage_missile_visual_for_attack(ObjectKind::Robot(RobotType::Tough), false),
            DamageMissileVisual::ToughRocket
        );
        assert_eq!(
            damage_missile_visual_for_attack(ObjectKind::Vehicle(VehicleType::Medium), false),
            DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 1,
                xx_large: 0
            }
        );
        assert_eq!(
            damage_missile_visual_for_attack(ObjectKind::Vehicle(VehicleType::Heavy), false),
            DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 1,
                xx_large: 1
            }
        );
        assert_eq!(
            damage_missile_visual_for_attack(ObjectKind::Cannon(CannonType::Howitzer), false),
            DamageMissileVisual::LightRocket {
                extra_small: 1,
                extra_large: 1,
                xx_large: 0
            }
        );
        assert_eq!(
            damage_missile_visual_for_attack(ObjectKind::Cannon(CannonType::MissileCannon), false),
            DamageMissileVisual::MissileCannon
        );
        assert_eq!(
            damage_missile_visual_for_attack(
                ObjectKind::Vehicle(VehicleType::MissileLauncher),
                false,
            ),
            DamageMissileVisual::MissileLauncher
        );
    }

    #[test]
    fn damage_missile_asset_profiles_live_on_units() {
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::ToughRocket),
            vec![
                "units/robots/tough/bullet_n00.png",
                "units/robots/tough/bullet_n01.png"
            ]
        );
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::LightRocket {
                extra_small: 1,
                extra_large: 1,
                xx_large: 0
            }),
            vec!["units/vehicles/light/bullet.png"]
        );
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::MissileCannon),
            vec!["units/cannons/missile_cannon/bullet.png"]
        );
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::MissileLauncher),
            vec!["units/vehicles/missile_launcher/bullet.png"]
        );
        assert_eq!(
            light_rocket_init_fire_frame_path(3),
            "units/vehicles/light/initfire_n03.png"
        );
    }

    #[test]
    fn damage_crater_profiles_live_on_units() {
        assert_eq!(light_rocket_big_crater_chance(0, 0), None);
        assert_eq!(light_rocket_big_crater_chance(1, 0), Some(0.15));
        assert_eq!(light_rocket_big_crater_chance(1, 1), Some(0.35));
        assert_eq!(
            damage_crater_for_attack(ObjectKind::Cannon(CannonType::MissileCannon), false),
            Some(DamageCrater {
                is_big: false,
                chance: 1.0,
                big_chance: None,
            })
        );
        assert_eq!(
            damage_crater_for_attack(ObjectKind::Vehicle(VehicleType::Heavy), false),
            Some(DamageCrater {
                is_big: false,
                chance: 0.75,
                big_chance: Some(0.35),
            })
        );
    }

    #[test]
    fn rocket_impact_profiles_live_on_projectile_units() {
        assert_eq!(
            rocket_impact_profile(DamageMissileVisual::LightRocket {
                extra_small: 1,
                extra_large: 1,
                xx_large: 0
            }),
            Some(RocketImpactProfile {
                xx_large_mushrooms: 0,
                large_mushrooms: 1,
                small_mushrooms: 1,
                unit_particle_radius: 40.0,
                unit_particle_amount: 12,
            })
        );
        assert_eq!(
            rocket_impact_profile(DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 1,
                xx_large: 1
            }),
            Some(RocketImpactProfile {
                xx_large_mushrooms: 1,
                large_mushrooms: 1,
                small_mushrooms: 0,
                unit_particle_radius: 40.0,
                unit_particle_amount: 14,
            })
        );
        assert_eq!(
            rocket_impact_profile(DamageMissileVisual::MissileCannon),
            Some(RocketImpactProfile {
                xx_large_mushrooms: 0,
                large_mushrooms: 3,
                small_mushrooms: 1,
                unit_particle_radius: 50.0,
                unit_particle_amount: 18,
            })
        );
        assert_eq!(
            rocket_impact_profile(DamageMissileVisual::MissileLauncher),
            Some(RocketImpactProfile {
                xx_large_mushrooms: 3,
                large_mushrooms: 0,
                small_mushrooms: 2,
                unit_particle_radius: 80.0,
                unit_particle_amount: 23,
            })
        );
    }

    #[test]
    fn multi_rocket_visual_offsets_live_on_projectile_units() {
        let start = Vec2::ZERO;
        let target = Vec2::new(100.0, 0.0);

        let missile_cannon =
            damage_missile_visual_geometry(DamageMissileVisual::MissileCannon, start, target);
        assert_eq!(missile_cannon.primary_offset, Vec2::new(0.0, -8.0));
        assert_eq!(missile_cannon.replica_offsets, vec![Vec2::ZERO]);
        assert_eq!(
            missile_cannon.smoke_offsets,
            vec![Vec2::new(0.0, -8.0), Vec2::ZERO]
        );

        let missile_launcher =
            damage_missile_visual_geometry(DamageMissileVisual::MissileLauncher, start, target);
        assert_eq!(missile_launcher.primary_offset, Vec2::ZERO);
        assert_eq!(
            missile_launcher.replica_offsets,
            vec![Vec2::new(0.0, 8.0), Vec2::new(0.0, -8.0)]
        );
        assert_eq!(
            missile_launcher.smoke_offsets,
            vec![Vec2::ZERO, Vec2::new(0.0, 8.0), Vec2::new(0.0, -8.0)]
        );
    }

    #[test]
    fn rocket_muzzle_offsets_live_on_unit_families() {
        assert_eq!(vehicle_rocket_muzzle_offset(0), Vec2::new(21.0, 2.0));
        assert_eq!(vehicle_rocket_muzzle_offset(2), Vec2::new(1.0, 22.0));
        assert_eq!(vehicle_rocket_muzzle_offset(4), Vec2::new(-19.0, 2.0));
        assert_eq!(vehicle_rocket_muzzle_offset(6), Vec2::new(1.0, -18.0));

        assert_eq!(tough_rocket_muzzle_offset(0), Vec2::new(8.0, 0.0));
        assert_eq!(tough_rocket_muzzle_offset(2), Vec2::new(0.0, 8.0));
        assert_eq!(tough_rocket_muzzle_offset(4), Vec2::new(-8.0, 0.0));
        assert_eq!(tough_rocket_muzzle_offset(6), Vec2::new(0.0, -8.0));
    }
}
