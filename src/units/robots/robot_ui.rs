use super::robot_state::RobotIdleActionKind;

use crate::{original::types::TeamType, render::atlas::MobileSpriteRole};

pub(crate) const WALK_FRAME_COUNT: usize = 4;
pub(crate) const MOBILE_FRAME_TIME: f32 = 0.3;
pub(crate) const IDLE_ACTION_FRAME_TIME: f32 = MOBILE_FRAME_TIME;
pub(crate) const GRENADE_THROW_FRAME_COUNT: usize = 4;
pub(crate) const GRENADE_THROW_FRAME_TIME: f32 = 0.15;
pub(crate) const GRENADE_PICKUP_FRAME_COUNT: usize = 4;
pub(crate) const GRENADE_PICKUP_FRAME_TIME: f32 = MOBILE_FRAME_TIME;

pub(crate) fn source_dimensions() -> bevy::prelude::Vec2 {
    bevy::prelude::Vec2::splat(16.0)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RobotAtlasFrameSpec {
    pub(crate) atlas_team: TeamType,
    pub(crate) frame_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RobotMovementProfile {
    pub(crate) frame_count: usize,
    pub(crate) frame_time: f32,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotGroupSelectionProfile {
    pub(crate) normalize_member_selection_to_leader: bool,
    pub(crate) ordered_selection_uses_leaders_only: bool,
    pub(crate) selected_leader_controls_members: bool,
}

pub(crate) fn movement_profile() -> RobotMovementProfile {
    RobotMovementProfile {
        frame_count: WALK_FRAME_COUNT,
        frame_time: MOBILE_FRAME_TIME,
    }
}

pub(crate) fn mobile_frame_count() -> usize {
    movement_profile().frame_count
}

pub(crate) fn mobile_frame_time() -> f32 {
    movement_profile().frame_time
}

pub(crate) fn mobile_sprite_role(layer_index: usize) -> Option<MobileSpriteRole> {
    (layer_index == 0).then_some(MobileSpriteRole::Robot)
}

pub(crate) fn stand_atlas_frame_spec(team: TeamType, rotation: u16) -> RobotAtlasFrameSpec {
    let atlas_team = team.atlas_team();
    RobotAtlasFrameSpec {
        atlas_team,
        frame_name: stand_atlas_frame_name(atlas_team, rotation),
    }
}

pub(crate) fn walk_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> RobotAtlasFrameSpec {
    let atlas_team = team.atlas_team();
    RobotAtlasFrameSpec {
        atlas_team,
        frame_name: walk_atlas_frame_name(atlas_team, rotation, frame),
    }
}

pub(crate) fn mobile_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
    frame: usize,
    moving: bool,
) -> RobotAtlasFrameSpec {
    if moving {
        walk_atlas_frame_spec(team, rotation, frame)
    } else {
        stand_atlas_frame_spec(team, rotation)
    }
}

pub(crate) fn throw_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> RobotAtlasFrameSpec {
    let atlas_team = team.atlas_team();
    RobotAtlasFrameSpec {
        atlas_team,
        frame_name: throw_atlas_frame_name(atlas_team, rotation, frame),
    }
}

pub(crate) fn grenade_pickup_atlas_frame_spec(
    team: TeamType,
    upward: bool,
    frame: usize,
) -> RobotAtlasFrameSpec {
    let atlas_team = team.atlas_team();
    RobotAtlasFrameSpec {
        atlas_team,
        frame_name: grenade_pickup_atlas_frame_name(atlas_team, upward, frame),
    }
}

pub(crate) fn idle_action_atlas_frame_spec(
    team: TeamType,
    kind: RobotIdleActionKind,
    frame: usize,
) -> RobotAtlasFrameSpec {
    let atlas_team = team.atlas_team();
    RobotAtlasFrameSpec {
        atlas_team,
        frame_name: idle_action_atlas_frame_name(atlas_team, kind, frame),
    }
}

pub(crate) fn stand_atlas_frame_name(team: TeamType, rotation: u16) -> String {
    let team_name = team.atlas_team().asset_name();
    format!("robot_stand_{team_name}_r{rotation:03}")
}

pub(crate) fn walk_atlas_frame_name(team: TeamType, rotation: u16, frame: usize) -> String {
    let team_name = team.atlas_team().asset_name();
    format!(
        "robot_walk_{team_name}_r{rotation:03}_n{:02}",
        frame % WALK_FRAME_COUNT
    )
}

#[cfg(test)]
pub(crate) fn mobile_atlas_frame_name(
    team: TeamType,
    rotation: u16,
    frame: usize,
    moving: bool,
) -> String {
    if moving {
        walk_atlas_frame_name(team, rotation, frame)
    } else {
        stand_atlas_frame_name(team, rotation)
    }
}

pub(crate) fn throw_atlas_frame_name(team: TeamType, rotation: u16, frame: usize) -> String {
    let team_name = team.atlas_team().asset_name();
    format!(
        "robot_throw_{team_name}_r{rotation:03}_n{:02}",
        frame % GRENADE_THROW_FRAME_COUNT
    )
}

pub(crate) fn grenade_pickup_atlas_frame_name(
    team: TeamType,
    upward: bool,
    frame: usize,
) -> String {
    let team_name = team.atlas_team().asset_name();
    let direction = if upward { "up" } else { "down" };
    format!(
        "robot_pickup-{direction}_{team_name}_n{:02}",
        frame % GRENADE_PICKUP_FRAME_COUNT
    )
}

pub(crate) fn idle_action_atlas_frame_name(
    team: TeamType,
    kind: RobotIdleActionKind,
    frame: usize,
) -> String {
    let team_name = team.atlas_team().asset_name();
    let (name, frame_count) = match kind {
        RobotIdleActionKind::Cigarette => ("cigarette", 11),
        RobotIdleActionKind::Beer => ("beer", 10),
        RobotIdleActionKind::FullScan => ("full_area_scan", 12),
        RobotIdleActionKind::HeadStretch => ("head_stretch", 11),
    };
    format!("robot_{name}_{team_name}_n{:02}", frame % frame_count)
}

#[cfg(test)]
pub(crate) fn group_selection_profile() -> RobotGroupSelectionProfile {
    RobotGroupSelectionProfile {
        normalize_member_selection_to_leader: true,
        ordered_selection_uses_leaders_only: true,
        selected_leader_controls_members: true,
    }
}

pub(crate) fn group_leader_ref_id(ref_id: u32, leader_ref_id: Option<u32>) -> u32 {
    leader_ref_id.unwrap_or(ref_id)
}

#[cfg(test)]
pub(crate) fn group_member_is_leader(ref_id: u32, leader_ref_id: Option<u32>) -> bool {
    group_leader_ref_id(ref_id, leader_ref_id) == ref_id
}

#[cfg(test)]
pub(crate) fn group_member_visible_in_ordered_selection(
    ref_id: u32,
    leader_ref_id: Option<u32>,
) -> bool {
    group_member_is_leader(ref_id, leader_ref_id)
}

#[cfg(test)]
pub(crate) fn selected_refs_include_group_member(
    selected_refs: &[u32],
    ref_id: u32,
    leader_ref_id: Option<u32>,
) -> bool {
    selected_refs.contains(&group_leader_ref_id(ref_id, leader_ref_id))
}
