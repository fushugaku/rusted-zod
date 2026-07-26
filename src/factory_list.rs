use bevy::{
    camera::visibility::RenderLayers, input::mouse::MouseWheel, prelude::*, sprite::Anchor,
};

use crate::{
    components::{
        BuildingLevel, BuildingProduction, BuildingProductionStatus, GameObjectEntity, HudAnchor,
        HudAssets, HudCommand, HudCommandQueue, ObjectStats, ObjectTeam, ProductionWindowState,
    },
    constants::{HUD_HEIGHT, HUD_LAYER},
    local_player::LocalPlayerState,
    original::objects::{BuildingType, CannonType, ObjectKind, RobotType, VehicleType},
};

const WIDTH: f32 = 142.0;
const TOP_HEIGHT: f32 = 20.0;
const ENTRY_HEIGHT: f32 = 60.0;
const ENTRY_CLICK_WIDTH: f32 = 120.0;
const BAR_OFFSET: Vec2 = Vec2::new(12.0, 7.0);
const BAR_SIZE: Vec2 = Vec2::new(102.0, 12.0);
const BAR_STEP: f32 = 17.0;

#[derive(Default, Resource)]
pub(crate) struct FactoryListState {
    pub(crate) visible: bool,
    start_entry: usize,
}

impl FactoryListState {
    pub(crate) fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

#[derive(Resource)]
pub(crate) struct FactoryListAssets {
    top: Handle<Image>,
    entry: Handle<Image>,
    right: Handle<Image>,
    bar_green: Handle<Image>,
    bar_grey: Handle<Image>,
    bar_white: Handle<Image>,
    up: Handle<Image>,
    down: Handle<Image>,
}

#[derive(Component)]
pub(crate) struct FactoryListEntity;

#[derive(Clone)]
struct FactoryEntry {
    ref_id: u32,
    building: BuildingType,
    level: i8,
    health: f32,
    production: BuildingProduction,
}

pub(crate) fn setup_factory_list_assets(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(FactoryListAssets {
        top: assets.load("other/factory_gui/main_top.png"),
        entry: assets.load("other/factory_gui/main_entry.png"),
        right: assets.load("other/factory_gui/main_right.png"),
        bar_green: assets.load("other/factory_gui/entry_bar_green.png"),
        bar_grey: assets.load("other/factory_gui/entry_bar_grey.png"),
        bar_white: assets.load("other/factory_gui/entry_bar_white_i.png"),
        up: assets.load("other/factory_gui/fup_button.png"),
        down: assets.load("other/factory_gui/fdown_button.png"),
    });
}

pub(crate) fn handle_factory_list_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    local_player: Res<LocalPlayerState>,
    mut state: ResMut<FactoryListState>,
    mut production_window: ResMut<ProductionWindowState>,
    mut commands: ResMut<HudCommandQueue>,
    buildings: Query<(
        &GameObjectEntity,
        &ObjectTeam,
        &ObjectStats,
        &BuildingProduction,
    )>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        state.toggle();
    }
    if !state.visible {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let mut entries: Vec<(u8, u32)> = buildings
        .iter()
        .filter_map(|(object, team, _, _)| {
            let building = factory_kind(object.kind)?;
            (team.0 == local_player.team()).then_some((factory_sort_key(building), object.ref_id))
        })
        .collect();
    entries.sort_unstable();
    let entries: Vec<u32> = entries.into_iter().map(|(_, ref_id)| ref_id).collect();
    let visible_count = visible_entry_count(window.height(), entries.len());
    let max_start = entries.len().saturating_sub(visible_count);
    for event in wheel.read() {
        if event.y > 0.0 {
            state.start_entry = state.start_entry.saturating_sub(1);
        } else if event.y < 0.0 {
            state.start_entry = (state.start_entry + 1).min(max_start);
        }
    }
    state.start_entry = state.start_entry.min(max_start);

    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let panel_height = TOP_HEIGHT + visible_count as f32 * ENTRY_HEIGHT;
    let top = (window.height() - HUD_HEIGHT - panel_height).max(0.0);
    let inside =
        cursor.x >= 0.0 && cursor.x < WIDTH && cursor.y >= top && cursor.y < top + panel_height;
    if inside {
        production_window.input_captured = true;
    }
    if !inside || !mouse.just_released(MouseButton::Left) {
        return;
    }

    if cursor.x >= 123.0 && visible_count < entries.len() {
        if cursor.y < top + TOP_HEIGHT + 11.0 {
            state.start_entry = state.start_entry.saturating_sub(1);
        } else if cursor.y >= top + panel_height - 11.0 {
            state.start_entry = (state.start_entry + 1).min(max_start);
        }
        return;
    }
    if cursor.x >= ENTRY_CLICK_WIDTH || cursor.y < top + TOP_HEIGHT {
        return;
    }
    let row = ((cursor.y - top - TOP_HEIGHT) / ENTRY_HEIGHT).floor() as usize;
    if let Some(ref_id) = entries.get(state.start_entry + row).copied() {
        commands.pending.push(HudCommand::FocusObject {
            ref_id,
            select_obj: false,
            open_gui: true,
        });
    }
}

pub(crate) fn update_factory_list(
    mut commands: Commands,
    windows: Query<&Window>,
    state: Res<FactoryListState>,
    local_player: Res<LocalPlayerState>,
    assets: Res<FactoryListAssets>,
    hud_assets: Res<HudAssets>,
    existing: Query<Entity, With<FactoryListEntity>>,
    buildings: Query<(
        &GameObjectEntity,
        &ObjectTeam,
        &ObjectStats,
        &BuildingLevel,
        &BuildingProduction,
    )>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if !state.visible {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let mut entries: Vec<_> = buildings
        .iter()
        .filter_map(|(object, team, stats, level, production)| {
            let building = factory_kind(object.kind)?;
            (team.0 == local_player.team()).then_some(FactoryEntry {
                ref_id: object.ref_id,
                building,
                level: level.0.original(),
                health: if stats.max_health <= 0.0 {
                    0.0
                } else {
                    (stats.health / stats.max_health).clamp(0.0, 1.0)
                },
                production: production.clone(),
            })
        })
        .collect();
    entries.sort_by_key(|entry| (factory_sort_key(entry.building), entry.ref_id));

    let visible_count = visible_entry_count(window.height(), entries.len());
    let start = state
        .start_entry
        .min(entries.len().saturating_sub(visible_count));
    let panel_height = TOP_HEIGHT + visible_count as f32 * ENTRY_HEIGHT;
    let top = (window.height() - HUD_HEIGHT - panel_height).max(0.0);
    spawn_image(
        &mut commands,
        assets.top.clone(),
        Vec2::new(0.0, top),
        Vec2::new(WIDTH, TOP_HEIGHT),
        780.0,
        "factory_list_top",
    );

    for (row, entry) in entries.iter().skip(start).take(visible_count).enumerate() {
        let entry_top = top + TOP_HEIGHT + row as f32 * ENTRY_HEIGHT;
        spawn_image(
            &mut commands,
            assets.entry.clone(),
            Vec2::new(0.0, entry_top),
            Vec2::new(138.0, ENTRY_HEIGHT),
            780.0,
            "factory_list_entry",
        );
        let rows = factory_entry_rows(entry);
        for (index, row) in rows.into_iter().enumerate() {
            let bar_top = entry_top + BAR_OFFSET.y + index as f32 * BAR_STEP;
            let bar_left = BAR_OFFSET.x;
            spawn_image(
                &mut commands,
                assets.bar_grey.clone(),
                Vec2::new(bar_left, bar_top),
                BAR_SIZE,
                781.0,
                "factory_list_bar_grey",
            );
            if row.colored {
                let width = (BAR_SIZE.x * row.percent.clamp(0.0, 1.0)).floor();
                if width > 0.0 {
                    spawn_image(
                        &mut commands,
                        assets.bar_green.clone(),
                        Vec2::new(bar_left, bar_top),
                        Vec2::new(width, BAR_SIZE.y),
                        782.0,
                        "factory_list_bar_green",
                    );
                }
                let white_x = bar_left + (width - 1.0).max(0.0);
                spawn_image(
                    &mut commands,
                    assets.bar_white.clone(),
                    Vec2::new(white_x, bar_top),
                    Vec2::new(1.0, BAR_SIZE.y),
                    783.0,
                    "factory_list_bar_white",
                );
            }
            spawn_text(
                &mut commands,
                hud_assets.font.clone(),
                row.left,
                Vec2::new(bar_left + 2.0, bar_top + 3.0),
                Anchor::TOP_LEFT,
            );
            if !row.right.is_empty() {
                spawn_text(
                    &mut commands,
                    hud_assets.font.clone(),
                    row.right,
                    Vec2::new(bar_left + 99.0, bar_top + 3.0),
                    Anchor::TOP_RIGHT,
                );
            }
        }
    }

    if visible_count > 0 {
        spawn_image(
            &mut commands,
            assets.right.clone(),
            Vec2::new(138.0, top + TOP_HEIGHT),
            Vec2::new(4.0, visible_count as f32 * ENTRY_HEIGHT),
            784.0,
            "factory_list_right",
        );
    }
    if visible_count < entries.len() {
        spawn_image(
            &mut commands,
            assets.up.clone(),
            Vec2::new(123.0, top + TOP_HEIGHT + 2.0),
            Vec2::new(11.0, 9.0),
            785.0,
            "factory_list_up",
        );
        spawn_image(
            &mut commands,
            assets.down.clone(),
            Vec2::new(123.0, top + panel_height - 11.0),
            Vec2::new(11.0, 9.0),
            785.0,
            "factory_list_down",
        );
    }
}

struct FactoryRow {
    left: String,
    right: String,
    colored: bool,
    percent: f32,
}

fn factory_entry_rows(entry: &FactoryEntry) -> [FactoryRow; 3] {
    let health = entry.health.clamp(0.0, 1.0);
    let health_row = FactoryRow {
        left: factory_name(entry.building).to_string(),
        right: format!("{}%", (health * 100.0) as i32),
        colored: true,
        percent: health,
    };
    let destroyed = health <= 0.0;
    let production_row = if destroyed {
        FactoryRow {
            left: "Destroyed".to_string(),
            right: String::new(),
            colored: true,
            percent: 0.0,
        }
    } else {
        match entry.production.status {
            BuildingProductionStatus::Place => factory_status_row("Placing Cannon"),
            BuildingProductionStatus::Select => factory_status_row("No Production"),
            BuildingProductionStatus::Paused => factory_status_row("Paused"),
            BuildingProductionStatus::Building => {
                let progress = entry.production.progress();
                FactoryRow {
                    left: entry
                        .production
                        .current
                        .map(production_name)
                        .unwrap_or("???")
                        .to_string(),
                    right: format_time(entry.production.time_left()),
                    colored: true,
                    percent: progress,
                }
            }
        }
    };
    let tech_row = FactoryRow {
        left: format!("Tech Level {}", i16::from(entry.level) + 1),
        right: String::new(),
        colored: destroyed,
        percent: 0.0,
    };
    [health_row, production_row, tech_row]
}

fn factory_status_row(label: &str) -> FactoryRow {
    FactoryRow {
        left: label.to_string(),
        right: String::new(),
        colored: false,
        percent: 0.0,
    }
}

fn factory_kind(kind: ObjectKind) -> Option<BuildingType> {
    match kind {
        ObjectKind::Building(
            building @ (BuildingType::FortFront
            | BuildingType::FortBack
            | BuildingType::RobotFactory
            | BuildingType::VehicleFactory),
        ) => Some(building),
        _ => None,
    }
}

fn factory_sort_key(building: BuildingType) -> u8 {
    match building {
        BuildingType::FortFront | BuildingType::FortBack => 0,
        BuildingType::RobotFactory => 1,
        BuildingType::VehicleFactory => 2,
        _ => 3,
    }
}

fn factory_name(building: BuildingType) -> &'static str {
    match building {
        BuildingType::FortFront | BuildingType::FortBack => "Fort Factory",
        BuildingType::RobotFactory => "Robot Factory",
        BuildingType::VehicleFactory => "Vehicle Factory",
        _ => "Factory",
    }
}

fn production_name(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Robot(RobotType::Grunt) => "Grunt",
        ObjectKind::Robot(RobotType::Psycho) => "Psycho",
        ObjectKind::Robot(RobotType::Sniper) => "Sniper",
        ObjectKind::Robot(RobotType::Tough) => "Tough",
        ObjectKind::Robot(RobotType::Pyro) => "Pyro",
        ObjectKind::Robot(RobotType::Laser) => "Laser",
        ObjectKind::Vehicle(VehicleType::Jeep) => "Jeep",
        ObjectKind::Vehicle(VehicleType::Light) => "Light",
        ObjectKind::Vehicle(VehicleType::Medium) => "Medium",
        ObjectKind::Vehicle(VehicleType::Heavy) => "Heavy",
        ObjectKind::Vehicle(VehicleType::Apc) => "APC",
        ObjectKind::Vehicle(VehicleType::MissileLauncher) => "M Missile",
        ObjectKind::Vehicle(VehicleType::Crane) => "Crane",
        ObjectKind::Cannon(CannonType::Gatling) => "Gatling",
        ObjectKind::Cannon(CannonType::Gun) => "Gun",
        ObjectKind::Cannon(CannonType::Howitzer) => "Howitzer",
        ObjectKind::Cannon(CannonType::MissileCannon) => "Missile",
        _ => "???",
    }
}

fn format_time(seconds: f32) -> String {
    let seconds = seconds.max(0.0) as i32;
    format!("{}:{:02}", (seconds / 60) % 60, seconds % 60)
}

fn visible_entry_count(window_height: f32, entry_count: usize) -> usize {
    let view_height = (window_height - HUD_HEIGHT).max(0.0);
    (((view_height - TOP_HEIGHT).max(0.0) / ENTRY_HEIGHT).floor() as usize).min(entry_count)
}

fn spawn_image(
    commands: &mut Commands,
    image: Handle<Image>,
    top_left: Vec2,
    size: Vec2,
    z: f32,
    name: &'static str,
) {
    commands.spawn((
        Sprite {
            image,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, z),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::ScreenTopLeft { top_left, size },
        FactoryListEntity,
        Name::new(name),
    ));
}

fn spawn_text(
    commands: &mut Commands,
    font: Handle<Font>,
    text: String,
    top_left: Vec2,
    anchor: Anchor,
) {
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font,
            font_size: 8.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(if anchor == Anchor::TOP_RIGHT {
            Justify::Right
        } else {
            Justify::Left
        }),
        anchor,
        Transform::from_xyz(0.0, 0.0, 786.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::ScreenTopLeft {
            top_left,
            size: Vec2::ZERO,
        },
        FactoryListEntity,
        Name::new("factory_list_text"),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_list_geometry_matches_source_assets() {
        assert_eq!(visible_entry_count(700.0, 20), 10);
        assert_eq!(factory_sort_key(BuildingType::FortBack), 0);
        assert_eq!(factory_sort_key(BuildingType::RobotFactory), 1);
        assert_eq!(factory_sort_key(BuildingType::VehicleFactory), 2);
        assert_eq!(
            production_name(ObjectKind::Vehicle(VehicleType::MissileLauncher)),
            "M Missile"
        );
    }
}
