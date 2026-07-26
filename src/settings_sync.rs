use bevy::prelude::{ResMut, Resource};

use crate::{
    components::ObjectStats,
    constants::TILE_SIZE,
    network_commands::{
        CommandPayload, RequestSettingsCommand, SOURCE_ZSETTINGS_PACKET_SIZE, SetSettingsPacket,
    },
    original::objects::{BuildingType, CannonType, ItemType, ObjectKind, RobotType, VehicleType},
    units::{self, UnitSettings, buildings, items, unit_stats::MAX_UNIT_HEALTH, vehicles},
};

const SOURCE_UNIT_SETTINGS_SIZE: usize = 72;
const SOURCE_ROBOT_COUNT: usize = 6;
const SOURCE_VEHICLE_COUNT: usize = 7;
const SOURCE_CANNON_COUNT: usize = 4;
const SOURCE_UNIT_COUNT: usize = SOURCE_ROBOT_COUNT + SOURCE_VEHICLE_COUNT + SOURCE_CANNON_COUNT;
const SOURCE_HEALTH_RATIOS_OFFSET: usize = SOURCE_UNIT_COUNT * SOURCE_UNIT_SETTINGS_SIZE;
const SOURCE_HEALTH_RATIO_COUNT: usize = 11;
const SOURCE_GLOBALS_OFFSET: usize = SOURCE_HEALTH_RATIOS_OFFSET + SOURCE_HEALTH_RATIO_COUNT * 8;

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub(crate) struct SourceSettingsState {
    packet_bytes: Vec<u8>,
}

#[derive(Default, Resource)]
pub(crate) struct SourceSettingsInitialRequestState {
    requested: bool,
}

impl Default for SourceSettingsState {
    fn default() -> Self {
        Self {
            packet_bytes: source_zsettings_default_payload(),
        }
    }
}

impl SourceSettingsState {
    #[cfg(test)]
    pub(crate) fn packet_bytes(&self) -> &[u8] {
        &self.packet_bytes
    }

    pub(crate) fn unit_settings(&self, kind: ObjectKind) -> Option<UnitSettings> {
        let index = source_unit_settings_index(kind)?;
        let offset = index.checked_mul(SOURCE_UNIT_SETTINGS_SIZE)?;
        Some(UnitSettings {
            group_amount: u8::try_from(self.read_i32(offset)?.max(0)).unwrap_or(u8::MAX),
            move_speed: self.read_i32(offset + 4)?.max(0) as f32,
            attack_radius: self.read_i32(offset + 8)?.max(0) as f32,
            attack_damage: self.read_f64(offset + 12)?.max(0.0) as f32,
            attack_damage_chance: self.read_f64(offset + 20)?.clamp(0.0, 1.0) as f32,
            attack_damage_radius: self.read_i32(offset + 28)?.max(0) as f32,
            attack_missile_speed: self.read_i32(offset + 32)?.max(0) as f32,
            attack_speed: self.read_f64(offset + 36)?.max(0.0) as f32,
            attack_snipe_chance: self.read_f64(offset + 44)?.clamp(0.0, 1.0) as f32,
            health_ratio: self.read_f64(offset + 52)?.max(0.0) as f32,
            build_time: self.read_i32(offset + 60)?.max(0) as f32,
            max_run_time: self.read_f64(offset + 64)?.max(0.0) as f32,
        })
    }

    pub(crate) fn object_health_ratio(&self, kind: ObjectKind) -> Option<f32> {
        if let Some(settings) = self.unit_settings(kind) {
            return Some(settings.health_ratio);
        }
        let index = source_health_ratio_index(kind)?;
        self.read_f64(SOURCE_HEALTH_RATIOS_OFFSET + index * 8)
            .map(|ratio| ratio.max(0.0) as f32)
    }

    pub(crate) fn object_stats(&self, kind: ObjectKind, health_percent: i32) -> ObjectStats {
        let mut stats = ObjectStats::from_kind(kind, health_percent);
        let max_health = self
            .object_health_ratio(kind)
            .map(source_scaled_stat)
            .unwrap_or(stats.max_health);
        stats.max_health = max_health;
        stats.health = source_health_from_percent(max_health, health_percent);

        if let Some(settings) = self.unit_settings(kind) {
            stats.move_speed = settings.move_speed;
            stats.attack_radius = settings.attack_radius;
            stats.attack_damage = source_scaled_stat(settings.attack_damage);
            stats.damage_chance = settings.attack_damage_chance;
            stats.damage_radius = settings.attack_damage_radius;
            stats.missile_speed = settings.attack_missile_speed;
            stats.attack_speed = settings.attack_speed;
            stats.snipe_chance = settings.attack_snipe_chance;
        }
        stats
    }

    pub(crate) fn produced_object_count(&self, kind: ObjectKind) -> u8 {
        match kind {
            ObjectKind::Robot(_) => self
                .unit_settings(kind)
                .map(|settings| settings.group_amount)
                .unwrap_or(0),
            ObjectKind::Vehicle(_) => 1,
            ObjectKind::Cannon(_) => 0,
            _ => 0,
        }
    }

    pub(crate) fn initial_spawn_count(&self, kind: ObjectKind) -> u32 {
        match kind {
            ObjectKind::Robot(_) => u32::from(self.produced_object_count(kind)),
            _ => 1,
        }
    }

    pub(crate) fn grenade_damage(&self) -> f32 {
        self.read_f64(SOURCE_GLOBALS_OFFSET)
            .map(|value| source_scaled_stat(value.max(0.0) as f32))
            .unwrap_or(items::grenades::GRENADE_DAMAGE)
    }

    pub(crate) fn grenade_damage_radius(&self) -> f32 {
        self.read_i32(SOURCE_GLOBALS_OFFSET + 8)
            .map(|value| value.max(0) as f32)
            .unwrap_or(items::grenades::GRENADE_DAMAGE_RADIUS)
    }

    pub(crate) fn grenade_missile_speed(&self) -> f32 {
        self.read_i32(SOURCE_GLOBALS_OFFSET + 12)
            .map(|value| value.max(0) as f32)
            .unwrap_or(items::grenades::GRENADE_MISSILE_SPEED)
    }

    pub(crate) fn grenade_attack_speed(&self) -> f32 {
        self.read_f64(SOURCE_GLOBALS_OFFSET + 16)
            .map(|value| value.max(0.0) as f32)
            .unwrap_or(items::grenades::GRENADE_ATTACK_SPEED)
    }

    fn read_i32(&self, offset: usize) -> Option<i32> {
        let bytes = self.packet_bytes.get(offset..offset.checked_add(4)?)?;
        Some(i32::from_le_bytes(bytes.try_into().ok()?))
    }

    fn read_f64(&self, offset: usize) -> Option<f64> {
        let bytes = self.packet_bytes.get(offset..offset.checked_add(8)?)?;
        let value = f64::from_le_bytes(bytes.try_into().ok()?);
        value.is_finite().then_some(value)
    }
}

fn source_unit_settings_index(kind: ObjectKind) -> Option<usize> {
    match kind {
        ObjectKind::Robot(robot) => [
            RobotType::Grunt,
            RobotType::Psycho,
            RobotType::Sniper,
            RobotType::Tough,
            RobotType::Pyro,
            RobotType::Laser,
        ]
        .iter()
        .position(|candidate| *candidate == robot),
        ObjectKind::Vehicle(vehicle) => [
            VehicleType::Jeep,
            VehicleType::Light,
            VehicleType::Medium,
            VehicleType::Heavy,
            VehicleType::Apc,
            VehicleType::MissileLauncher,
            VehicleType::Crane,
        ]
        .iter()
        .position(|candidate| *candidate == vehicle)
        .map(|index| SOURCE_ROBOT_COUNT + index),
        ObjectKind::Cannon(cannon) => [
            CannonType::Gatling,
            CannonType::Gun,
            CannonType::Howitzer,
            CannonType::MissileCannon,
        ]
        .iter()
        .position(|candidate| *candidate == cannon)
        .map(|index| SOURCE_ROBOT_COUNT + SOURCE_VEHICLE_COUNT + index),
        _ => None,
    }
}

fn source_health_ratio_index(kind: ObjectKind) -> Option<usize> {
    match kind {
        ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack) => Some(0),
        ObjectKind::Building(BuildingType::RobotFactory) => Some(1),
        ObjectKind::Building(BuildingType::VehicleFactory) => Some(2),
        ObjectKind::Building(BuildingType::Repair) => Some(3),
        ObjectKind::Building(BuildingType::Radar) => Some(4),
        ObjectKind::Bridge(BuildingType::BridgeVert | BuildingType::BridgeHorz) => Some(5),
        ObjectKind::Rock => Some(6),
        ObjectKind::MapItem(item) if item == ItemType::Rock as u8 => Some(6),
        ObjectKind::MapItem(item) if item == ItemType::Grenades as u8 => Some(7),
        ObjectKind::MapItem(item) if item == ItemType::Rockets as u8 => Some(8),
        ObjectKind::MapItem(item) if item == ItemType::Hut as u8 => Some(9),
        ObjectKind::MapItem(_) | ObjectKind::Animal(_) => Some(10),
        _ => None,
    }
}

fn source_scaled_stat(ratio: f32) -> f32 {
    (ratio * MAX_UNIT_HEALTH) as i32 as f32
}

fn source_health_from_percent(max_health: f32, health_percent: i32) -> f32 {
    (health_percent.clamp(0, 100) as f32 * max_health / 100.0) as i32 as f32
}

pub(crate) fn process_initial_settings_request(
    mut initial_request: ResMut<SourceSettingsInitialRequestState>,
    mut settings: ResMut<SourceSettingsState>,
) {
    if initial_request.requested {
        return;
    }

    relay_request_settings(&mut settings);
    initial_request.requested = true;
}

pub(crate) fn relay_request_settings(settings: &mut SourceSettingsState) -> bool {
    let wire_packet = RequestSettingsCommand.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(_request) = RequestSettingsCommand::decode_payload(payload) else {
        return false;
    };
    let Some(packet) = SetSettingsPacket::new(source_zsettings_default_payload()) else {
        return false;
    };
    relay_set_settings(settings, packet)
}

fn relay_set_settings(settings: &mut SourceSettingsState, packet: SetSettingsPacket) -> bool {
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = SetSettingsPacket::decode_payload(payload) else {
        return false;
    };
    apply_set_settings(settings, decoded_packet)
}

fn apply_set_settings(settings: &mut SourceSettingsState, packet: SetSettingsPacket) -> bool {
    settings.packet_bytes = packet.bytes;
    true
}

fn source_zsettings_default_payload() -> Vec<u8> {
    let mut payload = Vec::with_capacity(SOURCE_ZSETTINGS_PACKET_SIZE);

    for robot in [
        RobotType::Grunt,
        RobotType::Psycho,
        RobotType::Sniper,
        RobotType::Tough,
        RobotType::Pyro,
        RobotType::Laser,
    ] {
        push_unit_settings(
            &mut payload,
            units::unit_settings(ObjectKind::Robot(robot)).unwrap(),
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
        push_unit_settings(
            &mut payload,
            units::unit_settings(ObjectKind::Vehicle(vehicle)).unwrap(),
        );
    }
    for cannon in [
        CannonType::Gatling,
        CannonType::Gun,
        CannonType::Howitzer,
        CannonType::MissileCannon,
    ] {
        push_unit_settings(
            &mut payload,
            units::unit_settings(ObjectKind::Cannon(cannon)).unwrap(),
        );
    }

    for ratio in [
        buildings::health_ratio(BuildingType::FortFront),
        buildings::health_ratio(BuildingType::RobotFactory),
        buildings::health_ratio(BuildingType::VehicleFactory),
        buildings::health_ratio(BuildingType::Repair),
        buildings::health_ratio(BuildingType::Radar),
        buildings::health_ratio(BuildingType::BridgeVert),
        item_health_ratio(ObjectKind::MapItem(ItemType::Rock as u8)),
        item_health_ratio(ObjectKind::MapItem(ItemType::Grenades as u8)),
        item_health_ratio(ObjectKind::MapItem(ItemType::Rockets as u8)),
        item_health_ratio(ObjectKind::MapItem(ItemType::Hut as u8)),
        item_health_ratio(ObjectKind::MapItem(ItemType::MapObjectStart as u8)),
    ] {
        push_f64(&mut payload, ratio);
    }

    push_f64(&mut payload, 40.0 / 240.0);
    push_i32(&mut payload, items::grenades::GRENADE_DAMAGE_RADIUS as i32);
    push_i32(&mut payload, items::grenades::GRENADE_MISSILE_SPEED as i32);
    push_f64(&mut payload, items::grenades::GRENADE_ATTACK_SPEED);
    push_f64(&mut payload, 50.0 / 240.0);

    for value in [
        units::unit_behavior::AGRO_DISTANCE as i32,
        units::unit_behavior::AUTO_GRAB_VEHICLE_DISTANCE as i32,
        220,
        10 * 60,
        60,
        300,
        300,
        items::grenades::GRENADES_PER_BOX as i32,
    ] {
        push_i32(&mut payload, value);
    }

    for value in [
        vehicles::PARTIALLY_DAMAGED_UNIT_SPEED,
        vehicles::DAMAGED_UNIT_SPEED,
        units::unit_behavior::RUN_UNIT_SPEED,
        units::unit_behavior::RUN_RECHARGE_RATE,
    ] {
        push_f64(&mut payload, value);
    }

    push_i32(&mut payload, items::animal::HUT_ANIMAL_MAX as i32);
    push_i32(&mut payload, items::animal::HUT_ANIMAL_MIN as i32);
    push_i32(&mut payload, 7 * TILE_SIZE as i32);

    debug_assert_eq!(payload.len(), SOURCE_ZSETTINGS_PACKET_SIZE);
    payload
}

fn push_unit_settings(payload: &mut Vec<u8>, settings: UnitSettings) {
    push_i32(payload, i32::from(settings.group_amount));
    push_i32(payload, settings.move_speed as i32);
    push_i32(payload, settings.attack_radius as i32);
    push_f64(payload, settings.attack_damage);
    push_f64(payload, settings.attack_damage_chance);
    push_i32(payload, settings.attack_damage_radius as i32);
    push_i32(payload, settings.attack_missile_speed as i32);
    push_f64(payload, settings.attack_speed);
    push_f64(payload, settings.attack_snipe_chance);
    push_f64(payload, settings.health_ratio);
    push_i32(payload, settings.build_time as i32);
    push_f64(payload, settings.max_run_time);
}

fn item_health_ratio(kind: ObjectKind) -> f32 {
    items::item_object_health_ratio(kind).expect("source item health ratio exists")
}

fn push_i32(payload: &mut Vec<u8>, value: i32) {
    payload.extend_from_slice(&value.to_le_bytes());
}

fn push_f64(payload: &mut Vec<u8>, value: f32) {
    payload.extend_from_slice(&(value as f64).to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_i32(payload: &[u8], offset: usize) -> i32 {
        i32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap())
    }

    fn read_f64(payload: &[u8], offset: usize) -> f64 {
        f64::from_le_bytes(payload[offset..offset + 8].try_into().unwrap())
    }

    fn write_i32(payload: &mut [u8], offset: usize, value: i32) {
        payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_f64(payload: &mut [u8], offset: usize, value: f64) {
        payload[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn source_zsettings_payload_matches_packed_source_size() {
        let payload = source_zsettings_default_payload();

        assert_eq!(payload.len(), SOURCE_ZSETTINGS_PACKET_SIZE);
        assert_eq!(read_i32(&payload, 0), 3);
        assert_eq!(read_i32(&payload, 4), 14);
        assert_eq!(read_i32(&payload, 8), 120);
        assert!((read_f64(&payload, 12) - 0.0011046).abs() < 0.000000001);
        assert_eq!(read_i32(&payload, 60), 72);
    }

    #[test]
    fn settings_request_round_trips_source_default_payload() {
        let mut settings = SourceSettingsState {
            packet_bytes: vec![0; SOURCE_ZSETTINGS_PACKET_SIZE],
        };

        assert!(relay_request_settings(&mut settings));

        assert_eq!(settings.packet_bytes(), source_zsettings_default_payload());
    }

    #[test]
    fn set_settings_packet_replaces_client_settings_bytes() {
        let mut settings = SourceSettingsState::default();
        let packet = SetSettingsPacket::new(vec![3; SOURCE_ZSETTINGS_PACKET_SIZE]).unwrap();

        assert!(relay_set_settings(&mut settings, packet));

        assert!(settings.packet_bytes().iter().all(|byte| *byte == 3));
    }

    #[test]
    fn decoded_settings_drive_runtime_stats_groups_and_grenades() {
        let mut payload = source_zsettings_default_payload();
        write_i32(&mut payload, 0, 5);
        write_i32(&mut payload, 4, 31);
        write_f64(&mut payload, 12, 0.25);
        write_f64(&mut payload, 52, 0.5);
        write_f64(&mut payload, SOURCE_GLOBALS_OFFSET, 0.2);
        write_i32(&mut payload, SOURCE_GLOBALS_OFFSET + 8, 77);
        write_i32(&mut payload, SOURCE_GLOBALS_OFFSET + 12, 88);
        write_f64(&mut payload, SOURCE_GLOBALS_OFFSET + 16, 1.75);

        let mut settings = SourceSettingsState::default();
        assert!(relay_set_settings(
            &mut settings,
            SetSettingsPacket::new(payload).unwrap(),
        ));

        let kind = ObjectKind::Robot(RobotType::Grunt);
        let unit = settings.unit_settings(kind).unwrap();
        assert_eq!(unit.group_amount, 5);
        assert_eq!(unit.move_speed, 31.0);
        let stats = settings.object_stats(kind, 50);
        assert_eq!(stats.max_health, 5000.0);
        assert_eq!(stats.health, 2500.0);
        assert_eq!(stats.move_speed, 31.0);
        assert_eq!(stats.attack_damage, 2500.0);
        assert_eq!(settings.produced_object_count(kind), 5);
        assert_eq!(settings.initial_spawn_count(kind), 5);
        assert_eq!(settings.grenade_damage(), 2000.0);
        assert_eq!(settings.grenade_damage_radius(), 77.0);
        assert_eq!(settings.grenade_missile_speed(), 88.0);
        assert_eq!(settings.grenade_attack_speed(), 1.75);
    }
}
