use super::map::MapObjectType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum RobotType {
    Grunt = 0,
    Psycho = 1,
    Sniper = 2,
    Tough = 3,
    Pyro = 4,
    Laser = 5,
}

impl TryFrom<u8> for RobotType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Grunt),
            1 => Ok(Self::Psycho),
            2 => Ok(Self::Sniper),
            3 => Ok(Self::Tough),
            4 => Ok(Self::Pyro),
            5 => Ok(Self::Laser),
            other => Err(format!("unknown robot type {other}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum VehicleType {
    Jeep = 0,
    Light = 1,
    Medium = 2,
    Heavy = 3,
    Apc = 4,
    MissileLauncher = 5,
    Crane = 6,
}

impl TryFrom<u8> for VehicleType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Jeep),
            1 => Ok(Self::Light),
            2 => Ok(Self::Medium),
            3 => Ok(Self::Heavy),
            4 => Ok(Self::Apc),
            5 => Ok(Self::MissileLauncher),
            6 => Ok(Self::Crane),
            other => Err(format!("unknown vehicle type {other}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CannonType {
    Gatling = 0,
    Gun = 1,
    Howitzer = 2,
    MissileCannon = 3,
}

impl TryFrom<u8> for CannonType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Gatling),
            1 => Ok(Self::Gun),
            2 => Ok(Self::Howitzer),
            3 => Ok(Self::MissileCannon),
            other => Err(format!("unknown cannon type {other}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum BuildingType {
    FortFront = 0,
    FortBack = 1,
    Radar = 2,
    Repair = 3,
    RobotFactory = 4,
    VehicleFactory = 5,
    BridgeVert = 6,
    BridgeHorz = 7,
}

impl TryFrom<u8> for BuildingType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::FortFront),
            1 => Ok(Self::FortBack),
            2 => Ok(Self::Radar),
            3 => Ok(Self::Repair),
            4 => Ok(Self::RobotFactory),
            5 => Ok(Self::VehicleFactory),
            6 => Ok(Self::BridgeVert),
            7 => Ok(Self::BridgeHorz),
            other => Err(format!("unknown building type {other}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
#[repr(u8)]
pub enum ItemType {
    Flag = 0,
    Rock = 1,
    Grenades = 2,
    Rockets = 3,
    Hut = 4,
    MapObjectStart = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectKind {
    Rock,
    Bridge(BuildingType),
    Building(BuildingType),
    Cannon(CannonType),
    Vehicle(VehicleType),
    Robot(RobotType),
    Animal(u8),
    MapItem(u8),
}

impl ObjectKind {
    pub fn from_map_parts(object_type: MapObjectType, object_id: u8) -> Result<Self, String> {
        match object_type {
            MapObjectType::Rock => Ok(Self::Rock),
            MapObjectType::Bridge => Ok(Self::Bridge(BuildingType::try_from(object_id)?)),
            MapObjectType::Building => Ok(Self::Building(BuildingType::try_from(object_id)?)),
            MapObjectType::Cannon => Ok(Self::Cannon(CannonType::try_from(object_id)?)),
            MapObjectType::Vehicle => Ok(Self::Vehicle(VehicleType::try_from(object_id)?)),
            MapObjectType::Robot => Ok(Self::Robot(RobotType::try_from(object_id)?)),
            MapObjectType::Animal => Ok(Self::Animal(object_id)),
            MapObjectType::MapItem => Ok(Self::MapItem(object_id)),
        }
    }
}
