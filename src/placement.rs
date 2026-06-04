use bevy::prelude::*;

use crate::{
    camera::cursor_world_position,
    components::{
        BuildingProduction, CannonPlacementState, CurrentMap, GameObjectEntity, HudLayout,
        MainCamera, MapGridPosition, NextObjectRefId, ObjectStats, ObjectTeam,
        ProductionWindowState, Selectable, area_is_fort_turret_tile,
    },
    constants::{HUD_HEIGHT, HUD_WIDTH, TILE_SIZE},
    original::{
        map::{MapObjectType, ZMap},
        objects::{BuildingType, ObjectKind},
        types::TeamType,
    },
    production::remove_stored_cannon,
    render::atlas::GameAtlases,
    world_objects::spawn_runtime_object,
    zones::zone_at_tile,
};

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
        .unwrap_or_else(|| fallback_collision_size(object.kind))
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
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    game_atlases: Res<GameAtlases>,
    map: Res<CurrentMap>,
    hud_layout: Res<HudLayout>,
    production_window: Res<ProductionWindowState>,
    mut next_ref: ResMut<NextObjectRefId>,
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
            &mut BuildingProduction,
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

    let mut placed = false;
    for (object, grid, team, mut production) in &mut queries.p1() {
        if object.ref_id != request.source_ref_id {
            continue;
        }

        let source_zone = zone_at_tile(&map.0, grid.x, grid.y);
        if place_stored_cannon(
            &mut production,
            request.cannon,
            request.source_ref_id,
            team.0,
            source_zone,
            target_tile,
            &map.0,
            &obstacles,
        ) {
            let ref_id = next_ref.0;
            next_ref.0 += 1;
            placed = spawn_runtime_object(
                &mut commands,
                &game_atlases,
                map.0.basics.terrain_type,
                &hud_layout,
                ref_id,
                request.cannon,
                team.0,
                cannon_spawn_center(target_tile.0, target_tile.1),
                100,
                !area_is_fort_turret_tile(&map.0, target_tile.0, target_tile.1),
                false,
                None,
                None,
            );
        }
        break;
    }

    if placed {
        placement.pending = None;
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
    if !matches!(
        obstacle.kind,
        ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack)
    ) {
        return false;
    }

    let obstacle_left = obstacle.center.x - obstacle.size.x * 0.5;
    let obstacle_top = -obstacle.center.y - obstacle.size.y * 0.5;
    let local_x = (map_left - obstacle_left).round();
    let local_y = (map_top - obstacle_top).round();

    (local_x == TILE_SIZE && (local_y == 0.0 || local_y == TILE_SIZE * 3.0))
        || (local_x == TILE_SIZE * 7.0 && (local_y == 0.0 || local_y == TILE_SIZE * 3.0))
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

fn fallback_collision_size(kind: ObjectKind) -> Vec2 {
    match kind {
        ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack) => {
            Vec2::new(TILE_SIZE * 10.0, TILE_SIZE * 5.0)
        }
        ObjectKind::Building(_) | ObjectKind::Bridge(_) => Vec2::splat(TILE_SIZE * 3.0),
        ObjectKind::Cannon(_) => Vec2::splat(TILE_SIZE * 2.0),
        ObjectKind::Rock | ObjectKind::MapItem(_) => Vec2::splat(TILE_SIZE),
        ObjectKind::Vehicle(_) | ObjectKind::Robot(_) | ObjectKind::Animal(_) => Vec2::ZERO,
    }
}

#[allow(dead_code)]
fn map_object_type_for_collision(kind: ObjectKind) -> Option<MapObjectType> {
    match kind {
        ObjectKind::Bridge(_) => Some(MapObjectType::Bridge),
        ObjectKind::Building(_) => Some(MapObjectType::Building),
        ObjectKind::Cannon(_) => Some(MapObjectType::Cannon),
        ObjectKind::Rock | ObjectKind::MapItem(_) => Some(MapObjectType::MapItem),
        ObjectKind::Vehicle(_) | ObjectKind::Robot(_) | ObjectKind::Animal(_) => None,
    }
}
