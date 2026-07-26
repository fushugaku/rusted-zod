#[path = "animal/animal_mod.rs"]
pub(crate) mod animal;
#[path = "flag/flag_mod.rs"]
pub(crate) mod flag;
#[path = "grenades/grenades_mod.rs"]
pub(crate) mod grenades;
#[path = "hut/hut_mod.rs"]
pub(crate) mod hut;
pub(crate) mod item_ui;
#[path = "map_object/map_object_mod.rs"]
pub(crate) mod map_object;
#[path = "rock/rock_mod.rs"]
pub(crate) mod rock;
#[path = "rockets/rockets_mod.rs"]
pub(crate) mod rockets;

pub(crate) mod animal_ui {
    pub(crate) use super::animal::animal_ui::*;
}

pub(crate) mod flag_ui {
    pub(crate) use super::flag::flag_ui::*;
}

pub(crate) mod grenades_ui {
    pub(crate) use super::grenades::grenades_ui::*;
}

pub(crate) mod hut_ui {
    pub(crate) use super::hut::hut_ui::*;
}

pub(crate) mod map_object_ui {
    pub(crate) use super::map_object::map_object_ui::*;
}

pub(crate) mod rock_ui {
    pub(crate) use super::rock::rock_ui::*;
}

pub(crate) mod rockets_ui {
    pub(crate) use super::rockets::rockets_ui::*;
}

pub(crate) use item_ui::default_selection_size;

pub(crate) fn flag_animation_frame_names(team: crate::original::types::TeamType) -> Vec<String> {
    flag_ui::animation_frame_names(team)
}

pub(crate) fn map_item_atlas_frame_spec(
    item_id: u8,
    owner: crate::original::types::TeamType,
    planet: crate::original::types::PlanetType,
) -> Option<item_ui::MapItemAtlasFrameSpec> {
    item_ui::map_item_atlas_frame_spec(item_id, owner, planet)
}

pub(crate) fn map_item_display_policy(
    item_id: u8,
    owner: crate::original::types::TeamType,
) -> item_ui::MapItemDisplayPolicy {
    item_ui::map_item_display_policy(item_id, owner)
}

pub(crate) fn fallback_collision_size(
    kind: crate::original::objects::ObjectKind,
) -> Option<bevy::prelude::Vec2> {
    item_ui::fallback_collision_size(kind)
}

pub(crate) fn object_display_owner(
    kind: crate::original::objects::ObjectKind,
    owner: crate::original::types::TeamType,
) -> crate::original::types::TeamType {
    item_ui::object_display_owner(kind, owner)
}

pub(crate) fn map_item_blocks_tile(item_id: u8) -> bool {
    item_ui::map_item_blocks_tile(item_id)
}

pub(crate) fn map_item_pathing_block_offsets(item_id: u8) -> &'static [(u16, u16)] {
    item_ui::map_item_pathing_block_offsets(item_id)
}

pub(crate) fn item_object_destroyable(kind: crate::original::objects::ObjectKind) -> Option<bool> {
    item_ui::item_object_destroyable(kind)
}

pub(crate) fn item_object_health_ratio(kind: crate::original::objects::ObjectKind) -> Option<f32> {
    item_ui::item_object_health_ratio(kind)
}
