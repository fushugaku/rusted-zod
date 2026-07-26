use bevy::prelude::Vec2;

use crate::original::types::TeamType;

pub(crate) const FIRE_FRAME_COUNT: usize = 2;
pub(crate) const PORTRAIT_SOURCE_FACE_ID: u8 = 3;
pub(crate) const PORTRAIT_SHOULDERS_HEIGHT: f32 = 36.0;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(14.0)
}

pub(crate) fn hud_name() -> &'static str {
    "psycho"
}

pub(crate) fn portrait_frame_path(team: TeamType, frame: usize) -> Option<String> {
    (team != TeamType::Null).then(|| {
        let team = team.atlas_team().asset_name();
        format!(
            "other/hud/portraits/psycho_{team}/SHEADBI{PORTRAIT_SOURCE_FACE_ID}_{:04}.png",
            frame.min(39)
        )
    })
}

pub(crate) fn selected_reporting_voice_asset_path() -> &'static str {
    "sounds/ROB08.wav"
}

pub(crate) fn fire_atlas_frame_name(team: TeamType, rotation: u16, frame: usize) -> String {
    let team_name = team.atlas_team().asset_name();
    format!(
        "robot_psycho_fire_{team_name}_r{rotation:03}_n{:02}",
        frame % FIRE_FRAME_COUNT
    )
}
