use bevy::prelude::Vec2;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
}

pub(crate) fn asset_path() -> &'static str {
    "other/map_items/grenades.png"
}

pub(crate) fn projectile_frame_paths() -> Vec<String> {
    (0..4)
        .map(|frame| format!("other/grenades/grenade_n{frame:02}.png"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grenade_box_assets_match_original() {
        assert_eq!(asset_path(), "other/map_items/grenades.png");
        assert_eq!(projectile_frame_paths().len(), 4);
        assert_eq!(
            projectile_frame_paths(),
            vec![
                "other/grenades/grenade_n00.png",
                "other/grenades/grenade_n01.png",
                "other/grenades/grenade_n02.png",
                "other/grenades/grenade_n03.png",
            ]
        );
    }
}
