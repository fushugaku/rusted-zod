use bevy::prelude::*;

#[cfg(test)]
use crate::production::{production_duration, start_production};
use crate::{
    camera::cursor_world_position,
    components::*,
    constants::TILE_SIZE,
    network_commands::{CommandPayload, ComputerMessagePacket},
    original::{map::MapObject, objects::ObjectKind, types::TeamType},
    placement::first_stored_cannon,
    production::{
        add_production_queue, cancel_production_queue_item, production_duration_from_source,
        start_production_from_source, stop_production,
    },
    render::atlas::{GameAtlases, SpriteFrame},
    settings_sync::SourceSettingsState,
    units::{self, buildings},
};

const WINDOW_SIZE: Vec2 = Vec2::new(112.0, 80.0);
const WINDOW_EXPANDED_SIZE: Vec2 = Vec2::new(228.0, 96.0);
const NAME_LABEL_OFFSET: Vec2 = Vec2::new(9.0, 6.0);
const STATE_LABEL_OFFSET: Vec2 = Vec2::new(64.0, 19.0);
const NAME_LABEL_SIZE: Vec2 = Vec2::new(60.0, 10.0);
const STATE_LABEL_SIZE: Vec2 = Vec2::new(48.0, 12.0);
const BUTTON_SIZE: Vec2 = Vec2::new(40.0, 15.0);
const SELECTOR_UP_OFFSET: Vec2 = Vec2::new(50.0, 21.0);
const SELECTOR_DOWN_OFFSET: Vec2 = Vec2::new(50.0, 64.0);
const QUEUE_SELECTOR_UP_OFFSET: Vec2 = Vec2::new(158.0, 21.0);
const QUEUE_SELECTOR_DOWN_OFFSET: Vec2 = Vec2::new(158.0, 64.0);
const UNIT_SELECTOR_OFFSET: Vec2 = Vec2::new(3.0, 19.0);
const QUEUE_SELECTOR_OFFSET: Vec2 = Vec2::new(111.0, 19.0);
const SELECTOR_PORTRAIT_OFFSET: Vec2 = Vec2::new(2.0, 2.0);
const SELECTOR_PORTRAIT_SIZE: Vec2 = Vec2::new(44.0, 50.0);
const FUS_SELECTOR_CENTER_OFFSET: Vec2 = Vec2::new(24.0, 21.0);
const FUS_SIDE_SIZE: f32 = 4.0;
const FUS_TOP_HEIGHT: f32 = 20.0;
const FUS_MARGIN: f32 = 2.0;
const STARTING_MANUFACTURE_SOUND: &str = "sounds/comp_starting_manufacture.wav";
const MANUFACTURING_CANCELED_SOUND: &str = "sounds/comp_manufacturing_canceled.wav";
const FUS_OBJECT_SIZE: Vec2 = Vec2::new(45.0, 51.0);
const FUS_OBJECT_PREVIEW_CENTER: Vec2 = Vec2::new(22.0, 19.0);
const FUS_OBJECT_NAME_OFFSET: Vec2 = Vec2::new(3.0, 39.0);
const SELECTOR_ARROW_SIZE: Vec2 = Vec2::new(16.0, 8.0);
const SMALL_TOGGLE_OFFSET: Vec2 = Vec2::new(96.0, 4.0);
const SMALL_TOGGLE_SIZE: Vec2 = Vec2::new(12.0, 12.0);
const QUEUE_BUTTON_OFFSET: Vec2 = Vec2::new(111.0, 78.0);
const QUEUE_BUTTON_SIZE: Vec2 = Vec2::new(60.0, 13.0);
const QUEUE_ITEM_OFFSET: Vec2 = Vec2::new(177.0, 22.0);
const QUEUE_ITEM_SIZE: Vec2 = Vec2::new(45.0, 13.0);
const QUEUE_ITEM_STEP: f32 = 14.0;
const PROGRESS_BAR_OFFSET: Vec2 = Vec2::new(53.0, 21.0);
const PROGRESS_BAR_SIZE: Vec2 = Vec2::new(9.0, 50.0);
const PROGRESS_YELLOW_SIZE: Vec2 = Vec2::new(9.0, 39.0);
const STATE_BLINK_SECONDS: f32 = 0.3;
const SHOW_TIME_OFFSET: Vec2 = Vec2::new(90.0, 35.0);
const HEALTH_PERCENT_OFFSET: Vec2 = Vec2::new(86.0, 6.0);
const SMALL_TEXT_SIZE: f32 = 8.0;
const SELECTOR_PREVIEW_CENTER: Vec2 = Vec2::new(27.0, 40.0);
const QUEUE_SELECTOR_PREVIEW_CENTER: Vec2 = Vec2::new(135.0, 40.0);

pub(crate) fn load_production_ui_assets(asset_server: &AssetServer) -> ProductionUiAssets {
    ProductionUiAssets {
        base: asset_server.load("other/production_gui/base_image.png"),
        base_expanded: asset_server.load("other/production_gui/base_image_expanded.png"),
        labels: [
            ProductionWindowKind::Robot,
            ProductionWindowKind::Vehicle,
            ProductionWindowKind::Fort,
        ]
        .into_iter()
        .map(|kind| asset_server.load(buildings::production_label_asset_path(kind)))
        .collect(),
        state_labels: vec![
            [
                asset_server.load("other/production_gui/place_label.png"),
                asset_server.load("other/production_gui/placeless_label.png"),
            ],
            [
                asset_server.load("other/production_gui/select_label.png"),
                asset_server.load("other/production_gui/selectless_label.png"),
            ],
            [
                asset_server.load("other/production_gui/building_label.png"),
                asset_server.load("other/production_gui/buildingless_label.png"),
            ],
            [
                asset_server.load("other/production_gui/paused_label.png"),
                asset_server.load("other/production_gui/pausedless_label.png"),
            ],
        ],
        buttons: vec![
            load_production_button(asset_server, "place"),
            load_production_button(asset_server, "ok"),
            load_production_button(asset_server, "cancel"),
            load_production_button(asset_server, "up"),
            load_production_button(asset_server, "down"),
            load_production_button(asset_server, "plus_small"),
            load_production_button(asset_server, "minus_small"),
            load_production_button(asset_server, "queue"),
            load_production_button(asset_server, "up"),
            load_production_button(asset_server, "down"),
        ],
        queue_item_button: load_production_button(asset_server, "object_name"),
        object_button: load_production_button(asset_server, "object"),
        full_selector_frame: load_full_selector_frame(asset_server),
        progress_bar: asset_server.load("other/production_gui/percentage_bar.png"),
        progress_yellow: asset_server.load("other/production_gui/percentage_bar_yellow.png"),
        font: asset_server.load("arial.ttf"),
    }
}

fn load_full_selector_frame(asset_server: &AssetServer) -> ProductionFullSelectorFrameAssets {
    ProductionFullSelectorFrameAssets {
        top_left: asset_server.load("other/production_gui/fus_top_left.png"),
        top_right: asset_server.load("other/production_gui/fus_top_right.png"),
        bottom_left: asset_server.load("other/production_gui/fus_bottom_left.png"),
        bottom_right: asset_server.load("other/production_gui/fus_bottom_right.png"),
        top: asset_server.load("other/production_gui/fus_top.png"),
        bottom: asset_server.load("other/production_gui/fus_bottom.png"),
        left: asset_server.load("other/production_gui/fus_left.png"),
        right: asset_server.load("other/production_gui/fus_right.png"),
    }
}

fn load_production_button(asset_server: &AssetServer, name: &str) -> ProductionButtonImages {
    ProductionButtonImages {
        normal: asset_server.load(format!("other/production_gui/{name}_button.png")),
        pressed: asset_server.load(format!("other/production_gui/{name}_button_pressed.png")),
    }
}

pub(crate) fn open_debug_production_window(
    debug: Res<ProductionDebugOpen>,
    mut window_state: ResMut<ProductionWindowState>,
    object_query: Query<(
        &GameObjectEntity,
        &MapGridPosition,
        &ObjectTeam,
        &ObjectStats,
        Option<&BuildingProduction>,
    )>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if !debug.enabled || window_state.open.is_some() {
        return;
    }

    let target = object_query
        .iter()
        .filter_map(|(object, grid, team, stats, production)| {
            if debug.ref_id.is_some_and(|ref_id| object.ref_id != ref_id)
                || team.0 != TeamType::Red
                || stats.destroyed()
                || production.is_none()
            {
                return None;
            }

            buildings::production_window_kind(object.kind)
                .map(|window_kind| (object.ref_id, window_kind, *grid))
        })
        .next();

    let Some((building_ref_id, kind, grid)) = target else {
        return;
    };

    if let Ok(mut transform) = camera_query.single_mut() {
        transform.translation.x = grid.x as f32 * TILE_SIZE;
        transform.translation.y = -(grid.y as f32 * TILE_SIZE);
    }

    let expanded = debug.expanded
        || matches!(
            debug.full_selector,
            Some(ProductionFullSelectorTarget::Queue)
        );
    window_state.open = Some(ProductionWindow {
        building_ref_id,
        kind,
        selected_index: 0,
        queue_selected_index: 0,
        expanded,
        full_selector: debug.full_selector,
        pressed_button: None,
    });
}

pub(crate) fn production_window_top_left(building_tile: UVec2, map_size: Vec2) -> Vec2 {
    let mut top_left = Vec2::new(
        building_tile.x as f32 * TILE_SIZE - WINDOW_SIZE.x * 0.5,
        building_tile.y as f32 * TILE_SIZE - WINDOW_SIZE.y * 0.5,
    );

    if top_left.x + 228.0 + 16.0 > map_size.x {
        top_left.x = map_size.x - (228.0 + 16.0);
    }
    if top_left.y + 96.0 + 16.0 > map_size.y {
        top_left.y = map_size.y - (96.0 + 16.0);
    }
    top_left.x = top_left.x.max(16.0);
    top_left.y = top_left.y.max(16.0);
    top_left
}

fn production_window_size(expanded: bool) -> Vec2 {
    if expanded {
        WINDOW_EXPANDED_SIZE
    } else {
        WINDOW_SIZE
    }
}

pub(crate) fn production_button_spec(kind: ProductionButtonKind) -> (Vec2, Vec2) {
    match kind {
        ProductionButtonKind::Place | ProductionButtonKind::Ok => {
            (Vec2::new(67.0, 60.0), BUTTON_SIZE)
        }
        ProductionButtonKind::Cancel => (Vec2::new(67.0, 47.0), Vec2::new(40.0, 14.0)),
        ProductionButtonKind::Up => (SELECTOR_UP_OFFSET, SELECTOR_ARROW_SIZE),
        ProductionButtonKind::Down => (SELECTOR_DOWN_OFFSET, SELECTOR_ARROW_SIZE),
        ProductionButtonKind::Plus | ProductionButtonKind::Minus => {
            (SMALL_TOGGLE_OFFSET, SMALL_TOGGLE_SIZE)
        }
        ProductionButtonKind::Queue => (QUEUE_BUTTON_OFFSET, QUEUE_BUTTON_SIZE),
        ProductionButtonKind::QueueUp => (QUEUE_SELECTOR_UP_OFFSET, SELECTOR_ARROW_SIZE),
        ProductionButtonKind::QueueDown => (QUEUE_SELECTOR_DOWN_OFFSET, SELECTOR_ARROW_SIZE),
    }
}

pub(crate) fn building_production_window_state(
    production: &BuildingProduction,
    stats: ObjectStats,
) -> BuildingProductionStatus {
    if stats.destroyed() || production.unit_limit_reached {
        BuildingProductionStatus::Paused
    } else if !production.stored_cannons.is_empty() {
        BuildingProductionStatus::Place
    } else {
        production.status
    }
}

pub(crate) fn selected_production_unit_for_window(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    state: BuildingProductionStatus,
    selected_index: usize,
) -> Option<ObjectKind> {
    if state != BuildingProductionStatus::Select {
        return None;
    }

    buildings::selected_production_unit(kind, level, selected_index)
}

#[cfg(test)]
pub(crate) fn apply_production_ok(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
    production: &mut BuildingProduction,
    stats: ObjectStats,
) -> bool {
    let state = building_production_window_state(production, stats);
    let Some(unit) = selected_production_unit_for_window(kind, level, state, selected_index) else {
        return false;
    };

    start_production(production, unit, stats)
}

fn apply_production_ok_from_source(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
    production: &mut BuildingProduction,
    stats: ObjectStats,
    settings: &SourceSettingsState,
) -> bool {
    let state = building_production_window_state(production, stats);
    let Some(unit) = selected_production_unit_for_window(kind, level, state, selected_index) else {
        return false;
    };
    start_production_from_source(production, unit, stats, settings)
}

pub(crate) fn cycle_production_selection(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
    direction: i32,
) -> usize {
    buildings::cycle_production_selection(kind, level, selected_index, direction)
}

fn selected_queue_unit_for_window(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
) -> Option<ObjectKind> {
    buildings::selected_production_unit(kind, level, selected_index)
}

fn build_list_index_for_unit(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    unit: ObjectKind,
) -> Option<usize> {
    buildings::production_selector_unit_index(kind, level, unit)
}

#[cfg(test)]
pub(crate) fn apply_production_queue_add(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    queue_selected_index: usize,
    production: &mut BuildingProduction,
    stats: ObjectStats,
) -> bool {
    let Some(unit) = selected_queue_unit_for_window(kind, level, queue_selected_index) else {
        return false;
    };
    let ObjectKind::Building(building) = kind else {
        return false;
    };

    if production.current.is_none() {
        start_production(production, unit, stats)
    } else {
        add_production_queue(production, building, level, unit, true)
    }
}

fn apply_production_queue_add_from_source(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    queue_selected_index: usize,
    production: &mut BuildingProduction,
    stats: ObjectStats,
    settings: &SourceSettingsState,
) -> bool {
    let Some(unit) = selected_queue_unit_for_window(kind, level, queue_selected_index) else {
        return false;
    };
    let ObjectKind::Building(building) = kind else {
        return false;
    };
    if production.current.is_none() {
        start_production_from_source(production, unit, stats, settings)
    } else {
        add_production_queue(production, building, level, unit, true)
    }
}

pub(crate) fn apply_production_queue_cancel(
    production: &mut BuildingProduction,
    index: usize,
    unit: ObjectKind,
) -> bool {
    cancel_production_queue_item(production, index, unit)
}

fn full_selector_units(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
) -> Vec<(ObjectKind, Vec2)> {
    buildings::production_selector_units(kind, level)
}

fn full_selector_size(kind: ObjectKind, level: impl Into<ProductionLevel> + Copy) -> Vec2 {
    let units = full_selector_units(kind, level);
    if units.is_empty() {
        return Vec2::ZERO;
    }

    let max_right = units
        .iter()
        .map(|(_, offset)| offset.x + FUS_OBJECT_SIZE.x)
        .fold(
            FUS_OBJECT_SIZE.x * 2.0 + FUS_SIDE_SIZE + FUS_MARGIN * 2.0,
            f32::max,
        );
    let max_bottom = units
        .iter()
        .map(|(_, offset)| offset.y + FUS_OBJECT_SIZE.y)
        .fold(FUS_TOP_HEIGHT + FUS_MARGIN + FUS_OBJECT_SIZE.y, f32::max);

    Vec2::new(
        max_right + FUS_MARGIN + FUS_SIDE_SIZE,
        max_bottom + FUS_MARGIN + FUS_SIDE_SIZE,
    )
}

fn full_selector_fill_rect(size: Vec2) -> (Vec2, Vec2) {
    (
        Vec2::new(FUS_SIDE_SIZE, FUS_TOP_HEIGHT),
        Vec2::new(
            size.x - FUS_SIDE_SIZE * 2.0,
            size.y - FUS_TOP_HEIGHT - FUS_SIDE_SIZE,
        ),
    )
}

fn full_selector_top_left(
    window_top_left: Vec2,
    selector_offset: Vec2,
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    map_size: Vec2,
) -> Vec2 {
    let size = full_selector_size(kind, level);
    let center = window_top_left + selector_offset + FUS_SELECTOR_CENTER_OFFSET;
    let mut top_left = center - size * 0.5;

    if top_left.x + size.x + TILE_SIZE > map_size.x {
        top_left.x = map_size.x - (size.x + TILE_SIZE);
    }
    if top_left.y + size.y + TILE_SIZE > map_size.y {
        top_left.y = map_size.y - (size.y + TILE_SIZE);
    }
    top_left.x = top_left.x.max(TILE_SIZE);
    top_left.y = top_left.y.max(TILE_SIZE);
    top_left
}

fn full_selector_unit_at(
    map_pos: Vec2,
    full_top_left: Vec2,
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
) -> Option<ObjectKind> {
    let local_pos = map_pos - full_top_left;
    full_selector_units(kind, level)
        .into_iter()
        .find_map(|(unit, offset)| {
            (local_pos.x >= offset.x
                && local_pos.y >= offset.y
                && local_pos.x <= offset.x + FUS_OBJECT_SIZE.x
                && local_pos.y <= offset.y + FUS_OBJECT_SIZE.y)
                .then_some(unit)
        })
}

fn selector_portrait_contains(local_pos: Vec2, selector_offset: Vec2) -> bool {
    let offset = selector_offset + SELECTOR_PORTRAIT_OFFSET;
    local_pos.x >= offset.x
        && local_pos.y >= offset.y
        && local_pos.x <= offset.x + SELECTOR_PORTRAIT_SIZE.x
        && local_pos.y <= offset.y + SELECTOR_PORTRAIT_SIZE.y
}

pub(crate) fn apply_production_cancel(
    production: &mut BuildingProduction,
    stats: ObjectStats,
) -> bool {
    match building_production_window_state(production, stats) {
        BuildingProductionStatus::Building => stop_production(production, true),
        _ => false,
    }
}

pub(crate) fn production_state_blink_index(elapsed_secs: f32) -> usize {
    ((elapsed_secs / STATE_BLINK_SECONDS).floor() as usize) % 2
}

pub(crate) fn production_progress_yellow_size(progress: f32) -> Vec2 {
    Vec2::new(
        PROGRESS_YELLOW_SIZE.x,
        PROGRESS_YELLOW_SIZE.y * (1.0 - progress.clamp(0.0, 1.0)),
    )
}

#[cfg(test)]
pub(crate) fn production_time_text(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
    state: BuildingProductionStatus,
    production: &BuildingProduction,
    stats: ObjectStats,
) -> String {
    let seconds = match state {
        BuildingProductionStatus::Select => {
            selected_production_unit_for_window(kind, level, state, selected_index)
                .and_then(|unit| {
                    production_duration(
                        unit,
                        stats.health,
                        stats.max_health,
                        production.zone_ownage,
                    )
                })
                .unwrap_or(0.0)
        }
        BuildingProductionStatus::Building
        | BuildingProductionStatus::Place
        | BuildingProductionStatus::Paused => (production.duration - production.elapsed).max(0.0),
    };

    format_production_time(seconds as i32)
}

fn production_time_text_from_source(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
    state: BuildingProductionStatus,
    production: &BuildingProduction,
    stats: ObjectStats,
    settings: &SourceSettingsState,
) -> String {
    let seconds = match state {
        BuildingProductionStatus::Select => {
            selected_production_unit_for_window(kind, level, state, selected_index)
                .and_then(|unit| {
                    production_duration_from_source(
                        unit,
                        stats.health,
                        stats.max_health,
                        production.zone_ownage,
                        settings,
                    )
                })
                .unwrap_or(0.0)
        }
        BuildingProductionStatus::Building
        | BuildingProductionStatus::Place
        | BuildingProductionStatus::Paused => (production.duration - production.elapsed).max(0.0),
    };
    format_production_time(seconds as i32)
}

pub(crate) fn format_production_time(seconds: i32) -> String {
    let seconds = seconds.max(0);
    let minutes = (seconds / 60) % 60;
    let seconds = seconds % 60;
    format!("{minutes}:{seconds:02}")
}

pub(crate) fn health_percent_text(stats: ObjectStats) -> String {
    let percent = if stats.max_health <= 0.0 {
        0
    } else {
        (100.0 * stats.health / stats.max_health) as i32
    }
    .clamp(0, 100);

    format!("{percent}%")
}

pub(crate) fn production_preview_unit_for_window(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
    state: BuildingProductionStatus,
    production: &BuildingProduction,
) -> Option<ObjectKind> {
    match state {
        BuildingProductionStatus::Select => {
            selected_production_unit_for_window(kind, level, state, selected_index)
        }
        BuildingProductionStatus::Place => first_stored_cannon(production),
        BuildingProductionStatus::Building | BuildingProductionStatus::Paused => production.current,
    }
}

fn production_progress_visible(state: BuildingProductionStatus) -> bool {
    state != BuildingProductionStatus::Select
}

pub(crate) fn update_production_window(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<ProductionUiAssets>,
    game_atlases: Res<GameAtlases>,
    map: Res<CurrentMap>,
    settings: Res<SourceSettingsState>,
    window_state: Res<ProductionWindowState>,
    existing: Query<Entity, With<ProductionWindowEntity>>,
    building_query: Query<(
        &GameObjectEntity,
        &MapGridPosition,
        &BuildingLevel,
        &ObjectTeam,
        &ObjectStats,
        &BuildingProduction,
    )>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Some(window) = window_state.open else {
        return;
    };
    let Some((object, grid, level, team, stats, production)) = building_query
        .iter()
        .find(|(object, _, _, _, _, _)| object.ref_id == window.building_ref_id)
    else {
        return;
    };

    let top_left = production_window_top_left(
        UVec2::new(grid.x as u32, grid.y as u32),
        Vec2::new(
            map.0.basics.width as f32 * TILE_SIZE,
            map.0.basics.height as f32 * TILE_SIZE,
        ),
    );
    let state = building_production_window_state(production, *stats);

    spawn_window_piece(
        &mut commands,
        if window.expanded {
            assets.base_expanded.clone()
        } else {
            assets.base.clone()
        },
        window.building_ref_id,
        top_left,
        Vec2::ZERO,
        production_window_size(window.expanded),
        620.0,
        "production_window_base",
    );

    if let Some(label) = assets.labels.get(window.kind as usize) {
        spawn_window_piece(
            &mut commands,
            label.clone(),
            window.building_ref_id,
            top_left,
            NAME_LABEL_OFFSET,
            NAME_LABEL_SIZE,
            621.0,
            "production_window_label",
        )
        .insert(ProductionWindowLabel);
    }

    let state_index = production_state_index(state);
    let blink_index = production_state_blink_index(time.elapsed_secs());
    if let Some(labels) = assets.state_labels.get(state_index) {
        spawn_window_piece(
            &mut commands,
            labels[blink_index].clone(),
            window.building_ref_id,
            top_left,
            STATE_LABEL_OFFSET,
            STATE_LABEL_SIZE,
            621.0,
            "production_window_state",
        )
        .insert(ProductionWindowStateLabel);
    }

    spawn_window_text(
        &mut commands,
        production_time_text_from_source(
            object.kind,
            level.0,
            window.selected_index,
            state,
            production,
            *stats,
            &settings,
        ),
        assets.font.clone(),
        top_left,
        SHOW_TIME_OFFSET,
        bevy::sprite::Anchor::TOP_LEFT,
        622.5,
        "production_window_time",
    );

    spawn_window_text(
        &mut commands,
        health_percent_text(*stats),
        assets.font.clone(),
        top_left,
        HEALTH_PERCENT_OFFSET,
        bevy::sprite::Anchor::TOP_CENTER,
        622.5,
        "production_window_health",
    );

    if let Some(preview_unit) = production_preview_unit_for_window(
        object.kind,
        level.0,
        window.selected_index,
        state,
        production,
    ) {
        spawn_production_preview(
            &mut commands,
            &game_atlases,
            map.0.basics.terrain_type,
            team.0,
            preview_unit,
            top_left,
            SELECTOR_PREVIEW_CENTER,
        );
    }

    if window.expanded {
        if let Some(queue_unit) =
            selected_queue_unit_for_window(object.kind, level.0, window.queue_selected_index)
        {
            spawn_production_preview(
                &mut commands,
                &game_atlases,
                map.0.basics.terrain_type,
                team.0,
                queue_unit,
                top_left,
                QUEUE_SELECTOR_PREVIEW_CENTER,
            );
        }

        for (index, unit) in production.queue.iter().copied().enumerate() {
            let offset = queue_item_offset(index);
            spawn_window_piece(
                &mut commands,
                assets.queue_item_button.normal.clone(),
                window.building_ref_id,
                top_left,
                offset,
                QUEUE_ITEM_SIZE,
                622.0,
                "production_window_queue_item",
            );
            spawn_window_text(
                &mut commands,
                queue_item_text(unit),
                assets.font.clone(),
                top_left,
                offset + Vec2::new(3.0, 2.0),
                bevy::sprite::Anchor::TOP_LEFT,
                622.6,
                "production_window_queue_item_text",
            );
        }
    }

    if let Some(target) = window.full_selector {
        spawn_full_selector(
            &mut commands,
            &assets,
            &game_atlases,
            map.0.basics.terrain_type,
            team.0,
            object.kind,
            level.0,
            top_left,
            full_selector_anchor_offset(target),
            Vec2::new(
                map.0.basics.width as f32 * TILE_SIZE,
                map.0.basics.height as f32 * TILE_SIZE,
            ),
        );
    }

    if production_progress_visible(state) {
        spawn_window_piece(
            &mut commands,
            assets.progress_bar.clone(),
            window.building_ref_id,
            top_left,
            PROGRESS_BAR_OFFSET,
            PROGRESS_BAR_SIZE,
            621.5,
            "production_window_progress_bar",
        );

        let yellow_size = production_progress_yellow_size(production.progress());
        if yellow_size.y > 0.0 {
            spawn_window_piece(
                &mut commands,
                assets.progress_yellow.clone(),
                window.building_ref_id,
                top_left,
                PROGRESS_BAR_OFFSET,
                yellow_size,
                621.6,
                "production_window_progress_yellow",
            );
        }
    }

    for button_kind in [
        ProductionButtonKind::Place,
        ProductionButtonKind::Ok,
        ProductionButtonKind::Cancel,
        ProductionButtonKind::Up,
        ProductionButtonKind::Down,
        ProductionButtonKind::Plus,
        ProductionButtonKind::Minus,
        ProductionButtonKind::Queue,
        ProductionButtonKind::QueueUp,
        ProductionButtonKind::QueueDown,
    ] {
        if !production_button_active(button_kind, state, window.expanded) {
            continue;
        }
        let Some(images) = assets.buttons.get(button_kind as usize) else {
            continue;
        };
        let image = if window.pressed_button == Some(button_kind) {
            images.pressed.clone()
        } else {
            images.normal.clone()
        };
        let (offset, size) = production_button_spec(button_kind);
        spawn_window_piece(
            &mut commands,
            image,
            window.building_ref_id,
            top_left,
            offset,
            size,
            622.0,
            "production_window_button",
        )
        .insert(ProductionWindowButton);
    }
}

fn spawn_full_selector(
    commands: &mut Commands,
    assets: &ProductionUiAssets,
    atlases: &GameAtlases,
    planet: crate::original::types::PlanetType,
    team: TeamType,
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    window_top_left: Vec2,
    selector_offset: Vec2,
    map_size: Vec2,
) {
    let size = full_selector_size(kind, level);
    if size == Vec2::ZERO {
        return;
    }
    let top_left = full_selector_top_left(window_top_left, selector_offset, kind, level, map_size);
    spawn_full_selector_frame(commands, &assets.full_selector_frame, top_left, size);

    for (unit, offset) in full_selector_units(kind, level) {
        spawn_window_piece(
            commands,
            assets.object_button.normal.clone(),
            0,
            top_left,
            offset,
            FUS_OBJECT_SIZE,
            631.0,
            "production_full_selector_button",
        );
        spawn_production_preview(
            commands,
            atlases,
            planet,
            team,
            unit,
            top_left + offset,
            FUS_OBJECT_PREVIEW_CENTER,
        );
        spawn_window_text(
            commands,
            queue_item_text(unit),
            assets.font.clone(),
            top_left,
            offset + FUS_OBJECT_NAME_OFFSET,
            bevy::sprite::Anchor::TOP_CENTER,
            632.0,
            "production_full_selector_name",
        );
    }
}

fn spawn_full_selector_frame(
    commands: &mut Commands,
    assets: &ProductionFullSelectorFrameAssets,
    top_left: Vec2,
    size: Vec2,
) {
    let (fill_offset, fill_size) = full_selector_fill_rect(size);
    spawn_color_panel(
        commands,
        Color::srgb(57.0 / 255.0, 57.0 / 255.0, 57.0 / 255.0),
        top_left + fill_offset,
        fill_size,
        630.0,
        "production_full_selector_fill",
    );

    spawn_window_piece(
        commands,
        assets.top_left.clone(),
        0,
        top_left,
        Vec2::ZERO,
        Vec2::new(63.0, FUS_TOP_HEIGHT),
        630.2,
        "production_full_selector_top_left",
    );
    spawn_window_piece(
        commands,
        assets.top_right.clone(),
        0,
        top_left,
        Vec2::new(size.x - FUS_SIDE_SIZE, 0.0),
        Vec2::new(FUS_SIDE_SIZE, FUS_TOP_HEIGHT),
        630.2,
        "production_full_selector_top_right",
    );
    spawn_window_piece(
        commands,
        assets.bottom_left.clone(),
        0,
        top_left,
        Vec2::new(0.0, size.y - FUS_SIDE_SIZE),
        Vec2::splat(FUS_SIDE_SIZE),
        630.2,
        "production_full_selector_bottom_left",
    );
    spawn_window_piece(
        commands,
        assets.bottom_right.clone(),
        0,
        top_left,
        Vec2::new(size.x - FUS_SIDE_SIZE, size.y - FUS_SIDE_SIZE),
        Vec2::splat(FUS_SIDE_SIZE),
        630.2,
        "production_full_selector_bottom_right",
    );
    spawn_window_piece(
        commands,
        assets.top.clone(),
        0,
        top_left,
        Vec2::new(63.0, 0.0),
        Vec2::new((size.x - 63.0 - FUS_SIDE_SIZE).max(0.0), FUS_TOP_HEIGHT),
        630.1,
        "production_full_selector_top",
    );
    spawn_window_piece(
        commands,
        assets.bottom.clone(),
        0,
        top_left,
        Vec2::new(FUS_SIDE_SIZE, size.y - FUS_SIDE_SIZE),
        Vec2::new((size.x - FUS_SIDE_SIZE * 2.0).max(0.0), FUS_SIDE_SIZE),
        630.1,
        "production_full_selector_bottom",
    );
    spawn_window_piece(
        commands,
        assets.left.clone(),
        0,
        top_left,
        Vec2::new(0.0, FUS_TOP_HEIGHT),
        Vec2::new(
            FUS_SIDE_SIZE,
            (size.y - FUS_TOP_HEIGHT - FUS_SIDE_SIZE).max(0.0),
        ),
        630.1,
        "production_full_selector_left",
    );
    spawn_window_piece(
        commands,
        assets.right.clone(),
        0,
        top_left,
        Vec2::new(size.x - FUS_SIDE_SIZE, FUS_TOP_HEIGHT),
        Vec2::new(
            FUS_SIDE_SIZE,
            (size.y - FUS_TOP_HEIGHT - FUS_SIDE_SIZE).max(0.0),
        ),
        630.1,
        "production_full_selector_right",
    );
}

fn spawn_color_panel(
    commands: &mut Commands,
    color: Color,
    top_left: Vec2,
    size: Vec2,
    z: f32,
    name: &'static str,
) {
    if size.x <= 0.0 || size.y <= 0.0 {
        return;
    }

    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_xyz(top_left.x + size.x * 0.5, -(top_left.y + size.y * 0.5), z),
        ProductionWindowEntity,
        Name::new(name),
    ));
}

fn spawn_production_preview(
    commands: &mut Commands,
    atlases: &GameAtlases,
    planet: crate::original::types::PlanetType,
    team: TeamType,
    kind: ObjectKind,
    top_left: Vec2,
    center_offset: Vec2,
) {
    let Some((object_type, object_id)) = units::object_kind_to_map_parts(kind) else {
        return;
    };
    let object = MapObject {
        x: 0,
        y: 0,
        owner: team,
        object_type,
        object_id,
        building_level: 0,
        extra_links: 0,
        health_percent: 100,
    };
    let layers = atlases.sprite_layers_for_object(&object, planet);
    let Some(bounds) = preview_bounds(&layers) else {
        return;
    };

    for (layer_index, frame) in layers.into_iter().enumerate() {
        let frame_top_left = frame.world_offset + frame.source_offset - bounds.min;
        let center =
            top_left + center_offset - bounds.size * 0.5 + frame_top_left + frame.frame_size * 0.5;
        commands.spawn((
            Sprite {
                image: frame.image,
                texture_atlas: Some(TextureAtlas {
                    layout: frame.layout,
                    index: frame.index,
                }),
                ..default()
            },
            Transform::from_xyz(center.x, -center.y, 622.1 + layer_index as f32 * 0.01),
            ProductionWindowEntity,
            Name::new("production_window_preview"),
        ));
    }
}

fn queue_item_offset(index: usize) -> Vec2 {
    QUEUE_ITEM_OFFSET + Vec2::new(0.0, index as f32 * QUEUE_ITEM_STEP)
}

fn queue_item_at(local_pos: Vec2, queue: &std::collections::VecDeque<ObjectKind>) -> Option<usize> {
    (0..queue.len()).find(|index| {
        let offset = queue_item_offset(*index);
        local_pos.x >= offset.x
            && local_pos.y >= offset.y
            && local_pos.x <= offset.x + QUEUE_ITEM_SIZE.x
            && local_pos.y <= offset.y + QUEUE_ITEM_SIZE.y
    })
}

fn queue_item_text(unit: ObjectKind) -> String {
    crate::units::queue_item_text(unit)
}

fn full_selector_anchor_offset(target: ProductionFullSelectorTarget) -> Vec2 {
    match target {
        ProductionFullSelectorTarget::Main => UNIT_SELECTOR_OFFSET,
        ProductionFullSelectorTarget::Queue => QUEUE_SELECTOR_OFFSET,
    }
}

#[derive(Clone, Copy)]
struct PreviewBounds {
    min: Vec2,
    size: Vec2,
}

fn preview_bounds(layers: &[SpriteFrame]) -> Option<PreviewBounds> {
    let first = layers.first()?;
    let mut min = first.world_offset + first.source_offset;
    let mut max = min + first.frame_size;

    for frame in &layers[1..] {
        let frame_min = frame.world_offset + frame.source_offset;
        let frame_max = frame_min + frame.frame_size;
        min = min.min(frame_min);
        max = max.max(frame_max);
    }

    Some(PreviewBounds {
        min,
        size: max - min,
    })
}

fn spawn_window_text(
    commands: &mut Commands,
    text: String,
    font: Handle<Font>,
    top_left: Vec2,
    offset: Vec2,
    anchor: bevy::sprite::Anchor,
    z: f32,
    name: &'static str,
) {
    let position = top_left + offset;
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font,
            font_size: SMALL_TEXT_SIZE,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Left),
        anchor,
        Transform::from_xyz(position.x, -position.y, z),
        ProductionWindowEntity,
        Name::new(name),
    ));
}

fn spawn_window_piece<'a>(
    commands: &'a mut Commands,
    image: Handle<Image>,
    _building_ref_id: u32,
    top_left: Vec2,
    offset: Vec2,
    size: Vec2,
    z: f32,
    name: &'static str,
) -> EntityCommands<'a> {
    let center = top_left + offset + size * 0.5;
    commands.spawn((
        Sprite {
            image,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(center.x, -center.y, z),
        ProductionWindowEntity,
        Name::new(name),
    ))
}

pub(crate) fn handle_production_window_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    map: Res<CurrentMap>,
    settings: Res<SourceSettingsState>,
    mut window_state: ResMut<ProductionWindowState>,
    mut cannon_placement: ResMut<CannonPlacementState>,
    building_query: Query<(
        &GameObjectEntity,
        &MapGridPosition,
        &BuildingLevel,
        &ObjectStats,
        &mut BuildingProduction,
    )>,
) {
    window_state.input_captured = false;

    let Some(mut window) = window_state.open else {
        return;
    };
    let Some((object, grid, level, stats, mut production)) = building_query
        .into_iter()
        .find(|(object, _, _, _, _)| object.ref_id == window.building_ref_id)
    else {
        window_state.open = None;
        return;
    };

    let Some(world_pos) = cursor_world_position(&windows, &camera_query) else {
        return;
    };
    let map_pos = Vec2::new(world_pos.x, -world_pos.y);
    let top_left = production_window_top_left(
        UVec2::new(grid.x as u32, grid.y as u32),
        Vec2::new(
            map.0.basics.width as f32 * TILE_SIZE,
            map.0.basics.height as f32 * TILE_SIZE,
        ),
    );
    let state = building_production_window_state(&production, *stats);
    let local_pos = map_pos - top_left;
    let window_size = production_window_size(window.expanded);
    let within_window = local_pos.x >= 0.0
        && local_pos.y >= 0.0
        && local_pos.x <= window_size.x
        && local_pos.y <= window_size.y;

    if mouse.just_pressed(MouseButton::Left) {
        if within_window {
            window_state.input_captured = true;
        }
        window.pressed_button = production_button_at(map_pos, top_left, state, window.expanded);
        window_state.open = Some(window);
        return;
    }

    if mouse.just_released(MouseButton::Left) {
        if let Some(target) = window.full_selector {
            let map_size = Vec2::new(
                map.0.basics.width as f32 * TILE_SIZE,
                map.0.basics.height as f32 * TILE_SIZE,
            );
            let selector_top_left = full_selector_top_left(
                top_left,
                full_selector_anchor_offset(target),
                object.kind,
                level.0,
                map_size,
            );
            let selector_size = full_selector_size(object.kind, level.0);
            let inside_selector = map_pos.x >= selector_top_left.x
                && map_pos.y >= selector_top_left.y
                && map_pos.x <= selector_top_left.x + selector_size.x
                && map_pos.y <= selector_top_left.y + selector_size.y;

            if let Some(unit) =
                full_selector_unit_at(map_pos, selector_top_left, object.kind, level.0)
            {
                let index = build_list_index_for_unit(object.kind, level.0, unit)
                    .unwrap_or(window.selected_index);
                window.full_selector = None;
                match target {
                    ProductionFullSelectorTarget::Main => {
                        window.selected_index = index;
                        if apply_production_ok_from_source(
                            object.kind,
                            level.0,
                            window.selected_index,
                            &mut production,
                            *stats,
                            &settings,
                        ) {
                            play_starting_manufacture_sound(
                                &mut commands,
                                &asset_server,
                                object.ref_id,
                            );
                        }
                    }
                    ProductionFullSelectorTarget::Queue => {
                        window.queue_selected_index = index;
                        apply_production_queue_add_from_source(
                            object.kind,
                            level.0,
                            window.queue_selected_index,
                            &mut production,
                            *stats,
                            &settings,
                        );
                    }
                }
                window_state.input_captured = true;
                window_state.open = Some(window);
                return;
            }

            window.full_selector = None;
            if inside_selector {
                window_state.input_captured = true;
                window_state.open = Some(window);
                return;
            }
        }

        if within_window || window.pressed_button.is_some() {
            window_state.input_captured = true;
        }
        let released = production_button_at(map_pos, top_left, state, window.expanded);
        let pressed = window.pressed_button.take();
        if pressed == released {
            match released {
                Some(ProductionButtonKind::Place) => {
                    if let Some(cannon) = first_stored_cannon(&production) {
                        cannon_placement.pending = Some(CannonPlacementRequest {
                            source_ref_id: window.building_ref_id,
                            cannon,
                        });
                        window_state.open = None;
                        return;
                    }
                }
                Some(ProductionButtonKind::Cancel) => {
                    if apply_production_cancel(&mut production, *stats) {
                        play_manufacturing_canceled_sound(
                            &mut commands,
                            &asset_server,
                            window.building_ref_id,
                        );
                        window_state.open = Some(window);
                    } else {
                        window_state.open = None;
                    }
                    return;
                }
                Some(ProductionButtonKind::Ok) => {
                    if apply_production_ok_from_source(
                        object.kind,
                        level.0,
                        window.selected_index,
                        &mut production,
                        *stats,
                        &settings,
                    ) {
                        play_starting_manufacture_sound(
                            &mut commands,
                            &asset_server,
                            window.building_ref_id,
                        );
                        window_state.open = Some(window);
                    } else {
                        window_state.open = None;
                    }
                    return;
                }
                Some(ProductionButtonKind::Up) => {
                    window.selected_index =
                        cycle_production_selection(object.kind, level.0, window.selected_index, 1);
                    window_state.open = Some(window);
                    return;
                }
                Some(ProductionButtonKind::Down) => {
                    window.selected_index =
                        cycle_production_selection(object.kind, level.0, window.selected_index, -1);
                    window_state.open = Some(window);
                    return;
                }
                Some(ProductionButtonKind::Plus) => {
                    window.expanded = true;
                    window_state.open = Some(window);
                    return;
                }
                Some(ProductionButtonKind::Minus) => {
                    window.expanded = false;
                    window_state.open = Some(window);
                    return;
                }
                Some(ProductionButtonKind::Queue) => {
                    apply_production_queue_add_from_source(
                        object.kind,
                        level.0,
                        window.queue_selected_index,
                        &mut production,
                        *stats,
                        &settings,
                    );
                    window_state.open = Some(window);
                    return;
                }
                Some(ProductionButtonKind::QueueUp) => {
                    window.queue_selected_index = cycle_production_selection(
                        object.kind,
                        level.0,
                        window.queue_selected_index,
                        1,
                    );
                    window_state.open = Some(window);
                    return;
                }
                Some(ProductionButtonKind::QueueDown) => {
                    window.queue_selected_index = cycle_production_selection(
                        object.kind,
                        level.0,
                        window.queue_selected_index,
                        -1,
                    );
                    window_state.open = Some(window);
                    return;
                }
                None => {
                    if state == BuildingProductionStatus::Select
                        && selector_portrait_contains(local_pos, UNIT_SELECTOR_OFFSET)
                    {
                        window.full_selector = Some(ProductionFullSelectorTarget::Main);
                        window_state.open = Some(window);
                        return;
                    }
                    if window.expanded
                        && selector_portrait_contains(local_pos, QUEUE_SELECTOR_OFFSET)
                    {
                        window.full_selector = Some(ProductionFullSelectorTarget::Queue);
                        window_state.open = Some(window);
                        return;
                    }
                    if window.expanded {
                        if let Some(index) = queue_item_at(local_pos, &production.queue) {
                            if let Some(unit) = production.queue.get(index).copied() {
                                apply_production_queue_cancel(&mut production, index, unit);
                            }
                            window_state.open = Some(window);
                            return;
                        }
                    }
                }
            }
        } else if window.expanded {
            if let Some(index) = queue_item_at(local_pos, &production.queue) {
                if let Some(unit) = production.queue.get(index).copied() {
                    apply_production_queue_cancel(&mut production, index, unit);
                }
                window_state.open = Some(window);
                return;
            }
        }
        window_state.open = Some(window);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProductionComputerSound {
    StartingManufacture,
    ManufacturingCanceled,
}

impl ProductionComputerSound {
    const fn wire_id(self) -> i32 {
        match self {
            Self::StartingManufacture => 22,
            Self::ManufacturingCanceled => 23,
        }
    }

    const fn from_wire_id(sound: i32) -> Option<Self> {
        match sound {
            22 => Some(Self::StartingManufacture),
            23 => Some(Self::ManufacturingCanceled),
            _ => None,
        }
    }
}

fn production_computer_sound_asset_path(sound: ProductionComputerSound) -> &'static str {
    match sound {
        ProductionComputerSound::StartingManufacture => STARTING_MANUFACTURE_SOUND,
        ProductionComputerSound::ManufacturingCanceled => MANUFACTURING_CANCELED_SOUND,
    }
}

fn relay_production_computer_sound(
    ref_id: u32,
    sound: ProductionComputerSound,
) -> Option<&'static str> {
    let packet = ComputerMessagePacket {
        ref_id: i32::try_from(ref_id).ok()?,
        sound: sound.wire_id(),
    };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    let decoded_packet = ComputerMessagePacket::decode_payload(payload)?;
    let decoded_sound = ProductionComputerSound::from_wire_id(decoded_packet.sound)?;
    Some(production_computer_sound_asset_path(decoded_sound))
}

fn play_starting_manufacture_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ref_id: u32,
) {
    let Some(path) =
        relay_production_computer_sound(ref_id, ProductionComputerSound::StartingManufacture)
    else {
        return;
    };
    play_production_computer_sound(commands, asset_server, path);
}

fn play_manufacturing_canceled_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ref_id: u32,
) {
    let Some(path) =
        relay_production_computer_sound(ref_id, ProductionComputerSound::ManufacturingCanceled)
    else {
        return;
    };
    play_production_computer_sound(commands, asset_server, path);
}

fn play_production_computer_sound(commands: &mut Commands, asset_server: &AssetServer, path: &str) {
    commands.spawn((
        AudioPlayer::new(asset_server.load::<AudioSource>(path.to_string())),
        PlaybackSettings::DESPAWN,
    ));
}

fn production_button_at(
    map_pos: Vec2,
    top_left: Vec2,
    state: BuildingProductionStatus,
    expanded: bool,
) -> Option<ProductionButtonKind> {
    [
        ProductionButtonKind::Place,
        ProductionButtonKind::Ok,
        ProductionButtonKind::Cancel,
        ProductionButtonKind::Up,
        ProductionButtonKind::Down,
        ProductionButtonKind::Plus,
        ProductionButtonKind::Minus,
        ProductionButtonKind::Queue,
        ProductionButtonKind::QueueUp,
        ProductionButtonKind::QueueDown,
    ]
    .into_iter()
    .find(|button| {
        production_button_active(*button, state, expanded)
            && production_button_contains(*button, map_pos - top_left)
    })
}

fn production_button_active(
    button: ProductionButtonKind,
    state: BuildingProductionStatus,
    expanded: bool,
) -> bool {
    if matches!(button, ProductionButtonKind::Plus) {
        return !expanded;
    }
    if matches!(
        button,
        ProductionButtonKind::Minus
            | ProductionButtonKind::Queue
            | ProductionButtonKind::QueueUp
            | ProductionButtonKind::QueueDown
    ) {
        return expanded;
    }

    match state {
        BuildingProductionStatus::Place => {
            matches!(
                button,
                ProductionButtonKind::Place | ProductionButtonKind::Cancel
            )
        }
        BuildingProductionStatus::Select => {
            matches!(
                button,
                ProductionButtonKind::Ok
                    | ProductionButtonKind::Cancel
                    | ProductionButtonKind::Up
                    | ProductionButtonKind::Down
            )
        }
        BuildingProductionStatus::Building | BuildingProductionStatus::Paused => {
            matches!(
                button,
                ProductionButtonKind::Ok | ProductionButtonKind::Cancel
            )
        }
    }
}

fn production_button_contains(button: ProductionButtonKind, local_pos: Vec2) -> bool {
    let (offset, size) = production_button_spec(button);
    local_pos.x >= offset.x
        && local_pos.y >= offset.y
        && local_pos.x <= offset.x + size.x
        && local_pos.y <= offset.y + size.y
}

fn production_state_index(state: BuildingProductionStatus) -> usize {
    match state {
        BuildingProductionStatus::Place => 0,
        BuildingProductionStatus::Select => 1,
        BuildingProductionStatus::Building => 2,
        BuildingProductionStatus::Paused => 3,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;
    use crate::original::objects::{BuildingType, CannonType, RobotType};

    #[test]
    fn production_window_position_matches_original_set_cords() {
        assert_eq!(
            production_window_top_left(UVec2::new(10, 20), Vec2::new(1024.0, 1376.0)),
            Vec2::new(104.0, 280.0)
        );
        assert_eq!(
            production_window_top_left(UVec2::ZERO, Vec2::new(1024.0, 1376.0)),
            Vec2::new(16.0, 16.0)
        );
    }

    #[test]
    fn production_button_offsets_match_original_gui_buttons() {
        assert_eq!(
            production_button_spec(ProductionButtonKind::Place),
            (Vec2::new(67.0, 60.0), Vec2::new(40.0, 15.0))
        );
        assert_eq!(
            production_button_spec(ProductionButtonKind::Ok),
            (Vec2::new(67.0, 60.0), Vec2::new(40.0, 15.0))
        );
        assert_eq!(
            production_button_spec(ProductionButtonKind::Cancel),
            (Vec2::new(67.0, 47.0), Vec2::new(40.0, 14.0))
        );
        assert_eq!(
            production_button_spec(ProductionButtonKind::Plus),
            (Vec2::new(96.0, 4.0), Vec2::new(12.0, 12.0))
        );
        assert_eq!(
            production_button_spec(ProductionButtonKind::Minus),
            (Vec2::new(96.0, 4.0), Vec2::new(12.0, 12.0))
        );
        assert_eq!(
            production_button_spec(ProductionButtonKind::Queue),
            (Vec2::new(111.0, 78.0), Vec2::new(60.0, 13.0))
        );
        assert_eq!(
            production_button_spec(ProductionButtonKind::QueueUp),
            (Vec2::new(158.0, 21.0), Vec2::new(16.0, 8.0))
        );
        assert_eq!(
            production_button_spec(ProductionButtonKind::QueueDown),
            (Vec2::new(158.0, 64.0), Vec2::new(16.0, 8.0))
        );
        assert_eq!(PROGRESS_BAR_OFFSET, Vec2::new(53.0, 21.0));
        assert_eq!(PROGRESS_BAR_SIZE, Vec2::new(9.0, 50.0));
    }

    #[test]
    fn production_button_active_table_matches_original_states() {
        assert!(production_button_active(
            ProductionButtonKind::Place,
            BuildingProductionStatus::Place,
            false
        ));
        assert!(production_button_active(
            ProductionButtonKind::Cancel,
            BuildingProductionStatus::Place,
            false
        ));
        assert!(!production_button_active(
            ProductionButtonKind::Ok,
            BuildingProductionStatus::Place,
            false
        ));

        for state in [
            BuildingProductionStatus::Select,
            BuildingProductionStatus::Building,
            BuildingProductionStatus::Paused,
        ] {
            assert!(production_button_active(
                ProductionButtonKind::Ok,
                state,
                false
            ));
            assert!(production_button_active(
                ProductionButtonKind::Cancel,
                state,
                false
            ));
            assert!(!production_button_active(
                ProductionButtonKind::Place,
                state,
                false
            ));
        }

        assert!(production_button_active(
            ProductionButtonKind::Plus,
            BuildingProductionStatus::Building,
            false
        ));
        assert!(!production_button_active(
            ProductionButtonKind::Plus,
            BuildingProductionStatus::Building,
            true
        ));
        for button in [
            ProductionButtonKind::Minus,
            ProductionButtonKind::Queue,
            ProductionButtonKind::QueueUp,
            ProductionButtonKind::QueueDown,
        ] {
            assert!(production_button_active(
                button,
                BuildingProductionStatus::Place,
                true
            ));
            assert!(!production_button_active(
                button,
                BuildingProductionStatus::Place,
                false
            ));
        }
    }

    #[test]
    fn production_state_label_blinks_like_original_timer() {
        assert_eq!(production_state_blink_index(0.0), 0);
        assert_eq!(production_state_blink_index(0.29), 0);
        assert_eq!(production_state_blink_index(0.30), 1);
        assert_eq!(production_state_blink_index(0.59), 1);
        assert_eq!(production_state_blink_index(0.60), 0);
    }

    #[test]
    fn production_progress_bar_matches_original_non_select_states() {
        assert!(!production_progress_visible(
            BuildingProductionStatus::Select
        ));
        assert!(production_progress_visible(
            BuildingProductionStatus::Building
        ));
        assert!(production_progress_visible(BuildingProductionStatus::Place));
        assert!(production_progress_visible(
            BuildingProductionStatus::Paused
        ));

        assert_eq!(production_progress_yellow_size(0.0), Vec2::new(9.0, 39.0));
        assert_eq!(production_progress_yellow_size(0.5), Vec2::new(9.0, 19.5));
        assert_eq!(production_progress_yellow_size(1.0), Vec2::new(9.0, 0.0));
    }

    #[test]
    fn production_time_text_matches_original_formatting() {
        assert_eq!(format_production_time(-1), "0:00");
        assert_eq!(format_production_time(0), "0:00");
        assert_eq!(format_production_time(7), "0:07");
        assert_eq!(format_production_time(65), "1:05");
        assert_eq!(format_production_time(3661), "1:01");
    }

    #[test]
    fn production_time_uses_selected_unit_or_remaining_build_time() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = BuildingProduction {
            status: BuildingProductionStatus::Select,
            current: None,
            queue: VecDeque::new(),
            elapsed: 2.0,
            duration: 10.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        };

        let selected_time = production_time_text(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            0,
            BuildingProductionStatus::Select,
            &production,
            stats,
        );
        assert_ne!(selected_time, "0:08");

        production.status = BuildingProductionStatus::Building;
        assert_eq!(
            production_time_text(
                ObjectKind::Building(BuildingType::RobotFactory),
                0,
                0,
                BuildingProductionStatus::Building,
                &production,
                stats,
            ),
            "0:08"
        );
    }

    #[test]
    fn health_percent_text_matches_original_integer_percent() {
        let mut stats =
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        assert_eq!(health_percent_text(stats), "100%");
        stats.health = stats.max_health * 0.755;
        assert_eq!(health_percent_text(stats), "75%");
        stats.health = -1.0;
        assert_eq!(health_percent_text(stats), "0%");
    }

    #[test]
    fn production_preview_unit_matches_original_selector_modes() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = BuildingProduction {
            status: BuildingProductionStatus::Select,
            current: None,
            queue: VecDeque::new(),
            elapsed: 0.0,
            duration: 0.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        };

        assert_eq!(
            production_preview_unit_for_window(
                ObjectKind::Building(BuildingType::RobotFactory),
                0,
                0,
                BuildingProductionStatus::Select,
                &production,
            ),
            Some(ObjectKind::Robot(RobotType::Grunt))
        );

        start_production(
            &mut production,
            ObjectKind::Cannon(CannonType::Gatling),
            stats,
        );
        assert_eq!(
            production_preview_unit_for_window(
                ObjectKind::Building(BuildingType::RobotFactory),
                0,
                0,
                BuildingProductionStatus::Building,
                &production,
            ),
            Some(ObjectKind::Cannon(CannonType::Gatling))
        );

        production
            .stored_cannons
            .push(ObjectKind::Cannon(CannonType::Gun));
        assert_eq!(
            production_preview_unit_for_window(
                ObjectKind::Building(BuildingType::RobotFactory),
                0,
                0,
                BuildingProductionStatus::Place,
                &production,
            ),
            Some(ObjectKind::Cannon(CannonType::Gun))
        );
    }

    #[test]
    fn production_window_state_overrides_match_original() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = BuildingProduction {
            status: BuildingProductionStatus::Building,
            current: Some(ObjectKind::Robot(RobotType::Grunt)),
            queue: VecDeque::new(),
            elapsed: 0.0,
            duration: 10.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        };

        assert_eq!(
            building_production_window_state(&production, stats),
            BuildingProductionStatus::Building
        );

        production
            .stored_cannons
            .push(ObjectKind::Cannon(CannonType::Gatling));
        assert_eq!(
            building_production_window_state(&production, stats),
            BuildingProductionStatus::Place
        );

        production.unit_limit_reached = true;
        assert_eq!(
            building_production_window_state(&production, stats),
            BuildingProductionStatus::Paused
        );
        production.unit_limit_reached = false;

        let destroyed_stats =
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 0);
        assert_eq!(
            building_production_window_state(&production, destroyed_stats),
            BuildingProductionStatus::Paused
        );
    }

    #[test]
    fn production_window_selects_default_build_list_head_for_ok() {
        assert_eq!(
            selected_production_unit_for_window(
                ObjectKind::Building(BuildingType::RobotFactory),
                0,
                BuildingProductionStatus::Select,
                0,
            ),
            Some(ObjectKind::Robot(RobotType::Grunt))
        );
        assert_eq!(
            selected_production_unit_for_window(
                ObjectKind::Building(BuildingType::RobotFactory),
                0,
                BuildingProductionStatus::Select,
                1,
            ),
            Some(ObjectKind::Cannon(CannonType::Gatling))
        );
        assert_eq!(
            selected_production_unit_for_window(
                ObjectKind::Building(BuildingType::VehicleFactory),
                0,
                BuildingProductionStatus::Select,
                0,
            ),
            Some(ObjectKind::Vehicle(
                crate::original::objects::VehicleType::Jeep
            ))
        );
        assert_eq!(
            selected_production_unit_for_window(
                ObjectKind::Building(BuildingType::RobotFactory),
                0,
                BuildingProductionStatus::Building,
                0,
            ),
            None
        );
    }

    #[test]
    fn production_selection_buttons_wrap_like_original_selector() {
        let robot_factory = ObjectKind::Building(BuildingType::RobotFactory);

        assert_eq!(cycle_production_selection(robot_factory, 0, 0, 1), 1);
        assert_eq!(cycle_production_selection(robot_factory, 0, 1, 1), 0);
        assert_eq!(cycle_production_selection(robot_factory, 0, 0, -1), 1);
        assert_eq!(cycle_production_selection(robot_factory, 0, 1, -1), 0);
    }

    #[test]
    fn robot_factory_level1_selector_order_matches_original_build_list() {
        let robot_factory = ObjectKind::Building(BuildingType::RobotFactory);

        let next = cycle_production_selection(robot_factory, 1, 0, 1);
        assert_eq!(next, 1);
        assert_eq!(
            selected_production_unit_for_window(
                robot_factory,
                1,
                BuildingProductionStatus::Select,
                next,
            ),
            Some(ObjectKind::Robot(RobotType::Psycho))
        );
    }

    #[test]
    fn production_ok_starts_selected_unit_only_from_select_state() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = BuildingProduction {
            status: BuildingProductionStatus::Select,
            current: None,
            queue: VecDeque::new(),
            elapsed: 0.0,
            duration: 0.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        };

        assert!(apply_production_ok(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            1,
            &mut production,
            stats,
        ));
        assert_eq!(
            production.current,
            Some(ObjectKind::Cannon(CannonType::Gatling))
        );
        assert_eq!(production.status, BuildingProductionStatus::Building);

        assert!(!apply_production_ok(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            0,
            &mut production,
            stats,
        ));
    }

    #[test]
    fn production_start_cancel_sounds_match_original_assets() {
        assert_eq!(
            STARTING_MANUFACTURE_SOUND,
            "sounds/comp_starting_manufacture.wav"
        );
        assert_eq!(
            MANUFACTURING_CANCELED_SOUND,
            "sounds/comp_manufacturing_canceled.wav"
        );
        assert_eq!(
            production_computer_sound_asset_path(ProductionComputerSound::StartingManufacture),
            STARTING_MANUFACTURE_SOUND
        );
        assert_eq!(
            production_computer_sound_asset_path(ProductionComputerSound::ManufacturingCanceled),
            MANUFACTURING_CANCELED_SOUND
        );
    }

    #[test]
    fn production_start_cancel_sounds_round_trip_through_comp_msg() {
        assert_eq!(
            relay_production_computer_sound(7, ProductionComputerSound::StartingManufacture),
            Some(STARTING_MANUFACTURE_SOUND)
        );
        assert_eq!(
            relay_production_computer_sound(7, ProductionComputerSound::ManufacturingCanceled),
            Some(MANUFACTURING_CANCELED_SOUND)
        );
        assert_eq!(
            relay_production_computer_sound(u32::MAX, ProductionComputerSound::StartingManufacture,),
            None
        );
        assert_eq!(ProductionComputerSound::from_wire_id(21), None);
    }

    #[test]
    fn production_queue_add_matches_original_idle_or_active_rules() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = BuildingProduction {
            status: BuildingProductionStatus::Select,
            current: None,
            queue: VecDeque::new(),
            elapsed: 0.0,
            duration: 0.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        };

        assert!(apply_production_queue_add(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            0,
            &mut production,
            stats,
        ));
        assert_eq!(
            production.current,
            Some(ObjectKind::Robot(RobotType::Grunt))
        );
        assert_eq!(
            production.queue,
            VecDeque::from([ObjectKind::Robot(RobotType::Grunt)])
        );

        assert!(apply_production_queue_add(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            1,
            &mut production,
            stats,
        ));
        assert_eq!(
            production.queue,
            VecDeque::from([
                ObjectKind::Cannon(CannonType::Gatling),
                ObjectKind::Robot(RobotType::Grunt),
            ])
        );
    }

    #[test]
    fn queued_item_added_from_ui_builds_next_like_original_server_event() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = BuildingProduction {
            status: BuildingProductionStatus::Select,
            current: None,
            queue: VecDeque::new(),
            elapsed: 0.0,
            duration: 0.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        };

        assert!(apply_production_queue_add(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            0,
            &mut production,
            stats,
        ));
        assert!(apply_production_queue_add(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            1,
            &mut production,
            stats,
        ));

        let duration = production.duration;
        let completed = crate::production::advance_production(&mut production, duration, stats);

        assert_eq!(completed, vec![ObjectKind::Robot(RobotType::Grunt)]);
        assert_eq!(
            production.current,
            Some(ObjectKind::Cannon(CannonType::Gatling))
        );
        assert_eq!(
            production.queue,
            VecDeque::from([ObjectKind::Robot(RobotType::Grunt)])
        );
    }

    #[test]
    fn production_queue_cancel_requires_matching_index_and_unit() {
        let mut production = BuildingProduction {
            status: BuildingProductionStatus::Building,
            current: Some(ObjectKind::Robot(RobotType::Grunt)),
            queue: VecDeque::from([
                ObjectKind::Robot(RobotType::Grunt),
                ObjectKind::Cannon(CannonType::Gatling),
            ]),
            elapsed: 0.0,
            duration: 10.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        };

        assert!(!apply_production_queue_cancel(
            &mut production,
            1,
            ObjectKind::Robot(RobotType::Grunt),
        ));
        assert_eq!(production.queue.len(), 2);

        assert!(apply_production_queue_cancel(
            &mut production,
            1,
            ObjectKind::Cannon(CannonType::Gatling),
        ));
        assert_eq!(
            production.queue,
            VecDeque::from([ObjectKind::Robot(RobotType::Grunt)])
        );
    }

    #[test]
    fn full_selector_layout_matches_original_rows_and_sizes() {
        let robot_factory = ObjectKind::Building(BuildingType::RobotFactory);
        let units = full_selector_units(robot_factory, 0);
        assert_eq!(
            units,
            vec![
                (ObjectKind::Robot(RobotType::Grunt), Vec2::new(6.0, 22.0)),
                (
                    ObjectKind::Cannon(CannonType::Gatling),
                    Vec2::new(6.0, 75.0)
                ),
            ]
        );
        assert_eq!(
            full_selector_size(robot_factory, 0),
            Vec2::new(104.0, 132.0)
        );
        assert_eq!(
            full_selector_fill_rect(Vec2::new(104.0, 132.0)),
            (Vec2::new(4.0, 20.0), Vec2::new(96.0, 108.0))
        );
        assert_eq!(
            full_selector_unit_at(
                Vec2::new(22.0, 30.0),
                Vec2::new(16.0, 8.0),
                robot_factory,
                0
            ),
            Some(ObjectKind::Robot(RobotType::Grunt))
        );
    }

    #[test]
    fn full_selector_selection_maps_back_to_build_list_index() {
        let robot_factory = ObjectKind::Building(BuildingType::RobotFactory);
        assert_eq!(
            build_list_index_for_unit(robot_factory, 0, ObjectKind::Robot(RobotType::Grunt)),
            Some(0)
        );
        assert_eq!(
            build_list_index_for_unit(robot_factory, 0, ObjectKind::Cannon(CannonType::Gatling)),
            Some(1)
        );
        assert_eq!(
            build_list_index_for_unit(robot_factory, 0, ObjectKind::Cannon(CannonType::Gun)),
            None
        );
    }

    #[test]
    fn production_cancel_stops_active_building_and_clears_queue() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = BuildingProduction {
            status: BuildingProductionStatus::Building,
            current: Some(ObjectKind::Robot(RobotType::Grunt)),
            queue: VecDeque::from([ObjectKind::Robot(RobotType::Grunt)]),
            elapsed: 2.0,
            duration: 10.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        };

        assert!(apply_production_cancel(&mut production, stats));
        assert_eq!(production.status, BuildingProductionStatus::Select);
        assert_eq!(production.current, None);
        assert!(production.queue.is_empty());

        assert!(!apply_production_cancel(&mut production, stats));
    }
}
