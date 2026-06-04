use super::types::{PlanetType, TeamType};

const MAP_BASICS_SIZE: usize = 62;
const MAP_ZONE_SIZE: usize = 8;
const MAP_OBJECT_SIZE: usize = 16;
const MAP_TILE_SIZE: usize = 2;

#[derive(Clone, Debug)]
pub struct MapBasics {
    pub width: u16,
    pub height: u16,
    pub map_name: String,
    pub player_count: u8,
    pub object_count: u16,
    pub terrain_type: PlanetType,
    pub zone_count: u16,
}

#[derive(Clone, Debug)]
pub struct MapZone {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MapObjectType {
    Rock = 0,
    Bridge = 1,
    Building = 2,
    Cannon = 3,
    Vehicle = 4,
    Robot = 5,
    Animal = 6,
    MapItem = 7,
}

impl TryFrom<u8> for MapObjectType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Rock),
            1 => Ok(Self::Bridge),
            2 => Ok(Self::Building),
            3 => Ok(Self::Cannon),
            4 => Ok(Self::Vehicle),
            5 => Ok(Self::Robot),
            6 => Ok(Self::Animal),
            7 => Ok(Self::MapItem),
            other => Err(format!("unknown map object type {other}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MapObject {
    pub x: u16,
    pub y: u16,
    pub owner: TeamType,
    pub object_type: MapObjectType,
    pub object_id: u8,
    pub building_level: i8,
    pub extra_links: u16,
    pub health_percent: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct MapTile {
    pub tile: u16,
}

#[derive(Clone, Debug)]
pub struct ZMap {
    pub basics: MapBasics,
    pub zones: Vec<MapZone>,
    pub objects: Vec<MapObject>,
    pub tiles: Vec<MapTile>,
}

impl ZMap {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < MAP_BASICS_SIZE {
            return Err("map buffer is too small for map_basics".to_string());
        }

        let basics = MapBasics {
            width: read_u16(data, 0)?,
            height: read_u16(data, 2)?,
            map_name: read_c_string(data, 4, 50)?,
            player_count: read_u8(data, 54)?,
            object_count: read_u16(data, 56)?,
            terrain_type: PlanetType::try_from(read_u8(data, 58)?)?,
            zone_count: read_u16(data, 60)?,
        };

        let mut offset = MAP_BASICS_SIZE;
        let mut zones = Vec::with_capacity(basics.zone_count as usize);
        for _ in 0..basics.zone_count {
            zones.push(MapZone {
                x: read_u16(data, offset)?,
                y: read_u16(data, offset + 2)?,
                w: read_u16(data, offset + 4)?,
                h: read_u16(data, offset + 6)?,
            });
            offset += MAP_ZONE_SIZE;
        }

        let mut objects = Vec::with_capacity(basics.object_count as usize);
        for _ in 0..basics.object_count {
            // This mirrors g++'s native map_object layout from source/zmap.h.
            // The int health_percent is aligned at +12, leaving two padding bytes
            // after extra_links. Existing maps use 0xCCCC in that padding.
            objects.push(MapObject {
                x: read_u16(data, offset)?,
                y: read_u16(data, offset + 2)?,
                owner: TeamType::try_from(read_i8(data, offset + 4)?)?,
                object_type: MapObjectType::try_from(read_u8(data, offset + 5)?)?,
                object_id: read_u8(data, offset + 6)?,
                building_level: read_i8(data, offset + 7)?,
                extra_links: read_u16(data, offset + 8)?,
                health_percent: read_i32(data, offset + 12)?,
            });
            offset += MAP_OBJECT_SIZE;
        }

        let tile_count = basics.width as usize * basics.height as usize;
        let mut tiles = Vec::with_capacity(tile_count);
        for _ in 0..tile_count {
            tiles.push(MapTile {
                tile: read_u16(data, offset)?,
            });
            offset += MAP_TILE_SIZE;
        }

        Ok(Self {
            basics,
            zones,
            objects,
            tiles,
        })
    }
}

fn read_u8(data: &[u8], offset: usize) -> Result<u8, String> {
    data.get(offset)
        .copied()
        .ok_or_else(|| format!("read past end at byte {offset}"))
}

fn read_i8(data: &[u8], offset: usize) -> Result<i8, String> {
    Ok(read_u8(data, offset)? as i8)
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16, String> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| format!("read u16 past end at byte {offset}"))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_i32(data: &[u8], offset: usize) -> Result<i32, String> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| format!("read i32 past end at byte {offset}"))?;
    Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_c_string(data: &[u8], offset: usize, len: usize) -> Result<String, String> {
    let bytes = data
        .get(offset..offset + len)
        .ok_or_else(|| format!("read string past end at byte {offset}"))?;
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    Ok(String::from_utf8_lossy(&bytes[..end]).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_original_map_with_native_cpp_padding() {
        let map = ZMap::parse(include_bytes!("../../maps/p02_bb_orig01.map")).unwrap();

        assert_eq!(map.basics.width, 64);
        assert_eq!(map.basics.height, 86);
        assert_eq!(map.basics.object_count as usize, map.objects.len());
        assert_eq!(
            map.basics.width as usize * map.basics.height as usize,
            map.tiles.len()
        );
        assert_eq!(map.objects[0].health_percent, 100);
    }

    #[test]
    fn parses_every_original_map_file() {
        let maps_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("maps");
        let mut parsed = 0;

        for entry in std::fs::read_dir(&maps_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("map") {
                continue;
            }

            let bytes = std::fs::read(&path).unwrap();
            ZMap::parse(&bytes).unwrap_or_else(|err| {
                panic!("failed to parse {}: {err}", path.display());
            });
            parsed += 1;
        }

        assert_eq!(parsed, 57);
    }
}
