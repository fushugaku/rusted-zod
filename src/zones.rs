use crate::components::{ZoneLink, ZoneOwnership};
use crate::constants::TILE_SIZE;
use crate::original::map::{MapObjectType, ZMap};
use crate::original::objects::{BuildingType, ItemType, ObjectKind};
use crate::original::types::TeamType;
use crate::settings_sync::SourceSettingsState;

impl ZoneOwnership {
    #[cfg(test)]
    pub(crate) fn from_map(map: &ZMap) -> Self {
        Self::from_map_with_settings(map, &SourceSettingsState::default())
    }

    pub(crate) fn from_map_with_settings(map: &ZMap, settings: &SourceSettingsState) -> Self {
        let mut owners = vec![TeamType::Null; map.zones.len()];
        let object_ref_ids = initial_map_object_ref_ids(map, settings);

        for object in &map.objects {
            if object.object_type != MapObjectType::Building
                || !matches!(
                    ObjectKind::from_map_parts(object.object_type, object.object_id),
                    Ok(ObjectKind::Building(
                        BuildingType::FortFront | BuildingType::FortBack
                    ))
                )
            {
                continue;
            }

            if let Some(zone_index) = zone_at_tile(map, object.x, object.y) {
                owners[zone_index] = object.owner;
            }
        }

        let mut links = Vec::new();
        for (map_index, flag) in map.objects.iter().enumerate() {
            if flag.object_type != MapObjectType::MapItem || flag.object_id != ItemType::Flag as u8
            {
                continue;
            }
            let Some(zone_index) = zone_at_tile(map, flag.x, flag.y) else {
                continue;
            };

            owners[zone_index] = flag.owner;
            let building_refs = map
                .objects
                .iter()
                .enumerate()
                .filter_map(|(building_ref, object)| {
                    if object.object_type == MapObjectType::Building
                        && zone_at_tile(map, object.x, object.y) == Some(zone_index)
                    {
                        object_ref_ids.get(building_ref).copied()
                    } else {
                        None
                    }
                })
                .collect();

            links.push(ZoneLink {
                zone_index,
                flag_ref_id: object_ref_ids[map_index],
                building_refs,
            });
        }

        Self { owners, links }
    }

    pub(crate) fn zone_for_flag(&self, flag_ref_id: u32) -> Option<&ZoneLink> {
        self.links
            .iter()
            .find(|link| link.flag_ref_id == flag_ref_id)
    }

    pub(crate) fn team_zone_ownage(&self, team: TeamType) -> f32 {
        if self.links.is_empty() {
            return 0.0;
        }

        let owned = self
            .links
            .iter()
            .filter(|link| self.owners.get(link.zone_index).copied() == Some(team))
            .count();

        owned as f32 / self.links.len() as f32
    }
}

fn initial_map_object_ref_ids(map: &ZMap, settings: &SourceSettingsState) -> Vec<u32> {
    let mut ref_ids = Vec::with_capacity(map.objects.len());
    let mut next_ref_id = 0;

    for object in &map.objects {
        ref_ids.push(next_ref_id);
        let kind = ObjectKind::from_map_parts(object.object_type, object.object_id)
            .unwrap_or(ObjectKind::MapItem(object.object_id));
        next_ref_id += settings.initial_spawn_count(kind);
    }

    ref_ids
}

pub(crate) fn zone_at_tile(map: &ZMap, tile_x: u16, tile_y: u16) -> Option<usize> {
    zone_at_pixel(map, tile_x as f32 * TILE_SIZE, tile_y as f32 * TILE_SIZE)
}

pub(crate) fn zone_at_pixel(map: &ZMap, x: f32, y: f32) -> Option<usize> {
    map.zones.iter().position(|zone| {
        let min_x = zone.x as f32 * TILE_SIZE;
        let min_y = zone.y as f32 * TILE_SIZE;
        let max_x = min_x + zone.w as f32 * TILE_SIZE;
        let max_y = min_y + zone.h as f32 * TILE_SIZE;
        x >= min_x && y >= min_y && x <= max_x && y <= max_y
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::{
        map::{MapBasics, MapObject, MapTile, MapZone},
        objects::RobotType,
        types::PlanetType,
    };

    fn object(
        object_type: MapObjectType,
        object_id: u8,
        x: u16,
        y: u16,
        owner: TeamType,
    ) -> MapObject {
        MapObject {
            x,
            y,
            owner,
            object_type,
            object_id,
            building_level: 0,
            extra_links: 0,
            health_percent: 100,
        }
    }

    fn map_with_objects(objects: Vec<MapObject>) -> ZMap {
        ZMap {
            basics: MapBasics {
                width: 16,
                height: 16,
                map_name: "zone_ref_test".to_string(),
                player_count: 2,
                object_count: objects.len() as u16,
                terrain_type: PlanetType::Desert,
                zone_count: 1,
            },
            zones: vec![MapZone {
                x: 0,
                y: 0,
                w: 15,
                h: 15,
            }],
            objects,
            tiles: vec![MapTile { tile: 0 }; 16 * 16],
        }
    }

    #[test]
    fn initial_map_object_ref_ids_expand_robot_groups_like_original_startup() {
        let map = map_with_objects(vec![
            object(
                MapObjectType::Robot,
                RobotType::Grunt as u8,
                1,
                1,
                TeamType::Red,
            ),
            object(
                MapObjectType::MapItem,
                ItemType::Flag as u8,
                2,
                2,
                TeamType::Red,
            ),
            object(
                MapObjectType::Building,
                BuildingType::RobotFactory as u8,
                3,
                3,
                TeamType::Red,
            ),
        ]);
        let settings = SourceSettingsState::default();
        let grunt_count = settings.initial_spawn_count(ObjectKind::Robot(RobotType::Grunt));

        assert_eq!(
            initial_map_object_ref_ids(&map, &settings),
            vec![0, grunt_count, 1 + grunt_count]
        );
    }

    #[test]
    fn zone_links_use_expanded_startup_ref_ids_after_robot_groups() {
        let map = map_with_objects(vec![
            object(
                MapObjectType::Robot,
                RobotType::Grunt as u8,
                1,
                1,
                TeamType::Red,
            ),
            object(
                MapObjectType::MapItem,
                ItemType::Flag as u8,
                2,
                2,
                TeamType::Red,
            ),
            object(
                MapObjectType::Building,
                BuildingType::RobotFactory as u8,
                3,
                3,
                TeamType::Red,
            ),
        ]);
        let grunt_count =
            SourceSettingsState::default().initial_spawn_count(ObjectKind::Robot(RobotType::Grunt));
        let zones = ZoneOwnership::from_map(&map);

        assert_eq!(zones.links.len(), 1);
        assert_eq!(zones.links[0].flag_ref_id, grunt_count);
        assert_eq!(zones.links[0].building_refs, vec![1 + grunt_count]);
    }
}
