use bevy::prelude::*;

use crate::{
    camera::cursor_world_position,
    components::{
        BuildingProduction, CannonPlacementState, CurrentMap, GameObjectEntity, MainCamera,
        MapGridPosition, NextObjectRefId, ObjectStats, ObjectTeam, ProductionWindowState,
        Selectable,
    },
    constants::{HUD_HEIGHT, HUD_WIDTH, TILE_SIZE},
    local_player::LocalPlayerState,
    object_sync::{SourceObjectEventQueue, relay_built_cannon_list, relay_new_object},
    original::{map::ZMap, objects::ObjectKind, types::TeamType},
    units::{self, buildings},
    zones::zone_at_tile,
};

#[cfg(test)]
use crate::production::remove_stored_cannon;

#[derive(Clone, Copy, Debug)]
pub(crate) struct PlacementObstacle {
    pub(crate) ref_id: u32,
    pub(crate) kind: ObjectKind,
    pub(crate) center: Vec2,
    pub(crate) size: Vec2,
}

pub(crate) fn first_stored_cannon(production: &BuildingProduction) -> Option<ObjectKind> {
    production
        .stored_cannons
        .iter()
        .copied()
        .find(|unit| matches!(unit, ObjectKind::Cannon(_)))
}

pub(crate) fn cannon_placement_tile(world_pos: Vec2) -> Option<(i32, i32)> {
    if !world_pos.is_finite() {
        return None;
    }

    Some((
        (world_pos.x / TILE_SIZE).floor() as i32,
        ((-world_pos.y) / TILE_SIZE).floor() as i32,
    ))
}

pub(crate) fn cannon_spawn_center(tx: i32, ty: i32) -> Vec2 {
    Vec2::new(
        tx as f32 * TILE_SIZE + TILE_SIZE,
        -(ty as f32 * TILE_SIZE + TILE_SIZE),
    )
}

#[cfg(test)]
pub(crate) fn place_stored_cannon(
    production: &mut BuildingProduction,
    cannon: ObjectKind,
    source_ref_id: u32,
    source_team: TeamType,
    source_zone: Option<usize>,
    target_tile: (i32, i32),
    map: &ZMap,
    obstacles: &[PlacementObstacle],
) -> bool {
    if !can_place_stored_cannon(
        production,
        cannon,
        source_ref_id,
        source_team,
        source_zone,
        target_tile,
        map,
        obstacles,
    ) {
        return false;
    }

    remove_stored_cannon(production, cannon)
}

pub(crate) fn can_place_stored_cannon(
    production: &BuildingProduction,
    cannon: ObjectKind,
    source_ref_id: u32,
    source_team: TeamType,
    source_zone: Option<usize>,
    target_tile: (i32, i32),
    map: &ZMap,
    obstacles: &[PlacementObstacle],
) -> bool {
    if !matches!(cannon, ObjectKind::Cannon(_))
        || !production.stored_cannons.contains(&cannon)
        || source_team == TeamType::Null
    {
        return false;
    }

    let Some(zone_index) = source_zone else {
        return false;
    };
    let Some(zone) = map.zones.get(zone_index) else {
        return false;
    };

    let (tx, ty) = target_tile;
    if tx < 0 || ty < 0 || tx > map.basics.width as i32 || ty > map.basics.height as i32 {
        return false;
    }

    let x = tx as f32 * TILE_SIZE;
    let y = ty as f32 * TILE_SIZE;
    let right = x + TILE_SIZE * 2.0;
    let bottom = y + TILE_SIZE * 2.0;
    let zone_x = zone.x as f32 * TILE_SIZE;
    let zone_y = zone.y as f32 * TILE_SIZE;
    let zone_w = zone.w as f32 * TILE_SIZE;
    let zone_h = zone.h as f32 * TILE_SIZE;

    if x < zone_x + TILE_SIZE
        || y < zone_y + TILE_SIZE
        || right > zone_x + zone_w - TILE_SIZE
        || bottom > zone_y + zone_h - TILE_SIZE
    {
        return false;
    }

    !obstacles.iter().any(|obstacle| {
        obstacle.ref_id != source_ref_id
            && !matches!(obstacle.kind, ObjectKind::Robot(_) | ObjectKind::Vehicle(_))
            && cannon_not_placable_for_obstacle(obstacle, x, right, y, bottom)
    })
}

pub(crate) fn placement_obstacle_from_object(
    object: &GameObjectEntity,
    team: &ObjectTeam,
    stats: &ObjectStats,
    selectable: Option<&Selectable>,
    center: Vec2,
) -> Option<PlacementObstacle> {
    if stats.destroyed() || matches!(object.kind, ObjectKind::Robot(_) | ObjectKind::Vehicle(_)) {
        return None;
    }

    let size = selectable
        .map(|selectable| selectable.selection_size)
        .unwrap_or_else(|| units::fallback_collision_size(object.kind))
        .max(Vec2::splat(TILE_SIZE));
    let _ = team;
    Some(PlacementObstacle {
        ref_id: object.ref_id,
        kind: object.kind,
        center,
        size,
    })
}

pub(crate) fn process_cannon_placement(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    map: Res<CurrentMap>,
    local_player: Res<LocalPlayerState>,
    production_window: Res<ProductionWindowState>,
    mut next_ref: ResMut<NextObjectRefId>,
    mut object_events: ResMut<SourceObjectEventQueue>,
    mut placement: ResMut<CannonPlacementState>,
    mut queries: ParamSet<(
        Query<(
            &GameObjectEntity,
            &Transform,
            &ObjectTeam,
            &ObjectStats,
            Option<&Selectable>,
        )>,
        Query<(
            &GameObjectEntity,
            &MapGridPosition,
            &ObjectTeam,
            &BuildingProduction,
        )>,
    )>,
) {
    if production_window.input_captured {
        return;
    }

    let Some(request) = placement.pending else {
        return;
    };

    if mouse.just_released(MouseButton::Right) || keys.just_pressed(KeyCode::Escape) {
        placement.pending = None;
        return;
    }

    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    placement.pending = None;

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(screen_pos) = window.cursor_position() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());
    if screen_pos.x >= window_size.x - HUD_WIDTH || screen_pos.y >= window_size.y - HUD_HEIGHT {
        return;
    }

    let Some(world_pos) = cursor_world_position(&windows, &camera_query) else {
        return;
    };
    let Some(target_tile) = cannon_placement_tile(world_pos) else {
        return;
    };

    let obstacles: Vec<PlacementObstacle> = queries
        .p0()
        .iter()
        .filter_map(|(object, transform, team, stats, selectable)| {
            placement_obstacle_from_object(
                object,
                team,
                stats,
                selectable,
                transform.translation.truncate(),
            )
        })
        .collect();
    let existing_ref_ids: Vec<u32> = queries
        .p0()
        .iter()
        .map(|(object, _, _, _, _)| object.ref_id)
        .collect();

    for (object, grid, team, production) in &queries.p1() {
        if object.ref_id != request.source_ref_id {
            continue;
        }
        if team.0 == TeamType::Null
            || team.0 != local_player.team()
            || existing_ref_ids.contains(&next_ref.0)
        {
            break;
        }

        let source_zone = zone_at_tile(&map.0, grid.x, grid.y);
        if can_place_stored_cannon(
            &production,
            request.cannon,
            request.source_ref_id,
            team.0,
            source_zone,
            target_tile,
            &map.0,
            &obstacles,
        ) {
            let mut next_stored_cannons = production.stored_cannons.clone();
            let Some(remove_index) = next_stored_cannons
                .iter()
                .position(|stored| *stored == request.cannon)
            else {
                break;
            };
            next_stored_cannons.remove(remove_index);
            let ref_id = next_ref.0;
            let spawn_center = cannon_spawn_center(target_tile.0, target_tile.1);
            let source_map_position =
                Vec2::new(spawn_center.x - TILE_SIZE, -spawn_center.y - TILE_SIZE);
            let event_checkpoint = object_events.pending.len();
            let new_object_queued = relay_new_object(
                &mut object_events,
                ref_id,
                request.cannon,
                team.0,
                source_map_position,
                0,
                0,
                100,
                false,
            );
            let cannon_list_queued = new_object_queued
                && relay_built_cannon_list(
                    &mut object_events,
                    object.ref_id,
                    &next_stored_cannons,
                    Some(ref_id),
                );
            if new_object_queued && cannon_list_queued {
                next_ref.0 += 1;
            } else {
                object_events.pending.truncate(event_checkpoint);
            }
        }
        break;
    }
}

fn cannon_not_placable_for_obstacle(
    obstacle: &PlacementObstacle,
    map_left: f32,
    map_right: f32,
    map_top: f32,
    map_bottom: f32,
) -> bool {
    if fort_turret_slot_allows(obstacle, map_left, map_top) {
        return false;
    }

    rects_overlap_top_left(
        map_left,
        map_right,
        map_top,
        map_bottom,
        obstacle.center.x - obstacle.size.x * 0.5,
        obstacle.center.x + obstacle.size.x * 0.5,
        -obstacle.center.y - obstacle.size.y * 0.5,
        -obstacle.center.y + obstacle.size.y * 0.5,
    )
}

fn fort_turret_slot_allows(obstacle: &PlacementObstacle, map_left: f32, map_top: f32) -> bool {
    buildings::fort_turret_slot_allows(
        obstacle.kind,
        obstacle.center,
        obstacle.size,
        map_left,
        map_top,
    )
}

fn rects_overlap_top_left(
    a_left: f32,
    a_right: f32,
    a_top: f32,
    a_bottom: f32,
    b_left: f32,
    b_right: f32,
    b_top: f32,
    b_bottom: f32,
) -> bool {
    a_left < b_right && a_right > b_left && a_top < b_bottom && a_bottom > b_top
}
