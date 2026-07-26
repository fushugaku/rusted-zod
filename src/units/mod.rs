use bevy::prelude::{Color, Vec2};

use crate::{
    components::{CombatRng, DamageCrater, DamageMissileVisual, PortraitAnimationKind},
    original::objects::ObjectKind,
    render::atlas::MobileSpriteRole,
};

pub(crate) mod attack;
pub(crate) mod buildings;
pub(crate) mod cannons;
pub(crate) mod items;
pub(crate) mod robots;
pub(crate) mod unit_behavior;
pub(crate) mod unit_driver;
pub(crate) mod unit_enter;
pub(crate) mod unit_sound;
pub(crate) mod unit_stats;
pub(crate) mod unit_ui;
pub(crate) mod vehicles;

pub(crate) use attack::attack_sound_for_attack;
#[cfg(test)]
pub(crate) use attack::attack_sound_for_kind;
pub(crate) use attack::unit_rating_will_die;
pub use unit_stats::UnitSettings;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnitImpactSound {
    RandomExplosion,
    TurrentExplosion,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DamageMissileVisualGeometry {
    pub(crate) primary_offset: Vec2,
    pub(crate) replica_offsets: Vec<Vec2>,
    pub(crate) smoke_offsets: Vec<Vec2>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DamageMissileLaunchEffectProfile {
    pub(crate) frame_path: String,
    pub(crate) map_top_left_offset: Vec2,
    pub(crate) z: f32,
    pub(crate) frame_time: f32,
    pub(crate) name: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RocketImpactProfile {
    pub(crate) xx_large_mushrooms: u8,
    pub(crate) large_mushrooms: u8,
    pub(crate) small_mushrooms: u8,
    pub(crate) unit_particle_radius: f32,
    pub(crate) unit_particle_amount: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DamageMissileImpactEffectProfile {
    Rocket(RocketImpactProfile),
    ToughRocket,
    MapObjectTurrent,
    Generic,
}

pub(crate) fn combat_object_default_size(kind: ObjectKind) -> Vec2 {
    unit_ui::combat_object_default_size(kind)
}

pub(crate) fn source_mobile_dimensions(kind: ObjectKind) -> Option<Vec2> {
    match kind {
        ObjectKind::Robot(robot) => Some(robots::source_dimensions(robot)),
        ObjectKind::Vehicle(vehicle) => Some(vehicles::source_dimensions(vehicle)),
        ObjectKind::Cannon(_) => Some(Vec2::splat(32.0)),
        _ => None,
    }
}

pub(crate) fn requires_activation(kind: ObjectKind) -> bool {
    match kind {
        ObjectKind::Robot(robot) => robots::requires_activation(robot),
        ObjectKind::Vehicle(vehicle) => vehicles::requires_activation(vehicle),
        _ => false,
    }
}

pub(crate) fn selected_hud_icon_asset_name(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
) -> Option<String> {
    unit_ui::selected_hud_icon_asset_name(kind, team)
}

pub(crate) fn selected_hud_label_asset_name(kind: ObjectKind) -> Option<String> {
    unit_ui::selected_hud_label_asset_name(kind)
}

pub(crate) fn queue_item_text(unit: ObjectKind) -> String {
    unit_ui::queue_item_text(unit)
}

pub(crate) fn object_kind_to_map_parts(
    kind: ObjectKind,
) -> Option<(crate::original::map::MapObjectType, u8)> {
    unit_ui::object_kind_to_map_parts(kind)
}

pub(crate) fn mobile_frame_time(role: MobileSpriteRole) -> f32 {
    unit_ui::mobile_frame_time(role)
}

pub(crate) fn mobile_frame_count(kind: ObjectKind, role: MobileSpriteRole) -> usize {
    unit_ui::mobile_frame_count(kind, role)
}

pub(crate) fn mobile_frame_delta_seconds(
    kind: ObjectKind,
    role: MobileSpriteRole,
    delta_secs: f32,
    speed_offset_percent: f32,
) -> f32 {
    unit_ui::mobile_frame_delta_seconds(kind, role, delta_secs, speed_offset_percent)
}

pub(crate) fn mobile_sprite_role(kind: ObjectKind, layer_index: usize) -> Option<MobileSpriteRole> {
    unit_ui::mobile_sprite_role(kind, layer_index)
}

pub(crate) fn selectable_for(
    kind: ObjectKind,
    selection_size: Vec2,
) -> Option<crate::components::Selectable> {
    unit_ui::selectable_for(kind, selection_size)
}

pub(crate) fn fallback_marker_size(kind: ObjectKind) -> Option<Vec2> {
    unit_ui::fallback_marker_size(kind)
}

pub(crate) fn fallback_collision_size(kind: ObjectKind) -> Vec2 {
    unit_ui::fallback_collision_size(kind)
}

pub(crate) fn fallback_marker_color(
    kind: ObjectKind,
    owner: crate::original::types::TeamType,
) -> Color {
    unit_ui::fallback_marker_color(kind, owner)
}

pub(crate) fn unit_settings(kind: ObjectKind) -> Option<UnitSettings> {
    match kind {
        ObjectKind::Robot(robot) => Some(robots::settings(robot)),
        ObjectKind::Vehicle(vehicle) => Some(vehicles::settings(vehicle)),
        ObjectKind::Cannon(cannon) => Some(cannons::settings(cannon)),
        _ => None,
    }
}

pub(crate) fn object_max_health(kind: ObjectKind) -> f32 {
    unit_stats::object_max_health(kind)
}

pub(crate) fn object_move_speed(kind: ObjectKind) -> f32 {
    unit_stats::object_move_speed(kind)
}

pub(crate) fn object_attack_radius(kind: ObjectKind) -> f32 {
    unit_stats::object_attack_radius(kind)
}

pub(crate) fn object_attack_damage(kind: ObjectKind) -> f32 {
    unit_stats::object_attack_damage(kind)
}

pub(crate) fn object_damage_chance(kind: ObjectKind) -> f32 {
    unit_stats::object_damage_chance(kind)
}

pub(crate) fn object_damage_radius(kind: ObjectKind) -> f32 {
    unit_stats::object_damage_radius(kind)
}

pub(crate) fn object_missile_speed(kind: ObjectKind) -> f32 {
    unit_stats::object_missile_speed(kind)
}

pub(crate) fn object_attack_speed(kind: ObjectKind) -> f32 {
    unit_stats::object_attack_speed(kind)
}

pub(crate) fn object_snipe_chance(kind: ObjectKind) -> f32 {
    unit_stats::object_snipe_chance(kind)
}

pub(crate) fn selected_portrait_animation_for_object(
    kind: ObjectKind,
    has_driver: bool,
    rng: &mut CombatRng,
) -> Option<PortraitAnimationKind> {
    match kind {
        ObjectKind::Robot(robot) => Some(robots::selected_portrait_animation(robot, rng)),
        ObjectKind::Vehicle(_) | ObjectKind::Cannon(_) if has_driver => {
            Some(unit_sound::selected_common_portrait_animation(rng))
        }
        _ => None,
    }
}

pub(crate) fn selected_common_voice_asset_path(
    anim_index: u8,
    rng: Option<&mut CombatRng>,
) -> String {
    unit_sound::selected_common_voice_asset_path(anim_index, rng)
}

pub(crate) fn acknowledge_portrait_animation(
    no_way: bool,
    rng: &mut CombatRng,
) -> PortraitAnimationKind {
    unit_sound::acknowledge_portrait_animation(no_way, rng)
}

pub(crate) fn attack_sound_asset_path(sound: UnitAttackSound) -> &'static str {
    unit_sound::attack_sound_asset_path(sound)
}

pub(crate) fn impact_sound_asset_path(
    sound: UnitImpactSound,
    rng: Option<&mut CombatRng>,
) -> String {
    unit_sound::impact_sound_asset_path(sound, rng)
}

pub(crate) fn can_be_entered_target(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    stats: crate::components::ObjectStats,
) -> bool {
    unit_enter::can_be_entered_target(kind, team, stats)
}

pub(crate) fn can_enter_target(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    stats: crate::components::ObjectStats,
) -> bool {
    unit_enter::can_enter_target(kind, team, stats)
}

pub(crate) fn can_enter_fort_unit(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    stats: crate::components::ObjectStats,
) -> bool {
    unit_enter::can_enter_fort_unit(kind, team, stats)
}

pub(crate) fn can_enter_fort(
    kind: ObjectKind,
    fort_team: crate::original::types::TeamType,
    entering_team: crate::original::types::TeamType,
    stats: crate::components::ObjectStats,
) -> bool {
    unit_enter::can_enter_fort(kind, fort_team, entering_team, stats)
}

pub(crate) fn attacked_only_by_explosives(kind: ObjectKind) -> bool {
    unit_driver::attacked_only_by_explosives(kind)
}

pub(crate) fn initial_driver_health(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
) -> Option<crate::components::DriverHealth> {
    unit_driver::initial_driver_health(kind, team)
}

pub(crate) fn can_eject_drivers(kind: ObjectKind, stats: crate::components::ObjectStats) -> bool {
    unit_driver::can_eject_drivers(kind, stats)
}

pub(crate) fn cannon_ejectable_on_spawn(kind: ObjectKind, fort_turret_tile: bool) -> bool {
    unit_driver::cannon_ejectable_on_spawn(kind, fort_turret_tile)
}

pub(crate) fn cannon_ejectable_for_runtime_spawn(
    kind: ObjectKind,
    requested_ejectable: bool,
) -> bool {
    unit_driver::cannon_ejectable_for_runtime_spawn(kind, requested_ejectable)
}

pub(crate) fn enter_removes_group_member(
    target_kind: ObjectKind,
    entrant_ref_id: u32,
    member_ref_id: u32,
    member_leader_ref_id: u32,
    member_destroyed: bool,
    member_health: f32,
) -> bool {
    vehicles::enter_removes_group_member(
        target_kind,
        entrant_ref_id,
        member_ref_id,
        member_leader_ref_id,
        member_destroyed,
        member_health,
    )
}

pub(crate) fn apply_apc_driver_attack_stats(
    stats: &mut crate::components::ObjectStats,
    robot_kind: crate::original::objects::RobotType,
) {
    vehicles::apply_apc_driver_attack_stats(stats, robot_kind)
}

pub(crate) fn can_passively_engage(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    stats: crate::components::ObjectStats,
    mobile: bool,
    is_moving: bool,
) -> bool {
    unit_behavior::can_passively_engage(kind, team, stats, mobile, is_moving)
}

pub(crate) fn uses_robot_route_footprint(kind: ObjectKind) -> bool {
    unit_behavior::uses_robot_route_footprint(kind)
}

pub(crate) fn passive_auto_enter_allows_available_target(kind: ObjectKind) -> bool {
    unit_behavior::passive_auto_enter_allows_available_target(kind)
}

pub(crate) fn is_flag(kind: ObjectKind) -> bool {
    unit_behavior::is_flag(kind)
}

pub(crate) fn can_capture_zone(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    destroyed: bool,
) -> bool {
    unit_behavior::can_capture_zone(kind, team, destroyed)
}

pub(crate) fn eliminated_team_for_destroyed_fort(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    destroyed: bool,
) -> Option<crate::original::types::TeamType> {
    unit_behavior::eliminated_team_for_destroyed_fort(kind, team, destroyed)
}

pub(crate) fn is_alive_combat_unit(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    object_team: crate::original::types::TeamType,
    destroyed: bool,
) -> bool {
    unit_behavior::is_alive_combat_unit(kind, team, object_team, destroyed)
}

pub(crate) fn is_alive_fort(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    object_team: crate::original::types::TeamType,
    destroyed: bool,
) -> bool {
    unit_behavior::is_alive_fort(kind, team, object_team, destroyed)
}

pub(crate) fn object_should_be_destroyed_by_fort_elimination(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    destroyed: bool,
    eliminated_team: crate::original::types::TeamType,
) -> bool {
    unit_behavior::object_should_be_destroyed_by_fort_elimination(
        kind,
        team,
        destroyed,
        eliminated_team,
    )
}

pub(crate) fn movement_speed_multiplier(
    kind: ObjectKind,
    stats: crate::components::ObjectStats,
    running: bool,
) -> f32 {
    unit_behavior::movement_speed_multiplier(kind, stats, running)
}

pub(crate) fn speed_offset_percent(
    moving: bool,
    base_move_speed: f32,
    actual_move_speed: f32,
) -> f32 {
    unit_behavior::speed_offset_percent(moving, base_move_speed, actual_move_speed)
}

pub(crate) fn missile_object_particle_amount(kind: ObjectKind, amount: usize) -> usize {
    unit_behavior::missile_object_particle_amount(kind, amount)
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

    unit_ui::damage_crater_for_visual(damage_missile_visual_for_attack(attacker, false))
}

#[cfg(test)]
pub(crate) fn light_rocket_big_crater_chance(extra_large: u8, xx_large: u8) -> Option<f32> {
    vehicles::light_ui::big_crater_chance(extra_large, xx_large)
}

pub(crate) fn damage_missile_frame_paths(visual: DamageMissileVisual) -> Vec<String> {
    unit_ui::damage_missile_frame_paths(visual)
}

#[cfg(test)]
pub(crate) fn light_rocket_init_fire_frame_path(frame: usize) -> String {
    unit_ui::light_rocket_init_fire_frame_path(frame)
}

pub(crate) fn damage_missile_launch_effect_profile(
    visual: DamageMissileVisual,
    rng: &mut CombatRng,
) -> Option<DamageMissileLaunchEffectProfile> {
    unit_ui::damage_missile_launch_effect_profile(visual, rng)
}

pub(crate) fn damage_missile_visual_geometry(
    visual: DamageMissileVisual,
    start: Vec2,
    target: Vec2,
) -> DamageMissileVisualGeometry {
    unit_ui::damage_missile_visual_geometry(visual, start, target)
}

pub(crate) fn damage_missile_rotates(visual: DamageMissileVisual) -> bool {
    unit_ui::damage_missile_rotates(visual)
}

pub(crate) fn damage_missile_muzzle_offset(
    visual: DamageMissileVisual,
    direction: usize,
) -> Option<Vec2> {
    unit_ui::damage_missile_muzzle_offset(visual, direction)
}

#[cfg(test)]
pub(crate) fn vehicle_rocket_muzzle_offset(direction: usize) -> Vec2 {
    unit_ui::vehicle_rocket_muzzle_offset(direction)
}

#[cfg(test)]
pub(crate) fn tough_rocket_muzzle_offset(direction: usize) -> Vec2 {
    unit_ui::tough_rocket_muzzle_offset(direction)
}

#[cfg(test)]
pub(crate) fn rocket_impact_profile(visual: DamageMissileVisual) -> Option<RocketImpactProfile> {
    unit_ui::rocket_impact_profile(visual)
}

pub(crate) fn damage_missile_impact_effect_profile(
    visual: DamageMissileVisual,
) -> DamageMissileImpactEffectProfile {
    unit_ui::damage_missile_impact_effect_profile(visual)
}

pub(crate) fn damage_missile_impact_sound(visual: DamageMissileVisual) -> Option<UnitImpactSound> {
    unit_ui::damage_missile_impact_sound(visual)
}

pub(crate) fn object_destroyable(kind: ObjectKind) -> bool {
    match kind {
        ObjectKind::Building(_) | ObjectKind::Bridge(_) => false,
        ObjectKind::Cannon(_) | ObjectKind::Vehicle(_) | ObjectKind::Robot(_) => true,
        ObjectKind::Animal(_) => true,
        ObjectKind::Rock | ObjectKind::MapItem(_) => {
            items::item_object_destroyable(kind).expect("item object destroyable policy exists")
        }
    }
}

pub(crate) fn object_blocks_tile_when_destroyed(kind: ObjectKind) -> bool {
    match kind {
        ObjectKind::MapItem(item_id) => items::map_item_blocks_tile(item_id),
        _ => false,
    }
}

pub(crate) fn destroyed_asset_name(
    kind: ObjectKind,
    team: crate::original::types::TeamType,
    planet: crate::original::types::PlanetType,
) -> Option<String> {
    match kind {
        ObjectKind::Building(building) => buildings::destroyed_asset_path(building, planet),
        ObjectKind::Vehicle(vehicle) => vehicles::destroyed_asset_path(vehicle, team),
        ObjectKind::Cannon(cannon) => cannons::destroyed_asset_path(cannon, team),
        _ => None,
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
    use crate::render::atlas::MobileSpriteRole;

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
    fn fallback_collision_sizes_keep_placement_geometry_in_units() {
        assert_eq!(
            fallback_collision_size(ObjectKind::Building(BuildingType::FortFront)),
            Vec2::new(160.0, 80.0)
        );
        assert_eq!(
            fallback_collision_size(ObjectKind::Building(BuildingType::Radar)),
            Vec2::splat(48.0)
        );
        assert_eq!(
            fallback_collision_size(ObjectKind::Bridge(BuildingType::BridgeHorz)),
            Vec2::splat(48.0)
        );
        assert_eq!(
            fallback_collision_size(ObjectKind::Cannon(CannonType::Gun)),
            Vec2::splat(32.0)
        );
        assert_eq!(
            fallback_collision_size(ObjectKind::MapItem(ItemType::Grenades as u8)),
            Vec2::splat(16.0)
        );
        assert_eq!(
            fallback_collision_size(ObjectKind::Vehicle(VehicleType::Jeep)),
            Vec2::ZERO
        );
    }

    #[test]
    fn mobile_animation_frame_policy_lives_on_unit_ui() {
        assert_eq!(mobile_frame_time(MobileSpriteRole::Robot), 0.3);
        assert_eq!(mobile_frame_time(MobileSpriteRole::VehicleBase), 0.1);
        assert_eq!(mobile_frame_time(MobileSpriteRole::VehicleTop), 0.0);
        assert_eq!(
            mobile_frame_count(ObjectKind::Robot(RobotType::Grunt), MobileSpriteRole::Robot),
            4
        );
        assert_eq!(
            mobile_frame_count(
                ObjectKind::Vehicle(VehicleType::Jeep),
                MobileSpriteRole::VehicleBase
            ),
            2
        );
        assert_eq!(
            mobile_frame_count(
                ObjectKind::Vehicle(VehicleType::Heavy),
                MobileSpriteRole::VehicleBase
            ),
            3
        );
        assert_eq!(
            mobile_frame_count(
                ObjectKind::Vehicle(VehicleType::Heavy),
                MobileSpriteRole::VehicleTop
            ),
            1
        );
        assert!(
            (mobile_frame_delta_seconds(
                ObjectKind::Robot(RobotType::Grunt),
                MobileSpriteRole::Robot,
                0.1,
                1.8
            ) - 0.18)
                .abs()
                < f32::EPSILON
        );
        assert_eq!(
            mobile_frame_delta_seconds(
                ObjectKind::Vehicle(VehicleType::Jeep),
                MobileSpriteRole::VehicleBase,
                0.1,
                1.8
            ),
            0.1
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
    fn damage_missile_impact_policy_lives_on_units() {
        assert_eq!(
            damage_missile_impact_effect_profile(DamageMissileVisual::MissileLauncher),
            DamageMissileImpactEffectProfile::Rocket(RocketImpactProfile {
                xx_large_mushrooms: 3,
                large_mushrooms: 0,
                small_mushrooms: 2,
                unit_particle_radius: 80.0,
                unit_particle_amount: 23,
            })
        );
        assert_eq!(
            damage_missile_impact_effect_profile(DamageMissileVisual::ToughRocket),
            DamageMissileImpactEffectProfile::ToughRocket
        );
        assert_eq!(
            damage_missile_impact_effect_profile(DamageMissileVisual::MapObjectTurrent(0)),
            DamageMissileImpactEffectProfile::MapObjectTurrent
        );
        assert_eq!(
            damage_missile_impact_effect_profile(DamageMissileVisual::Grenade),
            DamageMissileImpactEffectProfile::Generic
        );
        assert!(damage_missile_rotates(DamageMissileVisual::LightRocket {
            extra_small: 0,
            extra_large: 0,
            xx_large: 0,
        }));
        assert!(!damage_missile_rotates(DamageMissileVisual::Grenade));
        assert_eq!(
            damage_missile_impact_sound(DamageMissileVisual::MapObjectTurrent(0)),
            Some(UnitImpactSound::TurrentExplosion)
        );
        assert_eq!(
            damage_missile_impact_sound(DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 0,
                xx_large: 0,
            }),
            Some(UnitImpactSound::RandomExplosion)
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

    #[test]
    fn selected_portrait_animation_follows_source_object_and_robot_branches() {
        let mut vehicle_rng = CombatRng(0);
        assert_eq!(
            selected_portrait_animation_for_object(
                ObjectKind::Vehicle(VehicleType::Jeep),
                false,
                &mut vehicle_rng,
            ),
            None
        );
        assert_eq!(
            selected_portrait_animation_for_object(
                ObjectKind::Vehicle(VehicleType::Jeep),
                true,
                &mut vehicle_rng,
            ),
            Some(PortraitAnimationKind::SelectedCommon(0))
        );

        let mut robot_reporting_rng = CombatRng(0);
        assert_eq!(
            selected_portrait_animation_for_object(
                ObjectKind::Robot(RobotType::Grunt),
                false,
                &mut robot_reporting_rng,
            ),
            Some(PortraitAnimationKind::SelectedRobotReporting(
                RobotType::Grunt
            ))
        );

        let mut robot_common_rng = CombatRng(1);
        assert_eq!(
            selected_portrait_animation_for_object(
                ObjectKind::Robot(RobotType::Pyro),
                false,
                &mut robot_common_rng,
            ),
            Some(PortraitAnimationKind::SelectedCommon(2))
        );
    }
}
