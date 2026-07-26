use bevy::prelude::Vec2;

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RocketsVisualSpec {
    pub(crate) asset_path: &'static str,
    pub(crate) atlas_frame_name: &'static str,
    pub(crate) selection_size: Vec2,
}

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
}

#[cfg(test)]
pub(crate) fn visual_spec() -> RocketsVisualSpec {
    RocketsVisualSpec {
        asset_path: asset_path(),
        atlas_frame_name: atlas_frame_name(),
        selection_size: default_selection_size(),
    }
}

#[cfg(test)]
pub(crate) fn asset_path() -> &'static str {
    "other/map_items/rockets.png"
}

pub(crate) fn atlas_frame_name() -> &'static str {
    "item_rockets"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rocket_box_visual_spec_matches_original_map_item_asset() {
        assert_eq!(
            visual_spec(),
            RocketsVisualSpec {
                asset_path: "other/map_items/rockets.png",
                atlas_frame_name: "item_rockets",
                selection_size: Vec2::splat(16.0),
            }
        );
    }
}
