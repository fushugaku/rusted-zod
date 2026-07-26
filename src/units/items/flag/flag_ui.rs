use bevy::prelude::Vec2;

use crate::original::types::TeamType;

pub(crate) const FLAG_ANIMATION_FRAME_COUNT: usize = 4;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
}

pub(crate) fn atlas_frame_name(team: TeamType) -> String {
    animation_frame_name(team, 0)
}

pub(crate) fn animation_frame_name(team: TeamType, frame: usize) -> String {
    let team_name = team.asset_name();
    format!("flag_{team_name}_{}", frame % FLAG_ANIMATION_FRAME_COUNT)
}

pub(crate) fn animation_frame_names(team: TeamType) -> Vec<String> {
    (0..FLAG_ANIMATION_FRAME_COUNT)
        .map(|frame| animation_frame_name(team, frame))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_animation_names_match_map_item_atlas_frames() {
        assert_eq!(atlas_frame_name(TeamType::Red), "flag_red_0");
        assert_eq!(
            animation_frame_names(TeamType::Blue),
            vec!["flag_blue_0", "flag_blue_1", "flag_blue_2", "flag_blue_3"]
        );
        assert_eq!(animation_frame_name(TeamType::Yellow, 5), "flag_yellow_1");
        assert_eq!(
            animation_frame_names(TeamType::Null),
            vec!["flag_null_0", "flag_null_1", "flag_null_2", "flag_null_3"]
        );
    }
}
