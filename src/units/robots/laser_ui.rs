use bevy::prelude::Vec2;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(14.0)
}

pub(crate) fn special_projectile_frame_paths() -> Vec<String> {
    (0..2)
        .map(|frame| format!("units/robots/laser/bullet_n{frame:02}.png"))
        .collect()
}
