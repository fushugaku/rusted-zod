use bevy::prelude::{Color, Vec2};

use crate::{
    constants::TILE_SIZE,
    original::{
        objects::{ItemType, ObjectKind},
        types::{PlanetType, TeamType},
    },
};

use super::{
    flag, flag_ui, grenades, grenades_ui, hut, hut_ui, map_object, map_object_ui, rock, rock_ui,
    rockets, rockets_ui,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MapItemAtlasFrameSpec {
    pub(crate) frame_name: String,
    pub(crate) animation_frame_names: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MapItemDisplayPolicy {
    #[cfg(test)]
    pub(crate) owner: TeamType,
    pub(crate) fallback_marker_size: Option<Vec2>,
    pub(crate) fallback_marker_color: Color,
    #[cfg(test)]
    pub(crate) blocks_tile: bool,
    #[cfg(test)]
    pub(crate) destroyable: bool,
}

const ROCK_PATHING_BLOCKS: &[(u16, u16)] = &[(0, 2)];
const SINGLE_TILE_PATHING_BLOCKS: &[(u16, u16)] = &[(0, 0)];
const NO_PATHING_BLOCKS: &[(u16, u16)] = &[];

pub(crate) fn default_selection_size(item_id: u8) -> Vec2 {
    match item_id {
        id if id == ItemType::Flag as u8 => flag_ui::default_selection_size(),
        id if id == ItemType::Rock as u8 => rock_ui::default_selection_size(),
        id if id == ItemType::Grenades as u8 => grenades_ui::default_selection_size(),
        id if id == ItemType::Rockets as u8 => rockets_ui::default_selection_size(),
        id if id == ItemType::Hut as u8 => hut_ui::default_selection_size(),
        id if id >= ItemType::MapObjectStart as u8 => map_object_ui::default_selection_size(),
        _ => map_object_ui::default_selection_size(),
    }
}

pub(crate) fn fallback_collision_size(kind: ObjectKind) -> Option<Vec2> {
    matches!(kind, ObjectKind::Rock | ObjectKind::MapItem(_)).then_some(Vec2::splat(TILE_SIZE))
}

pub(crate) fn map_item_atlas_frame_spec(
    item_id: u8,
    owner: TeamType,
    planet: PlanetType,
) -> Option<MapItemAtlasFrameSpec> {
    let (frame_name, animation_frame_names) = match item_id {
        id if id == ItemType::Flag as u8 => {
            let animation_frame_names = flag_ui::animation_frame_names(owner);
            (flag_ui::atlas_frame_name(owner), animation_frame_names)
        }
        id if id == ItemType::Grenades as u8 => {
            (grenades_ui::atlas_frame_name().to_string(), Vec::new())
        }
        id if id == ItemType::Rockets as u8 => {
            (rockets_ui::atlas_frame_name().to_string(), Vec::new())
        }
        id if id == ItemType::Hut as u8 => (hut_ui::atlas_frame_name(planet), Vec::new()),
        id if id >= ItemType::MapObjectStart as u8 => (
            map_object_ui::atlas_frame_name(id - ItemType::MapObjectStart as u8),
            Vec::new(),
        ),
        _ => return None,
    };

    Some(MapItemAtlasFrameSpec {
        frame_name,
        animation_frame_names,
    })
}

pub(crate) fn map_item_display_policy(item_id: u8, owner: TeamType) -> MapItemDisplayPolicy {
    let owner = map_item_display_owner(item_id, owner);
    MapItemDisplayPolicy {
        #[cfg(test)]
        owner,
        fallback_marker_size: map_item_fallback_marker_size(item_id),
        fallback_marker_color: map_item_fallback_marker_color(item_id, owner),
        #[cfg(test)]
        blocks_tile: map_item_blocks_tile(item_id),
        #[cfg(test)]
        destroyable: map_item_destroyable(item_id),
    }
}

pub(crate) fn map_item_health_ratio(item_id: u8) -> f32 {
    match item_id {
        id if id == ItemType::Flag as u8 => flag::health_ratio(),
        id if id == ItemType::Rock as u8 => rock::health_ratio(),
        id if id == ItemType::Grenades as u8 => grenades::health_ratio(),
        id if id == ItemType::Rockets as u8 => rockets::health_ratio(),
        id if id == ItemType::Hut as u8 => hut::health_ratio(),
        id if id >= ItemType::MapObjectStart as u8 => map_object::health_ratio(),
        _ => map_object::health_ratio(),
    }
}

pub(crate) fn item_object_health_ratio(kind: ObjectKind) -> Option<f32> {
    match kind {
        ObjectKind::Rock => Some(rock::health_ratio()),
        ObjectKind::Animal(_) => Some(super::animal::health_ratio()),
        ObjectKind::MapItem(item_id) => Some(map_item_health_ratio(item_id)),
        _ => None,
    }
}

pub(crate) fn object_display_owner(kind: ObjectKind, owner: TeamType) -> TeamType {
    match kind {
        ObjectKind::Rock => TeamType::Null,
        ObjectKind::MapItem(item_id) => map_item_display_owner(item_id, owner),
        _ => owner,
    }
}

pub(crate) fn map_item_display_owner(item_id: u8, owner: TeamType) -> TeamType {
    if item_id == ItemType::Flag as u8 {
        owner
    } else {
        TeamType::Null
    }
}

pub(crate) fn map_item_fallback_marker_size(item_id: u8) -> Option<Vec2> {
    if item_id == ItemType::Rock as u8 {
        None
    } else {
        Some(Vec2::splat(6.0))
    }
}

pub(crate) fn map_item_fallback_marker_color(item_id: u8, owner: TeamType) -> Color {
    if owner != TeamType::Null {
        return owner.color();
    }

    if item_id == ItemType::Rock as u8 {
        Color::srgb(0.45, 0.42, 0.38)
    } else {
        Color::srgb(0.1, 0.9, 0.2)
    }
}

pub(crate) fn map_item_blocks_tile(item_id: u8) -> bool {
    item_id == ItemType::Hut as u8 || item_id >= ItemType::MapObjectStart as u8
}

pub(crate) fn map_item_pathing_block_offsets(item_id: u8) -> &'static [(u16, u16)] {
    if item_id == ItemType::Rock as u8 {
        return ROCK_PATHING_BLOCKS;
    }

    if map_item_blocks_tile(item_id) {
        return SINGLE_TILE_PATHING_BLOCKS;
    }

    NO_PATHING_BLOCKS
}

pub(crate) fn map_item_destroyable(item_id: u8) -> bool {
    item_id != ItemType::Flag as u8
}

pub(crate) fn item_object_destroyable(kind: ObjectKind) -> Option<bool> {
    match kind {
        ObjectKind::Rock => Some(true),
        ObjectKind::MapItem(item_id) => Some(map_item_destroyable(item_id)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_item_atlas_specs_match_original_frame_names() {
        assert_eq!(
            map_item_atlas_frame_spec(ItemType::Flag as u8, TeamType::Green, PlanetType::Desert),
            Some(MapItemAtlasFrameSpec {
                frame_name: "flag_green_0".to_string(),
                animation_frame_names: vec![
                    "flag_green_0".to_string(),
                    "flag_green_1".to_string(),
                    "flag_green_2".to_string(),
                    "flag_green_3".to_string(),
                ],
            })
        );
        assert_eq!(
            map_item_atlas_frame_spec(ItemType::Grenades as u8, TeamType::Red, PlanetType::Desert)
                .unwrap()
                .frame_name,
            "item_grenades"
        );
        assert_eq!(
            map_item_atlas_frame_spec(ItemType::Rockets as u8, TeamType::Red, PlanetType::Desert)
                .unwrap()
                .frame_name,
            "item_rockets"
        );
        assert_eq!(
            map_item_atlas_frame_spec(ItemType::Hut as u8, TeamType::Red, PlanetType::City)
                .unwrap()
                .frame_name,
            "hut_city"
        );
        assert_eq!(
            map_item_atlas_frame_spec(
                ItemType::MapObjectStart as u8 + 99,
                TeamType::Red,
                PlanetType::Desert
            )
            .unwrap()
            .frame_name,
            "map_object21"
        );
        assert!(
            map_item_atlas_frame_spec(ItemType::Rock as u8, TeamType::Red, PlanetType::Desert)
                .is_none()
        );
    }

    #[test]
    fn map_item_display_policy_matches_original_owner_and_blocking_rules() {
        let flag = map_item_display_policy(ItemType::Flag as u8, TeamType::Blue);
        assert_eq!(flag.owner, TeamType::Blue);
        assert_eq!(flag.fallback_marker_size, Some(Vec2::splat(6.0)));
        assert!(!flag.blocks_tile);
        assert!(!flag.destroyable);

        let grenades = map_item_display_policy(ItemType::Grenades as u8, TeamType::Blue);
        assert_eq!(grenades.owner, TeamType::Null);
        assert_eq!(grenades.fallback_marker_size, Some(Vec2::splat(6.0)));
        assert!(!grenades.blocks_tile);
        assert!(grenades.destroyable);

        let hut = map_item_display_policy(ItemType::Hut as u8, TeamType::Yellow);
        assert_eq!(hut.owner, TeamType::Null);
        assert!(hut.blocks_tile);
        assert!(hut.destroyable);

        let map_object =
            map_item_display_policy(ItemType::MapObjectStart as u8 + 3, TeamType::Green);
        assert_eq!(map_object.owner, TeamType::Null);
        assert!(map_object.blocks_tile);

        let rock = map_item_display_policy(ItemType::Rock as u8, TeamType::Red);
        assert_eq!(rock.owner, TeamType::Null);
        assert_eq!(rock.fallback_marker_size, None);
        assert!(rock.destroyable);
    }

    #[test]
    fn map_item_pathing_offsets_match_original_blocked_tiles() {
        assert_eq!(
            map_item_pathing_block_offsets(ItemType::Rock as u8),
            &[(0, 2)]
        );
        assert_eq!(
            map_item_pathing_block_offsets(ItemType::Hut as u8),
            &[(0, 0)]
        );
        assert_eq!(
            map_item_pathing_block_offsets(ItemType::MapObjectStart as u8 + 2),
            &[(0, 0)]
        );
        assert!(map_item_pathing_block_offsets(ItemType::Flag as u8).is_empty());
        assert!(map_item_pathing_block_offsets(ItemType::Grenades as u8).is_empty());
    }

    #[test]
    fn object_display_owner_covers_rock_and_map_items() {
        assert_eq!(
            object_display_owner(ObjectKind::MapItem(ItemType::Flag as u8), TeamType::Red),
            TeamType::Red
        );
        assert_eq!(
            object_display_owner(ObjectKind::MapItem(ItemType::Rockets as u8), TeamType::Red),
            TeamType::Null
        );
        assert_eq!(
            object_display_owner(ObjectKind::Rock, TeamType::Blue),
            TeamType::Null
        );
        assert_eq!(
            item_object_destroyable(ObjectKind::MapItem(ItemType::Flag as u8)),
            Some(false)
        );
        assert_eq!(item_object_destroyable(ObjectKind::Rock), Some(true));
        assert_eq!(
            item_object_destroyable(ObjectKind::Robot(
                crate::original::objects::RobotType::Grunt
            )),
            None
        );
    }
}
