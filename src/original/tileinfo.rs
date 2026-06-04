pub const MAX_PLANET_TILES: usize = 20 * 24;
const TILE_INFO_RECORD_SIZE: usize = 12;

pub const ROAD_SPEED: f32 = 1.689;
pub const WATER_SPEED: f32 = 0.7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaletteTileInfo {
    pub is_water: bool,
    pub is_passable: bool,
    pub is_usable: bool,
    pub is_road: bool,
    pub is_effect: bool,
    pub is_water_effect: bool,
    pub next_tile_in_effect: u16,
    pub takes_tank_tracks: bool,
    pub crater_type: i16,
    pub is_starter_tile: bool,
}

impl PaletteTileInfo {
    pub fn walk_speed(self) -> f32 {
        if !self.is_passable {
            0.0
        } else if self.is_road {
            ROAD_SPEED
        } else if self.is_water {
            WATER_SPEED
        } else {
            1.0
        }
    }
}

pub fn parse_palette_tile_info(data: &[u8]) -> Result<Vec<PaletteTileInfo>, String> {
    let expected_size = MAX_PLANET_TILES * TILE_INFO_RECORD_SIZE;
    if data.len() != expected_size {
        return Err(format!(
            "tileinfo size mismatch: expected {expected_size} bytes, got {}",
            data.len()
        ));
    }

    let mut tiles = Vec::with_capacity(MAX_PLANET_TILES);
    for i in 0..MAX_PLANET_TILES {
        let offset = i * TILE_INFO_RECORD_SIZE;
        tiles.push(PaletteTileInfo {
            is_water: read_bool(data, offset)?,
            is_passable: read_bool(data, offset + 1)?,
            is_usable: read_bool(data, offset + 2)?,
            is_road: read_bool(data, offset + 3)?,
            is_effect: read_bool(data, offset + 4)?,
            is_water_effect: read_bool(data, offset + 5)?,
            next_tile_in_effect: read_u16(data, offset + 6)?,
            takes_tank_tracks: read_bool(data, offset + 8)?,
            crater_type: read_i16(data, offset + 9)?,
            is_starter_tile: read_bool(data, offset + 11)?,
        });
    }

    Ok(tiles)
}

fn read_bool(data: &[u8], offset: usize) -> Result<bool, String> {
    Ok(*data
        .get(offset)
        .ok_or_else(|| format!("read bool past end at byte {offset}"))?
        != 0)
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16, String> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| format!("read u16 past end at byte {offset}"))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_i16(data: &[u8], offset: usize) -> Result<i16, String> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| format!("read i16 past end at byte {offset}"))?;
    Ok(i16::from_le_bytes([bytes[0], bytes[1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_original_desert_tileinfo() {
        let tiles = parse_palette_tile_info(include_bytes!("../../assets/planets/desert.tileinfo"))
            .unwrap();

        assert_eq!(tiles.len(), MAX_PLANET_TILES);
        assert!(tiles.iter().any(|tile| tile.is_road));
        assert!(tiles.iter().any(|tile| tile.is_water));
        assert!(tiles.iter().any(|tile| tile.takes_tank_tracks));
        assert!(tiles.iter().any(|tile| tile.walk_speed() == ROAD_SPEED));
        assert!(tiles.iter().any(|tile| tile.walk_speed() == WATER_SPEED));
    }
}
