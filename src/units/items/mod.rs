use bevy::prelude::Vec2;

use crate::original::objects::ItemType;

pub(crate) mod animal;
pub(crate) mod animal_ui;
pub(crate) mod flag;
pub(crate) mod flag_ui;
pub(crate) mod grenades;
pub(crate) mod grenades_ui;
pub(crate) mod hut;
pub(crate) mod hut_ui;
pub(crate) mod map_object;
pub(crate) mod map_object_ui;
pub(crate) mod rock;
pub(crate) mod rock_ui;
pub(crate) mod rockets;
pub(crate) mod rockets_ui;

pub(crate) fn default_selection_size(item_id: u8) -> Vec2 {
    match item_id {
        id if id == ItemType::Flag as u8 => flag::default_selection_size(),
        id if id == ItemType::Rock as u8 => rock::default_selection_size(),
        id if id == ItemType::Grenades as u8 => grenades::default_selection_size(),
        id if id == ItemType::Rockets as u8 => rockets::default_selection_size(),
        id if id == ItemType::Hut as u8 => hut::default_selection_size(),
        id if id >= ItemType::MapObjectStart as u8 => map_object::default_selection_size(),
        _ => map_object::default_selection_size(),
    }
}
