use bevy::prelude::{Component, Vec2};

use crate::{
    components::CombatRng,
    original::{
        objects::{ObjectKind, VehicleType},
        types::{PlanetType, TeamType},
    },
    units::vehicles::{
        self, VehicleDamageProfile, VehicleDeathEffectBounds, VehicleDeathWreckProfile,
        VehicleDestroyedAssetProfile, VehicleTrackKind, vehicle_ui::VehicleAtlasFrameSpec,
    },
};

#[cfg(test)]
pub(crate) const TURRENT_TIME_INTERVAL: f32 = 1.0;
#[cfg(test)]
pub(crate) const HOOK_IDLE_FRAME_TIME: f32 = 0.7;
#[cfg(test)]
pub(crate) const HOOK_ACTIVE_FRAME_TIME: f32 = 0.01;
pub(crate) const CONCO_TRAVEL_TIME: f32 = 0.8;
pub(crate) const CONCO_SIZE: Vec2 = Vec2::new(32.0, 16.0);
pub(crate) const CONCO_CONE_SIZE: Vec2 = Vec2::new(16.0, 16.0);
pub(crate) const CONCO_SIGN_SIZE: Vec2 = Vec2::new(16.0, 16.0);
pub(crate) const CONCO_BOT_SIZE: Vec2 = Vec2::new(16.0, 16.0);
const CONCO_CONCO_DIST_FROM_ENTRANCE: f32 = 12.0;
const CONCO_CONE_DIST_FROM_ENTRANCE: f32 = 6.0;
const CONCO_CONE_DIST_FROM_CENTER: f32 = 18.0;
const CONCO_SIGN_DIST_FROM_CONCO: f32 = 6.0;
pub(crate) const CONCO_BOT_DIST_FROM_ENTRANCE: f32 = 16.0;
pub(crate) const CONCO_BOT_DIST_FROM_ENTRANCE_BOX: usize = 32;

#[cfg(test)]
const TOP_OFFSET_X: [f32; 8] = [-6.0, -3.0, 0.0, 3.0, 6.0, 1.0, 0.0, -2.0];
#[cfg(test)]
const TOP_OFFSET_Y: [f32; 8] = [-6.0, -4.0, -5.0, -4.0, -6.0, -8.0, -9.0, -8.0];
#[cfg(test)]
const HOOK_OFFSET_X: [f32; 8] = [0.0, 4.0, 14.0, 23.0, 25.0, 21.0, 14.0, 5.0];
#[cfg(test)]
const HOOK_OFFSET_Y: [f32; 8] = [14.0, 20.0, 23.0, 20.0, 14.0, 8.0, 5.0, 8.0];
#[cfg(test)]
const TOP_ROTATION: [u16; 8] = [180, 225, 270, 315, 0, 45, 90, 135];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CraneConcoPhase {
    TravelingTo,
    Working,
    Returning,
}

#[derive(Component)]
pub(crate) struct CraneConcoEffect {
    pub(crate) crane_ref_id: u32,
    pub(crate) team: TeamType,
    pub(crate) phase: CraneConcoPhase,
    pub(crate) elapsed: f32,
    pub(crate) frame: usize,
    pub(crate) jackbot_i: usize,
    pub(crate) next_jackbot_time: f32,
    pub(crate) pbot_i: usize,
    pub(crate) pbot_pointing: bool,
    pub(crate) next_pbot_time: f32,
}

impl CraneConcoEffect {
    pub(crate) fn new(crane_ref_id: u32, team: TeamType) -> Self {
        Self {
            crane_ref_id,
            team,
            phase: CraneConcoPhase::TravelingTo,
            elapsed: 0.0,
            frame: 7,
            jackbot_i: 0,
            next_jackbot_time: 0.0,
            pbot_i: 0,
            pbot_pointing: false,
            next_pbot_time: 0.0,
        }
    }
}

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn hud_name() -> &'static str {
    "crane"
}

#[derive(Component)]
pub(crate) struct CraneConcoPart {
    pub(crate) crane_ref_id: u32,
    pub(crate) item: CraneConcoRenderItem,
    pub(crate) start_map: Vec2,
    pub(crate) dest_map: Vec2,
    pub(crate) current_map: Vec2,
    pub(crate) size: Vec2,
    pub(crate) w_dist: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CraneConcoRenderItem {
    Conco,
    Cone0,
    Cone1,
    Jack,
    Paper,
    Sign,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CraneConcoSurface {
    Conco(usize),
    ConeNoShadow,
    Cone,
    RobotJackhammer(usize),
    RobotPaper(usize),
    RobotPoint(usize),
    RobotTravelRight,
    RobotTravelLeft,
    RobotTravelUpDown,
    SignFlip(usize),
    Sign,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CraneConcoItemDestination {
    pub(crate) item: CraneConcoRenderItem,
    pub(crate) top_left_map: Vec2,
    pub(crate) size: Vec2,
}

#[derive(Clone, Copy)]
pub(crate) struct CraneConcoTargetSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) top_left_map: Vec2,
    pub(crate) size: Vec2,
    pub(crate) is_bridge: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct CraneConcoCraneSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) team: TeamType,
    pub(crate) top_left_map: Vec2,
    pub(crate) target: Option<CraneConcoTargetSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub(crate) struct CraneConcoVisualTarget {
    pub(crate) ref_id: u32,
    pub(crate) top_left_map: Vec2,
    pub(crate) size: Vec2,
    pub(crate) is_bridge: bool,
}

pub(crate) fn conco_target_snapshot(
    target: Option<&CraneConcoVisualTarget>,
) -> Option<CraneConcoTargetSnapshot> {
    let target = target?;
    Some(CraneConcoTargetSnapshot {
        ref_id: target.ref_id,
        top_left_map: target.top_left_map,
        size: target.size,
        is_bridge: target.is_bridge,
    })
}

pub(crate) fn conco_crane_snapshot(
    kind: ObjectKind,
    ref_id: u32,
    team: TeamType,
    top_left_map: Vec2,
    target: Option<&CraneConcoVisualTarget>,
) -> Option<CraneConcoCraneSnapshot> {
    matches!(kind, ObjectKind::Vehicle(VehicleType::Crane)).then_some(CraneConcoCraneSnapshot {
        ref_id,
        team,
        top_left_map,
        target: conco_target_snapshot(target),
    })
}

pub(crate) fn movement_profile() -> vehicles::VehicleMovementProfile {
    vehicles::VehicleMovementProfile {
        base_frame_count: 3,
        base_frame_time: vehicles::VEHICLE_BASE_FRAME_TIME,
        track_kind: VehicleTrackKind::Tank,
        drops_damage_effects: true,
    }
}

pub(crate) fn base_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> VehicleAtlasFrameSpec {
    vehicles::vehicle_ui::base_atlas_frame_spec_for_folder(
        "crane",
        movement_profile().base_frame_count,
        team,
        rotation,
        frame,
    )
}

pub(crate) fn top_atlas_frame_spec(
    _team: TeamType,
    _rotation: u16,
) -> Option<VehicleAtlasFrameSpec> {
    None
}

pub(crate) fn damage_profile() -> VehicleDamageProfile {
    VehicleDamageProfile {
        missile_visual: None,
        missile_frames: None,
        impact: None,
    }
}

pub(crate) fn death_effect_bounds() -> VehicleDeathEffectBounds {
    VehicleDeathEffectBounds {
        x: 4,
        y: 9,
        width: 23,
        height: 19,
    }
}

pub(crate) fn death_profile() -> vehicles::VehicleDeathProfile {
    vehicles::VehicleDeathProfile {
        effect_bounds: death_effect_bounds(),
        wreck: VehicleDeathWreckProfile::Static,
        destroyed_asset: VehicleDestroyedAssetProfile::TeamStatic,
        turrent: None,
    }
}

pub(crate) fn death_wreck_asset_path() -> String {
    "units/vehicles/crane/wasted_null.png".to_string()
}

pub(crate) fn destroyed_asset_path(team: TeamType) -> Option<String> {
    let team = if team == TeamType::Null {
        "null"
    } else {
        match team.atlas_team() {
            TeamType::Blue => "blue",
            TeamType::Green => "green",
            TeamType::Yellow => "yellow",
            _ => "red",
        }
    };
    Some(format!("units/vehicles/crane/wasted_{team}.png"))
}

pub(crate) fn track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicles::tank_track_effect_frame_paths(planet, direction)
}

#[cfg(test)]
pub(crate) fn top_asset_path(direction: usize) -> String {
    let rotation = TOP_ROTATION[direction.min(7)];
    format!("units/vehicles/crane/crane_r{rotation:03}.png")
}

#[cfg(test)]
pub(crate) fn hook_asset_path(frame: usize) -> String {
    let frame = frame % 16;
    let source_frame = if frame < 8 { frame } else { 15 - frame };
    format!("units/vehicles/crane/hook_n{source_frame:02}.png")
}

#[cfg(test)]
pub(crate) fn top_offset(direction: usize) -> Vec2 {
    let direction = direction.min(7);
    Vec2::new(TOP_OFFSET_X[direction], TOP_OFFSET_Y[direction])
}

#[cfg(test)]
pub(crate) fn hook_offset(direction: usize) -> Vec2 {
    let direction = direction.min(7);
    top_offset(direction) + Vec2::new(HOOK_OFFSET_X[direction], HOOK_OFFSET_Y[direction])
}

pub(crate) fn surface_asset_path(team: TeamType, surface: CraneConcoSurface) -> String {
    let team = team.atlas_team().asset_name();
    match surface {
        CraneConcoSurface::Conco(frame) => {
            format!(
                "units/vehicles/crane/effects/conco_{team}_n{:02}.png",
                frame.min(7)
            )
        }
        CraneConcoSurface::ConeNoShadow => {
            format!("units/vehicles/crane/effects/cone_no_shadow_{team}.png")
        }
        CraneConcoSurface::Cone => format!("units/vehicles/crane/effects/cone_{team}.png"),
        CraneConcoSurface::RobotJackhammer(frame) => format!(
            "units/vehicles/crane/effects/robot_jackhammer_{team}_n{:02}.png",
            frame.min(1)
        ),
        CraneConcoSurface::RobotPaper(frame) => format!(
            "units/vehicles/crane/effects/robot_paper_{team}_n{:02}.png",
            frame.min(1)
        ),
        CraneConcoSurface::RobotPoint(frame) => format!(
            "units/vehicles/crane/effects/robot_point_{team}_n{:02}.png",
            frame.min(2)
        ),
        CraneConcoSurface::RobotTravelRight => {
            format!("units/vehicles/crane/effects/robot_travel_right_{team}.png")
        }
        CraneConcoSurface::RobotTravelLeft => {
            format!("units/vehicles/crane/effects/robot_travel_left_{team}.png")
        }
        CraneConcoSurface::RobotTravelUpDown => {
            format!("units/vehicles/crane/effects/robot_travel_updown_{team}.png")
        }
        CraneConcoSurface::SignFlip(frame) => format!(
            "units/vehicles/crane/effects/sign_flip_{team}_n{:02}.png",
            frame.min(7)
        ),
        CraneConcoSurface::Sign => format!("units/vehicles/crane/effects/sign_{team}.png"),
    }
}

pub(crate) fn travel_frame(progress: f32, travel_back: bool) -> usize {
    let progress = progress.clamp(0.0, 1.0);
    let frame = if travel_back {
        7.0 * progress
    } else {
        7.0 * (1.0 - progress)
    };
    frame.clamp(0.0, 7.0) as usize
}

pub(crate) fn static_destinations(
    crane_top_left_map: Vec2,
    target_top_left_map: Vec2,
    target_size: Vec2,
    is_bridge: bool,
) -> Vec<CraneConcoItemDestination> {
    let crane_center = crane_top_left_map + Vec2::splat(16.0);
    let target_center = target_top_left_map + target_size * 0.5;
    let mut items = Vec::with_capacity(4);

    let (conco, cone0, cone1, sign) = if !is_bridge {
        let conco = Vec2::new(
            target_center.x - CONCO_SIZE.x * 0.5,
            target_top_left_map.y + target_size.y + CONCO_CONCO_DIST_FROM_ENTRANCE,
        );
        let cone_y = target_top_left_map.y + target_size.y + CONCO_CONE_DIST_FROM_ENTRANCE;
        let cone0 = Vec2::new(
            target_center.x - (CONCO_CONE_SIZE.x + CONCO_CONE_DIST_FROM_CENTER),
            cone_y,
        );
        let cone1 = Vec2::new(target_center.x + CONCO_CONE_DIST_FROM_CENTER, cone_y);
        let sign = Vec2::new(
            (conco.x + CONCO_SIZE.x * 0.5) - CONCO_SIGN_SIZE.x * 0.5,
            conco.y - (CONCO_SIGN_SIZE.y + 1.0),
        );
        (conco, cone0, cone1, sign)
    } else if target_size.x > target_size.y {
        if crane_center.x > target_center.x {
            let conco = Vec2::new(
                target_top_left_map.x + target_size.x + CONCO_CONCO_DIST_FROM_ENTRANCE,
                target_center.y - CONCO_SIZE.y * 0.5,
            );
            let cone_x = target_top_left_map.x + target_size.x + CONCO_CONE_DIST_FROM_ENTRANCE;
            let cone0 = Vec2::new(
                cone_x,
                target_center.y - (CONCO_CONE_SIZE.y + CONCO_CONE_DIST_FROM_CENTER),
            );
            let cone1 = Vec2::new(cone_x, target_center.y + CONCO_CONE_DIST_FROM_CENTER);
            let sign = Vec2::new(
                conco.x - (CONCO_SIGN_DIST_FROM_CONCO + CONCO_SIGN_SIZE.x),
                (conco.y + CONCO_SIZE.y * 0.5) - CONCO_SIGN_SIZE.y * 0.5,
            );
            (conco, cone0, cone1, sign)
        } else {
            let conco = Vec2::new(
                target_top_left_map.x - (CONCO_CONCO_DIST_FROM_ENTRANCE + CONCO_SIZE.x),
                target_center.y - CONCO_SIZE.y * 0.5,
            );
            let cone_x =
                target_top_left_map.x - (CONCO_CONE_DIST_FROM_ENTRANCE + CONCO_CONE_SIZE.x);
            let cone0 = Vec2::new(
                cone_x,
                target_center.y - (CONCO_CONE_SIZE.y + CONCO_CONE_DIST_FROM_CENTER),
            );
            let cone1 = Vec2::new(cone_x, target_center.y + CONCO_CONE_DIST_FROM_CENTER);
            let sign = Vec2::new(
                conco.x + CONCO_SIZE.x + CONCO_SIGN_DIST_FROM_CONCO,
                (conco.y + CONCO_SIZE.y * 0.5) - CONCO_SIGN_SIZE.y * 0.5,
            );
            (conco, cone0, cone1, sign)
        }
    } else if crane_center.y < target_center.y {
        let conco = Vec2::new(
            target_center.x - CONCO_SIZE.x * 0.5,
            target_top_left_map.y - (CONCO_CONCO_DIST_FROM_ENTRANCE + CONCO_SIZE.y),
        );
        let cone_y = target_top_left_map.y - (CONCO_CONE_DIST_FROM_ENTRANCE + CONCO_CONE_SIZE.y);
        let cone0 = Vec2::new(
            target_center.x - (CONCO_CONE_SIZE.x + CONCO_CONE_DIST_FROM_CENTER),
            cone_y,
        );
        let cone1 = Vec2::new(target_center.x + CONCO_CONE_DIST_FROM_CENTER, cone_y);
        let sign = Vec2::new(
            (conco.x + CONCO_SIZE.x * 0.5) - CONCO_SIGN_SIZE.x * 0.5,
            conco.y - (CONCO_SIGN_SIZE.y + 1.0),
        );
        (conco, cone0, cone1, sign)
    } else {
        let conco = Vec2::new(
            target_center.x - CONCO_SIZE.x * 0.5,
            target_top_left_map.y + target_size.y + CONCO_CONCO_DIST_FROM_ENTRANCE,
        );
        let cone_y = target_top_left_map.y + target_size.y + CONCO_CONE_DIST_FROM_ENTRANCE;
        let cone0 = Vec2::new(
            target_center.x - (CONCO_CONE_SIZE.x + CONCO_CONE_DIST_FROM_CENTER),
            cone_y,
        );
        let cone1 = Vec2::new(target_center.x + CONCO_CONE_DIST_FROM_CENTER, cone_y);
        let sign = Vec2::new(
            (conco.x + CONCO_SIZE.x * 0.5) - CONCO_SIGN_SIZE.x * 0.5,
            conco.y - (CONCO_SIGN_SIZE.y + 1.0),
        );
        (conco, cone0, cone1, sign)
    };

    items.push(CraneConcoItemDestination {
        item: CraneConcoRenderItem::Conco,
        top_left_map: conco,
        size: CONCO_SIZE,
    });
    items.push(CraneConcoItemDestination {
        item: CraneConcoRenderItem::Cone0,
        top_left_map: cone0,
        size: CONCO_CONE_SIZE,
    });
    items.push(CraneConcoItemDestination {
        item: CraneConcoRenderItem::Cone1,
        top_left_map: cone1,
        size: CONCO_CONE_SIZE,
    });
    items.push(CraneConcoItemDestination {
        item: CraneConcoRenderItem::Sign,
        top_left_map: sign,
        size: CONCO_SIGN_SIZE,
    });
    items
}

pub(crate) fn advance_bots(effect: &mut CraneConcoEffect, delta: f32, rng: &mut CombatRng) {
    effect.next_jackbot_time -= delta;
    if effect.next_jackbot_time <= 0.0 {
        effect.next_jackbot_time = 0.045 + rng.index(20) as f32 * 0.001;
        effect.jackbot_i = usize::from(effect.jackbot_i == 0);
    }

    effect.next_pbot_time -= delta;
    if effect.next_pbot_time <= 0.0 {
        effect.next_pbot_time = 0.15 + rng.index(20) as f32 * 0.01;
        if effect.pbot_pointing {
            if effect.pbot_i >= 2 {
                effect.pbot_i = 0;
                effect.pbot_pointing = false;
            } else {
                effect.pbot_i += 1;
                effect.next_pbot_time = 0.3 + rng.index(30) as f32 * 0.01;
            }
        } else if rng.index(10) == 0 {
            effect.pbot_i = 0;
            effect.pbot_pointing = true;
        } else {
            effect.pbot_i = usize::from(effect.pbot_i == 0);
        }
    }
}

pub(crate) fn phase_progress(phase: CraneConcoPhase, frame: usize) -> f32 {
    match phase {
        CraneConcoPhase::TravelingTo => 1.0 - frame as f32 / 7.0,
        CraneConcoPhase::Working => 1.0,
        CraneConcoPhase::Returning => frame as f32 / 7.0,
    }
}

pub(crate) fn surface_for_part(
    item: CraneConcoRenderItem,
    phase: CraneConcoPhase,
    frame: usize,
    jackbot_i: usize,
    pbot_i: usize,
    pbot_pointing: bool,
    w_dist: f32,
) -> CraneConcoSurface {
    let traveling = phase != CraneConcoPhase::Working;
    match item {
        CraneConcoRenderItem::Conco => CraneConcoSurface::Conco(frame),
        CraneConcoRenderItem::Sign if traveling => CraneConcoSurface::SignFlip(frame),
        CraneConcoRenderItem::Sign => CraneConcoSurface::Sign,
        CraneConcoRenderItem::Cone0 | CraneConcoRenderItem::Cone1 if traveling => {
            CraneConcoSurface::ConeNoShadow
        }
        CraneConcoRenderItem::Cone0 | CraneConcoRenderItem::Cone1 => CraneConcoSurface::Cone,
        CraneConcoRenderItem::Jack if traveling => travel_robot_surface(w_dist),
        CraneConcoRenderItem::Jack => CraneConcoSurface::RobotJackhammer(jackbot_i),
        CraneConcoRenderItem::Paper if traveling => travel_robot_surface(w_dist),
        CraneConcoRenderItem::Paper if pbot_pointing => CraneConcoSurface::RobotPoint(pbot_i),
        CraneConcoRenderItem::Paper => CraneConcoSurface::RobotPaper(pbot_i),
    }
}

pub(crate) fn bot_destination(
    crane_top_left_map: Vec2,
    target_top_left_map: Vec2,
    target_size: Vec2,
    is_bridge: bool,
    rng: &mut CombatRng,
) -> Vec2 {
    if target_size.x <= CONCO_BOT_SIZE.x || target_size.y <= CONCO_BOT_SIZE.y {
        return start_top_left(crane_top_left_map);
    }

    let target_center = target_top_left_map + target_size * 0.5;
    let crane_center = crane_top_left_map + Vec2::splat(16.0);
    let x_span = (target_size.x - CONCO_BOT_SIZE.x).max(1.0) as usize;
    let y_span = (target_size.y - CONCO_BOT_SIZE.y).max(1.0) as usize;

    if !is_bridge {
        return Vec2::new(
            target_top_left_map.x + rng.index(x_span) as f32,
            target_top_left_map.y
                + target_size.y
                + CONCO_BOT_DIST_FROM_ENTRANCE
                + rng.index(CONCO_BOT_DIST_FROM_ENTRANCE_BOX) as f32,
        );
    }

    if target_size.x > target_size.y {
        if rng.index(2) != 0 {
            if crane_center.x > target_center.x {
                Vec2::new(
                    target_top_left_map.x
                        + target_size.x
                        + CONCO_BOT_DIST_FROM_ENTRANCE
                        + rng.index(CONCO_BOT_DIST_FROM_ENTRANCE_BOX) as f32,
                    target_top_left_map.y + rng.index(y_span) as f32,
                )
            } else {
                Vec2::new(
                    target_top_left_map.x
                        - (CONCO_BOT_DIST_FROM_ENTRANCE
                            + rng.index(CONCO_BOT_DIST_FROM_ENTRANCE_BOX) as f32),
                    target_top_left_map.y + rng.index(y_span) as f32,
                )
            }
        } else {
            Vec2::new(
                target_top_left_map.x + rng.index(x_span) as f32,
                target_top_left_map.y + 16.0 + rng.index(16) as f32,
            )
        }
    } else if rng.index(2) != 0 {
        if crane_center.y < target_center.y {
            Vec2::new(
                target_top_left_map.x + rng.index(x_span) as f32,
                target_top_left_map.y - rng.index(CONCO_BOT_DIST_FROM_ENTRANCE_BOX) as f32,
            )
        } else {
            Vec2::new(
                target_top_left_map.x + rng.index(x_span) as f32,
                target_top_left_map.y
                    + target_size.y
                    + CONCO_BOT_DIST_FROM_ENTRANCE
                    + rng.index(CONCO_BOT_DIST_FROM_ENTRANCE_BOX) as f32,
            )
        }
    } else {
        Vec2::new(
            target_top_left_map.x + 16.0 + rng.index(16) as f32,
            target_top_left_map.y + rng.index(y_span) as f32,
        )
    }
}

pub(crate) fn start_top_left(crane_top_left_map: Vec2) -> Vec2 {
    crane_top_left_map + Vec2::splat(16.0)
}

pub(crate) fn return_top_left(crane_top_left_map: Vec2, size: Vec2) -> Vec2 {
    crane_top_left_map + Vec2::splat(16.0) - size * 0.5
}

pub(crate) fn z(top_left_map: Vec2, size: Vec2) -> f32 {
    36.0 + (top_left_map.y + size.y) * 0.001
}

fn travel_robot_surface(w_dist: f32) -> CraneConcoSurface {
    if w_dist > 0.0 {
        CraneConcoSurface::RobotTravelRight
    } else if w_dist < 0.0 {
        CraneConcoSurface::RobotTravelLeft
    } else {
        CraneConcoSurface::RobotTravelUpDown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::CombatRng, original::types::TeamType};
    use bevy::prelude::Vec2;

    #[test]
    fn crane_conco_asset_paths_match_original_effect_names() {
        assert_eq!(
            surface_asset_path(TeamType::Blue, CraneConcoSurface::Conco(9)),
            "units/vehicles/crane/effects/conco_blue_n07.png"
        );
        assert_eq!(
            surface_asset_path(TeamType::Green, CraneConcoSurface::SignFlip(3)),
            "units/vehicles/crane/effects/sign_flip_green_n03.png"
        );
        assert_eq!(
            surface_asset_path(TeamType::Yellow, CraneConcoSurface::ConeNoShadow),
            "units/vehicles/crane/effects/cone_no_shadow_yellow.png"
        );
        assert_eq!(
            surface_asset_path(TeamType::Red, CraneConcoSurface::RobotJackhammer(4)),
            "units/vehicles/crane/effects/robot_jackhammer_red_n01.png"
        );
        assert_eq!(
            surface_asset_path(TeamType::Red, CraneConcoSurface::RobotPoint(7)),
            "units/vehicles/crane/effects/robot_point_red_n02.png"
        );
    }

    #[test]
    fn crane_conco_travel_frames_match_original_progress_math() {
        assert_eq!(CONCO_TRAVEL_TIME, 0.8);
        assert_eq!(travel_frame(0.0, false), 7);
        assert_eq!(travel_frame(0.5, false), 3);
        assert_eq!(travel_frame(1.0, false), 0);
        assert_eq!(travel_frame(0.0, true), 0);
        assert_eq!(travel_frame(0.5, true), 3);
        assert_eq!(travel_frame(1.0, true), 7);
    }

    #[test]
    fn crane_conco_building_destinations_match_original_layout() {
        let destinations = static_destinations(
            Vec2::new(10.0, 10.0),
            Vec2::new(100.0, 200.0),
            Vec2::new(64.0, 48.0),
            false,
        );

        assert_eq!(
            destinations,
            vec![
                CraneConcoItemDestination {
                    item: CraneConcoRenderItem::Conco,
                    top_left_map: Vec2::new(116.0, 260.0),
                    size: CONCO_SIZE,
                },
                CraneConcoItemDestination {
                    item: CraneConcoRenderItem::Cone0,
                    top_left_map: Vec2::new(98.0, 254.0),
                    size: CONCO_CONE_SIZE,
                },
                CraneConcoItemDestination {
                    item: CraneConcoRenderItem::Cone1,
                    top_left_map: Vec2::new(150.0, 254.0),
                    size: CONCO_CONE_SIZE,
                },
                CraneConcoItemDestination {
                    item: CraneConcoRenderItem::Sign,
                    top_left_map: Vec2::new(124.0, 243.0),
                    size: CONCO_SIGN_SIZE,
                },
            ]
        );
    }

    #[test]
    fn crane_conco_bridge_destinations_match_original_side_layouts() {
        let horizontal_from_right = static_destinations(
            Vec2::new(240.0, 112.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(80.0, 32.0),
            true,
        );
        assert_eq!(
            horizontal_from_right
                .iter()
                .find(|item| item.item == CraneConcoRenderItem::Conco)
                .unwrap()
                .top_left_map,
            Vec2::new(192.0, 108.0)
        );
        assert_eq!(
            horizontal_from_right
                .iter()
                .find(|item| item.item == CraneConcoRenderItem::Sign)
                .unwrap()
                .top_left_map,
            Vec2::new(170.0, 108.0)
        );

        let vertical_from_top = static_destinations(
            Vec2::new(128.0, 20.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(32.0, 80.0),
            true,
        );
        assert_eq!(
            vertical_from_top
                .iter()
                .find(|item| item.item == CraneConcoRenderItem::Conco)
                .unwrap()
                .top_left_map,
            Vec2::new(100.0, 72.0)
        );
        assert_eq!(
            vertical_from_top
                .iter()
                .find(|item| item.item == CraneConcoRenderItem::Cone0)
                .unwrap()
                .top_left_map,
            Vec2::new(82.0, 78.0)
        );
    }

    #[test]
    fn crane_conco_runtime_surfaces_match_original_phases() {
        assert_eq!(
            surface_for_part(
                CraneConcoRenderItem::Conco,
                CraneConcoPhase::TravelingTo,
                7,
                0,
                0,
                false,
                0.0
            ),
            CraneConcoSurface::Conco(7)
        );
        assert_eq!(
            surface_for_part(
                CraneConcoRenderItem::Sign,
                CraneConcoPhase::TravelingTo,
                3,
                0,
                0,
                false,
                0.0
            ),
            CraneConcoSurface::SignFlip(3)
        );
        assert_eq!(
            surface_for_part(
                CraneConcoRenderItem::Cone0,
                CraneConcoPhase::Working,
                0,
                0,
                0,
                false,
                0.0
            ),
            CraneConcoSurface::Cone
        );
        assert_eq!(
            surface_for_part(
                CraneConcoRenderItem::Jack,
                CraneConcoPhase::TravelingTo,
                0,
                0,
                0,
                false,
                12.0
            ),
            CraneConcoSurface::RobotTravelRight
        );
        assert_eq!(
            surface_for_part(
                CraneConcoRenderItem::Paper,
                CraneConcoPhase::Working,
                0,
                0,
                2,
                true,
                0.0
            ),
            CraneConcoSurface::RobotPoint(2)
        );
    }

    #[test]
    fn crane_conco_return_points_match_original_begin_death() {
        let crane_top_left = Vec2::new(100.0, 200.0);

        assert_eq!(start_top_left(crane_top_left), Vec2::new(116.0, 216.0));
        assert_eq!(
            return_top_left(crane_top_left, CONCO_SIZE),
            Vec2::new(100.0, 208.0)
        );
        assert_eq!(
            return_top_left(crane_top_left, CONCO_BOT_SIZE),
            Vec2::new(108.0, 208.0)
        );
    }

    #[test]
    fn crane_conco_bot_destinations_stay_in_original_repair_regions() {
        let mut rng = CombatRng(2);
        let target_top_left = Vec2::new(100.0, 200.0);
        let target_size = Vec2::new(64.0, 48.0);
        let bot = bot_destination(
            Vec2::new(10.0, 10.0),
            target_top_left,
            target_size,
            false,
            &mut rng,
        );
        assert!(bot.x >= target_top_left.x);
        assert!(bot.x < target_top_left.x + target_size.x - CONCO_BOT_SIZE.x);
        assert!(bot.y >= target_top_left.y + target_size.y + CONCO_BOT_DIST_FROM_ENTRANCE);
        assert!(
            bot.y
                < target_top_left.y
                    + target_size.y
                    + CONCO_BOT_DIST_FROM_ENTRANCE
                    + CONCO_BOT_DIST_FROM_ENTRANCE_BOX as f32
        );
    }

    #[test]
    fn crane_top_and_hook_profiles_match_original_vcrane_assets() {
        assert_eq!(TURRENT_TIME_INTERVAL, 1.0);
        assert_eq!(HOOK_IDLE_FRAME_TIME, 0.7);
        assert_eq!(HOOK_ACTIVE_FRAME_TIME, 0.01);
        assert_eq!(top_asset_path(0), "units/vehicles/crane/crane_r180.png");
        assert_eq!(top_asset_path(4), "units/vehicles/crane/crane_r000.png");
        assert_eq!(hook_asset_path(0), "units/vehicles/crane/hook_n00.png");
        assert_eq!(hook_asset_path(7), "units/vehicles/crane/hook_n07.png");
        assert_eq!(hook_asset_path(8), "units/vehicles/crane/hook_n07.png");
        assert_eq!(hook_asset_path(15), "units/vehicles/crane/hook_n00.png");
        assert_eq!(top_offset(0), Vec2::new(-6.0, -6.0));
        assert_eq!(hook_offset(0), Vec2::new(-6.0, 8.0));
        assert_eq!(top_offset(6), Vec2::new(0.0, -9.0));
        assert_eq!(hook_offset(6), Vec2::new(14.0, -4.0));
    }

    #[test]
    fn crane_wasted_assets_stay_with_crane_profile() {
        assert_eq!(
            death_wreck_asset_path(),
            "units/vehicles/crane/wasted_null.png"
        );
        assert_eq!(
            destroyed_asset_path(TeamType::Yellow).unwrap(),
            "units/vehicles/crane/wasted_yellow.png"
        );
        assert_eq!(
            destroyed_asset_path(TeamType::Null).unwrap(),
            "units/vehicles/crane/wasted_null.png"
        );
    }
}
