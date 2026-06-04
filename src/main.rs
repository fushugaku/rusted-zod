use bevy::{
    asset::{AssetMetaCheck, AssetPlugin},
    audio::{AudioPlayer, AudioSource, PlaybackSettings},
    prelude::*,
    render::{
        RenderPlugin,
        settings::{RenderCreation, WgpuSettings},
    },
    window::WindowResolution,
};
use std::collections::{HashMap, HashSet};

#[cfg(target_arch = "wasm32")]
use bevy::render::settings::{Backends, WgpuSettingsPriority};

mod camera;
mod components;
mod constants;
mod cursor;
mod enter;
mod grenades;
mod hud;
mod original;
mod pathing;
mod placement;
mod production;
mod production_ui;
mod render;
mod repair;
mod robot_groups;
mod selection;
mod units;
mod world_objects;
mod zones;

use camera::*;
use components::*;
use constants::*;
use cursor::*;
use enter::*;
use grenades::*;
use hud::*;
use original::map::{MapObject, ZMap};
use original::objects::{BuildingType, CannonType, ItemType, ObjectKind, RobotType, VehicleType};
use original::settings::{
    AGRO_DISTANCE, AUTO_GRAB_VEHICLE_DISTANCE, GRENADE_ATTACK_SPEED, GRENADE_DAMAGE,
    GRENADE_DAMAGE_RADIUS, GRENADE_MISSILE_SPEED, GRENADE_SCATTER_HALF_EXTENT,
    MAP_ITEM_TURRENT_DAMAGE, MAP_ITEM_TURRENT_RADIUS, RUN_RECHARGE_RATE, RUN_UNIT_SPEED,
};
use original::tileinfo::parse_palette_tile_info;
use original::types::{PlanetType, TeamType};
use placement::*;
use production::*;
use production_ui::*;
use render::atlas::{
    BridgeVisualState, FactoryOverlayKind, GameAtlases, MobileSpriteRole, RadarOverlayKind,
    RepairOverlayKind,
};
use render::rocks::spawn_rocks;
use render::terrain::{animate_terrain_effects, spawn_terrain, spawn_zone_overlays};
use repair::*;
use robot_groups::*;
use selection::*;
#[cfg(test)]
use units::buildings::{BuildingDeathProfile, BuildingEffectBox};
use units::vehicles::crane as crane_unit;
use units::vehicles::crane::{
    CraneConcoCraneSnapshot, CraneConcoEffect, CraneConcoPart, CraneConcoPhase,
    CraneConcoRenderItem, CraneConcoTargetSnapshot,
};
use units::{
    DamageMissileVisualGeometry, RocketImpactProfile, UnitAttackSound, buildings, cannons,
    combat_object_default_size,
    items::{animal, grenades as item_grenades, map_object, rock as item_rock},
    robots::{self, SpecialProjectileKind},
    vehicles,
};
use world_objects::{object_kind_to_map_parts, spawn_objects, spawn_runtime_object};
use zones::*;

const DIRECT_FIRE_BULLET_SPEED: f32 = 300.0;
const SPECIAL_PROJECTILE_SPEED: f32 = 300.0;
const SPECIAL_PROJECTILE_FRAME_TIME: f32 = 0.05;
const LIGHT_ROCKET_INIT_FIRE_FRAME_TIME: f32 = 0.02;
const DAMAGE_DEATH_CAUSE_WINDOW: f32 = 1.5;
const DAMAGE_MISSILE_FRAME_TIME: f32 = 0.1;
const SIDE_EXPLOSION_FRAME_TIME: f32 = 0.13;
const VEHICLE_DEATH_STANDARD_FRAME_TIME: f32 = 0.15;
const BIRD_MAP_PADDING: i32 = 160;
const BIRD_TILE_DENSITY: u32 = 650;
const BIRD_CITY_FRAME_TIME: f32 = 0.03;
const BIRD_DEFAULT_FRAME_TIME: f32 = 0.3;
const BIRD_FRAME_COUNT: usize = 5;
const BIRD_SOUND_SIZE: Vec2 = Vec2::new(16.0, 16.0);
const FORT_UNDER_ATTACK_SCAN_INTERVAL: f32 = 0.25;
const FORT_UNDER_ATTACK_DISTANCE: f32 = 250.0;
const FORT_UNDER_ATTACK_VERBAL_COOLDOWN: f32 = 10.0;
const LOSING_VERBAL_WARNING_COOLDOWN: f32 = 8.0;
const LOSING_VERBAL_WARNING_FACTOR: f32 = 1.7;
const COMPUTER_LOSING_MESSAGE_COUNT: usize = 10;
const TEAM_TYPE_COUNT: usize = 9;

impl Default for CombatRng {
    fn default() -> Self {
        Self(0x5eed_5eed)
    }
}

impl CombatRng {
    pub(crate) fn next_roll(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.0 >> 8) % 10_000) as f32 / 10_000.0
    }

    pub(crate) fn scatter(&mut self, half_extent: f32) -> f32 {
        (self.next_roll() * 2.0 - 1.0) * half_extent
    }

    pub(crate) fn index(&mut self, max: usize) -> usize {
        if max == 0 {
            0
        } else {
            (self.next_roll() * max as f32)
                .floor()
                .min((max - 1) as f32) as usize
        }
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(wgpu_settings()),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Zod Rust Bevy Port".to_string(),
                        resolution: primary_window_resolution(),
                        fit_canvas_to_parent: false,
                        canvas: Some("#bevy".to_string()),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.018)))
        .insert_resource(SelectionState::default())
        .insert_resource(MouseCommandState::default())
        .insert_resource(ZCursorState::default())
        .insert_resource(HudCommandQueue::default())
        .insert_resource(HudCommandState::default())
        .insert_resource(HudAttackAlert::default())
        .insert_resource(FortUnderAttackWarning::default())
        .insert_resource(LosingVerbalWarning::default())
        .insert_resource(CannonPlacementState::default())
        .insert_resource(ProductionWindowState::default())
        .insert_resource(ProductionDebugOpen::from_env())
        .insert_resource(StartupScreenshot::from_env())
        .insert_resource(CombatRng::default())
        .insert_resource(CraterStampRegistry::default())
        .insert_resource(PassiveEngageTimer(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )))
        .insert_resource(FlagCaptureTimer(Timer::from_seconds(
            0.2,
            TimerMode::Repeating,
        )))
        .add_systems(
            Startup,
            (load_original_map, setup_assets, setup_camera).chain(),
        )
        .add_systems(Startup, spawn_map.after(setup_assets))
        .add_systems(Startup, spawn_zcursor.after(setup_assets))
        .add_systems(Startup, open_debug_production_window.after(spawn_map))
        .add_systems(
            Update,
            (
                camera_controls,
                handle_minimap_camera_focus,
                sync_game_camera_viewport,
                handle_hud_button_input,
                process_hud_commands,
                handle_production_window_input,
                process_cannon_placement,
                handle_mouse_commands,
                process_eject_driver_commands,
                process_passive_engage,
                process_flag_captures,
                tick_damage_cause_timers,
                process_attack_targets,
                process_direct_fire_bullets,
                process_damage_missiles,
                process_building_production,
                process_destroyed_fort_eliminations,
                process_building_auto_repairs,
                process_destroyed_objects,
                update_zcursor,
            )
                .chain(),
        )
        .add_systems(
            Update,
            process_fort_under_attack_warning
                .after(process_attack_targets)
                .before(process_direct_fire_bullets),
        )
        .add_systems(
            Update,
            process_losing_verbal_warning.after(process_destroyed_objects),
        )
        .add_systems(
            Update,
            process_special_projectile_effects.after(process_attack_targets),
        )
        .add_systems(
            Update,
            process_damage_missile_replicas.after(process_damage_missiles),
        )
        .add_systems(
            Update,
            (
                animal::process_hut_animal_spawners,
                animal::process_hut_animal_movement,
            )
                .chain(),
        )
        .add_systems(Update, process_ambient_birds)
        .add_systems(
            Update,
            process_building_standard_effects.after(process_destroyed_objects),
        )
        .add_systems(
            Update,
            (
                move_commanded_objects,
                process_repair_targets.after(move_commanded_objects),
                process_grenade_pickups.after(move_commanded_objects),
                process_enter_targets.after(move_commanded_objects),
                process_enter_fort_targets.after(move_commanded_objects),
                update_selection_markers,
                update_production_window,
                animate_atlas_sprites,
                animate_image_effects,
                animate_dynamic_image_effects,
                animate_looping_image_effects,
                process_vehicle_death_effects,
                process_cannon_death_effects,
                process_bridge_revive_pending,
                animate_cannon_turrent_missile_effects,
                animate_death_spark_effects,
                animate_bridge_turrent_effects,
                animate_bridge_rock_particle_effects,
                animate_robot_turrent_death_effects,
                capture_startup_screenshot,
            ),
        )
        .add_systems(
            Update,
            sync_crane_conco_effects.after(process_repair_targets),
        )
        .add_systems(
            Update,
            animate_crane_conco_effects.after(sync_crane_conco_effects),
        )
        .add_systems(Update, animate_vehicle_track_effects)
        .add_systems(Update, animate_radar_overlays)
        .add_systems(Update, animate_repair_overlays)
        .add_systems(Update, animate_factory_overlays)
        .add_systems(Update, animate_terrain_effects)
        .add_systems(Update, animate_unit_particle_effects)
        .add_systems(Update, animate_rock_turrent_effects)
        .add_systems(Update, animate_building_turrent_missile_effects)
        .add_systems(
            Update,
            (
                update_hud_button_availability,
                update_hud_attack_alert,
                update_hud_computer_message,
                update_minimap_dots,
                update_minimap_view_box,
                update_hud_selected_object,
                update_hud_grenade_indicator,
                update_hud_health_bar,
                update_hud_anchors,
            )
                .chain(),
        )
        .run();
}

fn primary_window_resolution() -> WindowResolution {
    let resolution = WindowResolution::new(800, 600);

    #[cfg(target_arch = "wasm32")]
    {
        return resolution.with_scale_factor_override(1.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        resolution
    }
}

fn wgpu_settings() -> WgpuSettings {
    #[cfg(target_arch = "wasm32")]
    {
        WgpuSettings {
            backends: Some(Backends::GL),
            priority: WgpuSettingsPriority::WebGL2,
            ..default()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        WgpuSettings::default()
    }
}

fn load_original_map(mut commands: Commands) {
    let (map_bytes, map_source) = starting_map_bytes();
    let map = ZMap::parse(&map_bytes).expect("original .map should parse");
    let tile_info = parse_palette_tile_info(tileinfo_bytes(map.basics.terrain_type))
        .expect("embedded original .tileinfo should parse");
    println!(
        "Loaded map '{}' from {map_source} {}x{}, {} players, {} zones, {} objects, {} tile metadata records",
        map.basics.map_name,
        map.basics.width,
        map.basics.height,
        map.basics.player_count,
        map.zones.len(),
        map.objects.len(),
        tile_info.len()
    );
    commands.insert_resource(CurrentMap(map));
    commands.insert_resource(CurrentTileInfo(tile_info));
}

fn starting_map_bytes() -> (Vec<u8>, String) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(path) = std::env::var("ZOD_MAP") {
            return (
                std::fs::read(&path).unwrap_or_else(|err| {
                    panic!("failed to read ZOD_MAP={path}: {err}");
                }),
                path,
            );
        }
    }

    (
        STARTING_MAP.to_vec(),
        "embedded maps/p02_bb_orig01.map".to_string(),
    )
}

fn setup_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        20,
        24,
        None,
        None,
    ));

    commands.insert_resource(PlanetAtlas {
        desert: asset_server.load("planets/desert.bmp"),
        volcanic: asset_server.load("planets/volcanic.bmp"),
        arctic: asset_server.load("planets/arctic.bmp"),
        jungle: asset_server.load("planets/jungle.bmp"),
        city: asset_server.load("planets/city.bmp"),
        layout,
    });

    let rock_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        6,
        6,
        None,
        None,
    ));

    commands.insert_resource(RockAtlas {
        desert: asset_server.load("planets/rocks_desert.png"),
        volcanic: asset_server.load("planets/rocks_volcanic.png"),
        arctic: asset_server.load("planets/rocks_arctic.png"),
        jungle: asset_server.load("planets/rocks_jungle.png"),
        city: asset_server.load("planets/rocks_city.png"),
        layout: rock_layout,
    });

    commands.insert_resource(GameAtlases::build(&asset_server, &mut layouts));
    commands.insert_resource(load_production_ui_assets(&asset_server));
    commands.insert_resource(load_cursor_assets(&asset_server));
    commands.insert_resource(HudAssets {
        side_panel: asset_server.load("other/hud/main_hud_side_red.png"),
        side_filler: asset_server.load("other/hud/side_filler.bmp"),
        bottom_left: asset_server.load("other/hud/main_hud_bottom_left.bmp"),
        bottom_center: asset_server.load("other/hud/main_hud_bottom_center.bmp"),
        bottom_right: asset_server.load("other/hud/main_hud_bottom_right.bmp"),
        health_full: asset_server.load("other/hud/health_full.png"),
        health_lost: asset_server.load("other/hud/health_lost.png"),
        health_empty: asset_server.load("other/hud/health_empty.png"),
        grenade_icons: grenade_icon_assets(&asset_server),
        fort_under_attack_message: asset_server.load("other/comp_messages/fort_under_attack.png"),
        font: asset_server.load("arial.ttf"),
        buttons: hud_button_specs()
            .iter()
            .map(|spec| load_hud_button_images(&asset_server, spec.asset_name))
            .collect(),
    });
}

fn grenade_icon_assets(asset_server: &AssetServer) -> Vec<(TeamType, Handle<Image>)> {
    [
        TeamType::Null,
        TeamType::Red,
        TeamType::Blue,
        TeamType::Green,
        TeamType::Yellow,
    ]
    .into_iter()
    .map(|team| {
        (
            team,
            asset_server.load(format!("other/hud/{}", grenade_icon_asset_name(team))),
        )
    })
    .collect()
}

fn grenade_icon_asset_name(team: TeamType) -> String {
    let asset_team = if team == TeamType::Null {
        TeamType::Null
    } else {
        team.atlas_team()
    };
    format!("icon_grenade_{}.png", asset_team.asset_name())
}

fn load_hud_button_images(asset_server: &AssetServer, asset_name: &str) -> HudButtonImages {
    HudButtonImages {
        active: asset_server.load(format!("other/hud/{asset_name}_active.bmp")),
        inactive: asset_server.load(format!("other/hud/{asset_name}_inactive.bmp")),
        pressed: asset_server.load(format!("other/hud/{asset_name}_pressed.bmp")),
    }
}

fn spawn_map(
    mut commands: Commands,
    map: Res<CurrentMap>,
    tile_info: Res<CurrentTileInfo>,
    asset_server: Res<AssetServer>,
    atlas: Res<PlanetAtlas>,
    rock_atlas: Res<RockAtlas>,
    game_atlases: Res<GameAtlases>,
    hud_assets: Res<HudAssets>,
    mut rng: ResMut<CombatRng>,
) {
    commands.insert_resource(PassabilityGrid::build(&map.0, &tile_info.0));
    spawn_terrain(&mut commands, &map.0, &tile_info.0, &atlas, &mut rng);
    spawn_rocks(&mut commands, &map.0, &rock_atlas);
    spawn_zone_overlays(&mut commands, &map.0);
    spawn_ambient_birds(&mut commands, &map.0, &asset_server, &mut rng);
    let next_ref_id = spawn_objects(&mut commands, &map.0, &game_atlases);
    let zone_ownership = ZoneOwnership::from_map(&map.0);
    spawn_hud(&mut commands, &map.0, &hud_assets, &zone_ownership);
    commands.insert_resource(zone_ownership);
    commands.insert_resource(NextObjectRefId(next_ref_id));
}

fn spawn_ambient_birds(
    commands: &mut Commands,
    map: &ZMap,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
) {
    let count = ambient_bird_count(map.basics.width, map.basics.height);
    let map_size = Vec2::new(
        map.basics.width as f32 * TILE_SIZE,
        map.basics.height as f32 * TILE_SIZE,
    );
    let now = 0.0;

    for index in 0..count {
        let bird = reset_ambient_bird(map.basics.terrain_type, map_size, now, rng);
        let render_position = ambient_bird_render_position(bird.position_map, bird.rise);
        let world = map_top_left_to_world(render_position);
        commands.spawn((
            Sprite::from_image(
                asset_server.load(ambient_bird_frame_path(bird.planet, bird.render_frame)),
            ),
            bevy::sprite::Anchor::CENTER,
            Transform {
                translation: Vec3::new(world.x, world.y, ambient_bird_z()),
                rotation: Quat::from_rotation_z(bird.angle_degrees.to_radians()),
                scale: Vec3::splat(bird.rise),
            },
            bird,
            Name::new(format!("ambient_bird_{index}")),
        ));
    }
}

fn tileinfo_bytes(planet: PlanetType) -> &'static [u8] {
    match planet {
        PlanetType::Desert => include_bytes!("../assets/planets/desert.tileinfo"),
        PlanetType::Volcanic => include_bytes!("../assets/planets/volcanic.tileinfo"),
        PlanetType::Arctic => include_bytes!("../assets/planets/arctic.tileinfo"),
        PlanetType::Jungle => include_bytes!("../assets/planets/jungle.tileinfo"),
        PlanetType::City => include_bytes!("../assets/planets/city.tileinfo"),
    }
}

#[derive(Clone, Copy)]
struct CombatObjectSnapshot {
    ref_id: u32,
    kind: ObjectKind,
    position: Vec2,
    size: Vec2,
    team: TeamType,
    stats: ObjectStats,
    attack_target_ref: Option<u32>,
}

#[derive(Clone, Copy)]
struct FortWarningSnapshot {
    ref_id: u32,
    kind: ObjectKind,
    position: Vec2,
    team: TeamType,
    destroyed: bool,
}

#[derive(Clone, Copy)]
struct PendingAttackDamage {
    target_ref_id: u32,
    attacker_kind: ObjectKind,
    attacker_stats: ObjectStats,
    target_can_be_sniped: bool,
    damage: f32,
    damage_chance: f32,
}

#[derive(Clone, Copy)]
struct ObjectLayerSnapshot {
    entity: Entity,
    ref_id: u32,
    position: Vec2,
}

#[derive(Clone, Copy)]
struct PassiveAutoEnterRobotSnapshot {
    ref_id: u32,
    position: Vec2,
    has_waypoint: bool,
    has_attack_target: bool,
    has_task_target: bool,
    is_minion: bool,
    just_left_cannon: bool,
}

#[derive(Clone, Copy)]
struct PassiveAutoEnterTargetSnapshot {
    ref_id: u32,
    kind: ObjectKind,
    position: Vec2,
}

#[derive(Clone, Copy)]
struct PassiveGrenadeBoxSnapshot {
    ref_id: u32,
    position: Vec2,
}

#[derive(Clone, Copy)]
struct DestroyedObjectSnapshot {
    ref_id: u32,
    kind: ObjectKind,
    team: TeamType,
    position: Vec2,
    grid: MapGridPosition,
    mobile_rotation: u16,
    mobile_frame: usize,
    grenade_amount: u8,
    bridge: Option<BridgeFootprint>,
    do_fire_death: bool,
    do_missile_death: bool,
}

#[derive(Clone, Copy)]
struct MovementRunRequest {
    ref_id: u32,
    target: Vec2,
    speed: f32,
    attempt_run: bool,
}

#[derive(Clone, Copy)]
struct MovementSpeedSnapshot {
    ref_id: u32,
    multiplier: f32,
}

#[derive(Component)]
struct RobotTurrentDeathEffect {
    frames: Vec<Handle<Image>>,
    start: Vec2,
    end: Vec2,
    elapsed: f32,
    final_time: f32,
    rise: f32,
    frame_elapsed: f32,
    current: usize,
    landed: bool,
}

#[derive(Component)]
struct DynamicImageEffect {
    frames: Vec<Handle<Image>>,
    frame_time: f32,
    elapsed: f32,
    age: f32,
    current: usize,
    base_map_position: Vec2,
    velocity_map: Vec2,
    frame_offsets: Vec<Vec2>,
    scale: f32,
}

#[derive(Component)]
struct SpecialProjectileEffect {
    kind: SpecialProjectileKind,
    start: Vec2,
    target: Vec2,
    time_remaining: f32,
    total_time: f32,
    frames: Vec<Handle<Image>>,
    frame_time: f32,
    frame_elapsed: f32,
    current: usize,
}

#[derive(Component)]
struct DamageMissileReplica {
    start: Vec2,
    target: Vec2,
    time_remaining: f32,
    total_time: f32,
    offset: Vec2,
    frames: Vec<Handle<Image>>,
    frame_time: f32,
    frame_elapsed: f32,
    frame: usize,
}

#[derive(Component)]
struct CannonDeathEffect {
    cannon: CannonType,
    timer: f32,
    start_map: Vec2,
    end_map: Vec2,
    missile_time: f32,
    rise: f32,
    image: Handle<Image>,
}

#[derive(Component)]
struct CannonTurrentMissileEffect {
    cannon: CannonType,
    start_map: Vec2,
    end_map: Vec2,
    elapsed: f32,
    final_time: f32,
    rise: f32,
    angle_degrees_per_sec: f32,
}

#[derive(Component)]
struct DeathSparkEffect {
    frames: Vec<Handle<Image>>,
    start_map: Vec2,
    velocity_map: Vec2,
    elapsed: f32,
    final_time: f32,
    rise: f32,
    frame_elapsed: f32,
    current: usize,
}

#[derive(Component)]
struct BuildingTurrentMissileEffect {
    frames: Vec<Handle<Image>>,
    start_map: Vec2,
    end_map: Vec2,
    elapsed: f32,
    final_time: f32,
    rise: f32,
    angle_degrees_per_sec: f32,
    frame_elapsed: f32,
    current: usize,
}

#[derive(Component)]
struct BridgeTurrentEffect {
    frames: Vec<Handle<Image>>,
    start_map: Vec2,
    end_map: Vec2,
    elapsed: f32,
    final_time: f32,
    rise: f32,
    angle_degrees_per_sec: f32,
    frame_elapsed: f32,
    current: usize,
    reversed: bool,
    planet: PlanetType,
}

#[derive(Component)]
struct VehicleTrackEffect {
    frames: Vec<Handle<Image>>,
    elapsed: f32,
    start_delay: f32,
    current: usize,
}

#[derive(Component)]
struct BridgeRockParticleEffect {
    frames: Vec<Handle<Image>>,
    start_map: Vec2,
    velocity_map: Vec2,
    elapsed: f32,
    final_time: f32,
    rise: f32,
    frame_elapsed: f32,
    current: usize,
}

#[derive(Component)]
struct UnitParticleEffect {
    frames: Vec<Handle<Image>>,
    start_map: Vec2,
    velocity_map: Vec2,
    elapsed: f32,
    final_time: f32,
    rise: f32,
    frame_elapsed: f32,
    current: usize,
}

#[derive(Component)]
struct RockTurrentEffect {
    frames: Vec<Handle<Image>>,
    start_map: Vec2,
    velocity_map: Vec2,
    elapsed: f32,
    final_time: f32,
    rise: f32,
    frame_elapsed: f32,
    current: usize,
    angle_degrees_per_sec: f32,
    planet: PlanetType,
}

#[derive(Component)]
struct VehicleDeathEffect {
    timer: f32,
    top_left_map: Vec2,
}

#[derive(Component)]
struct LoopingImageEffect {
    frames: Vec<Handle<Image>>,
    frame_time: f32,
    elapsed: f32,
    lifetime: f32,
    current: usize,
}

#[derive(Component)]
struct BuildingEffectState {
    max_effects: usize,
}

#[derive(Component)]
struct BuildingStandardEffect {
    ref_id: u32,
}

#[derive(Default, Resource)]
struct CraterStampRegistry {
    stamped_tiles: HashSet<(i32, i32)>,
}

#[derive(Clone, Debug, PartialEq)]
struct CraterStampSpec {
    tile: IVec2,
    is_big: bool,
    crater_type: i16,
    variant: usize,
    asset_path: String,
}

fn process_ambient_birds(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut rng: ResMut<CombatRng>,
    mut birds: Query<(&mut AmbientBird, &mut Transform, &mut Sprite), Without<MainCamera>>,
) {
    let now = time.elapsed_secs();
    let sound_view =
        ambient_bird_sound_view_rect_from_world(&windows, &camera_query).unwrap_or(MapViewRect {
            top_left: Vec2::ZERO,
            size: Vec2::ZERO,
        });

    for (mut bird, mut transform, mut sprite) in &mut birds {
        let sound_request = advance_ambient_bird_state(&mut bird, now, &mut rng);
        if let Some(sound_request) = sound_request
            && map_rect_intersects_view(
                sound_request.position_map,
                sound_request.restricted_size,
                sound_view,
            )
        {
            play_ambient_bird_sound(&mut commands, &asset_server, sound_request.kind);
        }

        let render_position = ambient_bird_render_position(bird.position_map, bird.rise);
        let world = map_top_left_to_world(render_position);
        transform.translation = Vec3::new(world.x, world.y, ambient_bird_z());
        transform.rotation = Quat::from_rotation_z(bird.angle_degrees.to_radians());
        transform.scale = Vec3::splat(bird.rise);
        sprite.image = asset_server.load(ambient_bird_frame_path(bird.planet, bird.render_frame));
    }
}

fn play_ambient_bird_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    kind: AmbientBirdSoundKind,
) {
    play_game_sound(
        commands,
        asset_server,
        GameSoundKind::AmbientBird(kind),
        None,
    );
}

fn ambient_bird_count(width_tiles: u16, height_tiles: u16) -> u32 {
    u32::from(width_tiles) * u32::from(height_tiles) / BIRD_TILE_DENSITY
}

fn reset_ambient_bird(
    planet: PlanetType,
    map_size: Vec2,
    now: f32,
    rng: &mut CombatRng,
) -> AmbientBird {
    let position_map = ambient_bird_reset_position(map_size, rng);
    let angle_degrees = ambient_bird_angle_to_center(position_map, map_size);
    let speed = ambient_bird_speed(planet, rng);
    let render_frame = 0;

    AmbientBird {
        planet,
        map_size,
        position_map,
        fractional_shift: Vec2::ZERO,
        angle_degrees,
        dangle: 0.0,
        rise: 1.0,
        render_frame,
        speed,
        next_render_time: now + ambient_bird_frame_interval(planet),
        last_process_time: now,
        next_dangle_time: ambient_bird_next_straight_end(now, rng),
        next_caw_sound_time: ambient_bird_next_initial_caw_time(now, rng),
        next_height_change_time: ambient_bird_next_initial_height_time(now, rng),
        rise_change_end: now,
        rise_change_start_time: now,
        rise_change_target: 1.0,
        rise_change_start: 1.0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AmbientBirdSoundKind {
    BatChirp,
    Crow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AmbientBirdSoundRequest {
    kind: AmbientBirdSoundKind,
    position_map: Vec2,
    restricted_size: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MapViewRect {
    top_left: Vec2,
    size: Vec2,
}

fn advance_ambient_bird_state(
    bird: &mut AmbientBird,
    now: f32,
    rng: &mut CombatRng,
) -> Option<AmbientBirdSoundRequest> {
    let mut sound_request = None;

    if now >= bird.next_render_time {
        bird.next_render_time = now + ambient_bird_frame_interval(bird.planet);
        bird.render_frame = (bird.render_frame + 1) % BIRD_FRAME_COUNT;
    }

    if now >= bird.next_dangle_time {
        if bird.dangle.abs() < 0.00001 {
            bird.next_dangle_time = ambient_bird_next_turn_end(now, rng);
            bird.dangle = rng.index(100) as f32 - 50.0;
        } else {
            bird.next_dangle_time = ambient_bird_next_straight_end(now, rng);
            bird.dangle = 0.0;
        }
    }

    if now >= bird.next_caw_sound_time {
        bird.next_caw_sound_time = ambient_bird_next_later_caw_time(now, rng);
        sound_request = ambient_bird_caw_request(bird.planet, bird.position_map, rng);
    }

    if now >= bird.next_height_change_time {
        bird.next_height_change_time = ambient_bird_next_later_height_time(now, rng);
        bird.rise_change_end = ambient_bird_height_change_end(now, rng);
        bird.rise_change_start_time = now;
        bird.rise_change_start = bird.rise;
        bird.rise_change_target = ambient_bird_height_target(rng);
    }

    if bird.rise != bird.rise_change_target {
        if now > bird.rise_change_end {
            bird.rise = bird.rise_change_target;
        } else {
            let duration = bird.rise_change_end - bird.rise_change_start_time;
            if duration > 0.0 {
                let t = ((now - bird.rise_change_start_time) / duration).clamp(0.0, 1.0);
                bird.rise =
                    bird.rise_change_start + (bird.rise_change_target - bird.rise_change_start) * t;
            }
        }
    }

    let time_diff = now - bird.last_process_time;
    bird.last_process_time = now;
    bird.angle_degrees = normalize_degrees(bird.angle_degrees + bird.dangle * time_diff);

    let radians = bird.angle_degrees.to_radians();
    let dx = bird.speed * radians.cos();
    let dy = -bird.speed * radians.sin();
    let shift = Vec2::new(dx * time_diff, dy * time_diff) + bird.fractional_shift;
    let whole = Vec2::new(shift.x as i32 as f32, shift.y as i32 as f32);
    bird.position_map += whole;
    bird.fractional_shift = shift - whole;

    if ambient_bird_out_of_bounds(bird.position_map, bird.map_size) {
        *bird = reset_ambient_bird(bird.planet, bird.map_size, now, rng);
    }

    sound_request
}

fn ambient_bird_caw_request(
    planet: PlanetType,
    position_map: Vec2,
    rng: &mut CombatRng,
) -> Option<AmbientBirdSoundRequest> {
    (rng.index(3) == 0).then(|| AmbientBirdSoundRequest {
        kind: ambient_bird_caw_sound(planet),
        position_map,
        restricted_size: BIRD_SOUND_SIZE,
    })
}

fn ambient_bird_caw_sound(planet: PlanetType) -> AmbientBirdSoundKind {
    match planet {
        PlanetType::City => AmbientBirdSoundKind::BatChirp,
        _ => AmbientBirdSoundKind::Crow,
    }
}

fn ambient_bird_sound_asset_path(kind: AmbientBirdSoundKind) -> &'static str {
    match kind {
        AmbientBirdSoundKind::BatChirp => "sounds/BATCHIRP.wav",
        AmbientBirdSoundKind::Crow => "sounds/CROW2.wav",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GameSoundKind {
    AmbientBird(AmbientBirdSoundKind),
    RifleFire,
    PsychoFire,
    ToughFire,
    PyroFire,
    LaserFire,
    GunFire,
    GatlingFire,
    JeepFire,
    LightFire,
    MediumFire,
    HeavyFire,
    MobileMissileFire,
    Ricochet,
    RandomExplosion,
    TurrentExplosion,
    ThrowGrenade,
    ComputerFortUnderAttack,
    ComputerYouAreLosing,
}

impl From<UnitAttackSound> for GameSoundKind {
    fn from(sound: UnitAttackSound) -> Self {
        match sound {
            UnitAttackSound::Rifle => Self::RifleFire,
            UnitAttackSound::Psycho => Self::PsychoFire,
            UnitAttackSound::Tough => Self::ToughFire,
            UnitAttackSound::Pyro => Self::PyroFire,
            UnitAttackSound::Laser => Self::LaserFire,
            UnitAttackSound::Gun => Self::GunFire,
            UnitAttackSound::Gatling => Self::GatlingFire,
            UnitAttackSound::Jeep => Self::JeepFire,
            UnitAttackSound::Light => Self::LightFire,
            UnitAttackSound::Medium => Self::MediumFire,
            UnitAttackSound::Heavy => Self::HeavyFire,
            UnitAttackSound::MobileMissile => Self::MobileMissileFire,
            UnitAttackSound::ThrowGrenade => Self::ThrowGrenade,
        }
    }
}

fn play_game_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    kind: GameSoundKind,
    rng: Option<&mut CombatRng>,
) {
    let path = game_sound_asset_path(kind, rng);
    commands.spawn((
        AudioPlayer::new(asset_server.load::<AudioSource>(path)),
        PlaybackSettings::DESPAWN,
    ));
}

fn play_restricted_game_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    windows: &Query<&Window>,
    camera_query: &Query<&Transform, With<MainCamera>>,
    kind: GameSoundKind,
    position_map: Vec2,
    size: Vec2,
    rng: Option<&mut CombatRng>,
) {
    let Some(view) = ambient_bird_sound_view_rect_from_world(windows, camera_query) else {
        return;
    };
    if map_rect_intersects_view(position_map, size, view) {
        play_game_sound(commands, asset_server, kind, rng);
    }
}

fn game_sound_asset_path(kind: GameSoundKind, rng: Option<&mut CombatRng>) -> String {
    match kind {
        GameSoundKind::AmbientBird(kind) => ambient_bird_sound_asset_path(kind).to_string(),
        GameSoundKind::RifleFire => "sounds/RIFLE3.wav".to_string(),
        GameSoundKind::PsychoFire => "sounds/MACHGUN2.wav".to_string(),
        GameSoundKind::ToughFire => "sounds/MOBIMISS.wav".to_string(),
        GameSoundKind::PyroFire => "sounds/FLAMER.wav".to_string(),
        GameSoundKind::LaserFire => "sounds/LASERGUN.wav".to_string(),
        GameSoundKind::GunFire => "sounds/LTGUN.wav".to_string(),
        GameSoundKind::GatlingFire => "sounds/GATTGUN.wav".to_string(),
        GameSoundKind::JeepFire => "sounds/JEEPMGUN.wav".to_string(),
        GameSoundKind::LightFire => "sounds/LTANKGUN.wav".to_string(),
        GameSoundKind::MediumFire => "sounds/MTANKGUN.wav".to_string(),
        GameSoundKind::HeavyFire => "sounds/HTANKGUN.wav".to_string(),
        GameSoundKind::MobileMissileFire => "sounds/MOBIMIS2.wav".to_string(),
        GameSoundKind::Ricochet => "sounds/RICOCH1.wav".to_string(),
        GameSoundKind::TurrentExplosion => "sounds/METGRND.wav".to_string(),
        GameSoundKind::ThrowGrenade => "sounds/GRENLOBX.wav".to_string(),
        GameSoundKind::ComputerFortUnderAttack => "sounds/comp_fort_under_attack.wav".to_string(),
        GameSoundKind::ComputerYouAreLosing => {
            let index = rng.map_or(0, |rng| rng.index(COMPUTER_LOSING_MESSAGE_COUNT));
            format!("sounds/comp_youre_losing_{index:02}.wav")
        }
        GameSoundKind::RandomExplosion => {
            let index = rng.map_or(0, |rng| rng.index(5));
            format!("sounds/explosion_{index:02}.wav")
        }
    }
}

fn ambient_bird_sound_view_rect_from_world(
    windows: &Query<&Window>,
    camera_query: &Query<&Transform, With<MainCamera>>,
) -> Option<MapViewRect> {
    let window = windows.single().ok()?;
    let camera = camera_query.single().ok()?;
    Some(ambient_bird_sound_view_rect(
        camera.translation.truncate(),
        game_view_size(window),
    ))
}

fn ambient_bird_sound_view_rect(camera_center_world: Vec2, view_size: Vec2) -> MapViewRect {
    MapViewRect {
        top_left: Vec2::new(
            camera_center_world.x - view_size.x * 0.5,
            -camera_center_world.y - view_size.y * 0.5,
        ),
        size: view_size,
    }
}

fn map_rect_intersects_view(position: Vec2, size: Vec2, view: MapViewRect) -> bool {
    position.x <= view.top_left.x + view.size.x
        && position.y <= view.top_left.y + view.size.y
        && position.x + size.x >= view.top_left.x
        && position.y + size.y >= view.top_left.y
}

fn ambient_bird_reset_position(map_size: Vec2, rng: &mut CombatRng) -> Vec2 {
    let padding = BIRD_MAP_PADDING;
    let x_extent = map_size.x as i32 + padding * 2;
    let y_extent = map_size.y as i32 + padding * 2;

    match rng.index(4) {
        0 => Vec2::new(
            rng.index(x_extent.max(1) as usize) as i32 as f32 - padding as f32,
            -((rng.index(padding as usize) as i32 + 16) as f32),
        ),
        1 => Vec2::new(
            rng.index(x_extent.max(1) as usize) as i32 as f32 - padding as f32,
            map_size.y + (rng.index(padding as usize) as i32 + 16) as f32,
        ),
        2 => Vec2::new(
            -((rng.index(padding as usize) as i32 + 16) as f32),
            rng.index(y_extent.max(1) as usize) as i32 as f32 - padding as f32,
        ),
        _ => Vec2::new(
            map_size.x + (rng.index(padding as usize) as i32 + 16) as f32,
            rng.index(y_extent.max(1) as usize) as i32 as f32 - padding as f32,
        ),
    }
}

fn ambient_bird_angle_to_center(position_map: Vec2, map_size: Vec2) -> f32 {
    let center = map_size * 0.5;
    normalize_degrees(
        (-(center.y - position_map.y))
            .atan2(center.x - position_map.x)
            .to_degrees(),
    )
}

fn ambient_bird_speed(planet: PlanetType, rng: &mut CombatRng) -> f32 {
    match planet {
        PlanetType::City => 80.0 + rng.index(20) as f32,
        _ => 20.0 + rng.index(10) as f32,
    }
}

fn ambient_bird_frame_interval(planet: PlanetType) -> f32 {
    match planet {
        PlanetType::City => BIRD_CITY_FRAME_TIME,
        _ => BIRD_DEFAULT_FRAME_TIME,
    }
}

fn ambient_bird_render_position(position_map: Vec2, rise: f32) -> Vec2 {
    Vec2::new(position_map.x, position_map.y - ((rise - 1.0) * 50.0))
}

fn ambient_bird_z() -> f32 {
    33.0
}

fn ambient_bird_out_of_bounds(position_map: Vec2, map_size: Vec2) -> bool {
    position_map.x < -BIRD_MAP_PADDING as f32
        || position_map.y < -BIRD_MAP_PADDING as f32
        || position_map.x > map_size.x + BIRD_MAP_PADDING as f32
        || position_map.y > map_size.y + BIRD_MAP_PADDING as f32
}

fn ambient_bird_next_straight_end(now: f32, rng: &mut CombatRng) -> f32 {
    now + 9.0 + rng.index(50) as f32 * 0.1
}

fn ambient_bird_next_turn_end(now: f32, rng: &mut CombatRng) -> f32 {
    now + 5.0 + rng.index(30) as f32 * 0.1
}

fn ambient_bird_next_initial_caw_time(now: f32, rng: &mut CombatRng) -> f32 {
    now + 3.0 + rng.index(50) as f32 * 0.1
}

fn ambient_bird_next_later_caw_time(now: f32, rng: &mut CombatRng) -> f32 {
    now + 15.0 + rng.index(50) as f32 * 0.1
}

fn ambient_bird_next_initial_height_time(now: f32, rng: &mut CombatRng) -> f32 {
    now + 5.0 + rng.index(100) as f32 * 0.1
}

fn ambient_bird_next_later_height_time(now: f32, rng: &mut CombatRng) -> f32 {
    now + 15.0 + rng.index(100) as f32 * 0.1
}

fn ambient_bird_height_change_end(now: f32, rng: &mut CombatRng) -> f32 {
    now + 4.0 + rng.index(60) as f32 * 0.1
}

fn ambient_bird_height_target(rng: &mut CombatRng) -> f32 {
    1.0 + rng.index(10) as f32 * 0.1
}

fn ambient_bird_frame_path(planet: PlanetType, frame: usize) -> String {
    format!(
        "other/birds/bird_{}_r000_n{:02}.png",
        planet_asset_name(planet),
        frame % BIRD_FRAME_COUNT
    )
}

fn normalize_degrees(mut angle: f32) -> f32 {
    while angle >= 360.0 {
        angle -= 360.0;
    }
    while angle < 0.0 {
        angle += 360.0;
    }
    angle
}

fn process_passive_engage(
    mut commands: Commands,
    time: Res<Time>,
    passability: Res<PassabilityGrid>,
    mut timer: ResMut<PassiveEngageTimer>,
    objects: Query<
        (
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&GrenadeInventory>,
            Option<&AttackTarget>,
            Option<&MovementPath>,
            Option<&EnterTarget>,
            Option<&EnterFortTarget>,
            Option<&PickupGrenadesTarget>,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RobotGroup>,
            Option<&JustLeftCannon>,
        ),
        (Without<DestroyedObject>, With<Selectable>),
    >,
    grenade_boxes: Query<
        (&GameObjectEntity, &Transform, &ObjectStats, &GrenadeBox),
        Without<DestroyedObject>,
    >,
    layers: Query<(Entity, &Transform, &ObjectLayerRef), Without<DestroyedObject>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let snapshots: Vec<CombatObjectSnapshot> = objects
        .iter()
        .map(
            |(object, transform, selectable, team, stats, _, _, _, _, _, _, _, _, _, _)| {
                CombatObjectSnapshot {
                    ref_id: object.ref_id,
                    kind: object.kind,
                    position: transform.translation.truncate(),
                    size: selectable.selection_size,
                    team: team.0,
                    stats: *stats,
                    attack_target_ref: None,
                }
            },
        )
        .collect();
    let layer_snapshots: Vec<ObjectLayerSnapshot> = layers
        .iter()
        .map(|(entity, transform, layer_ref)| ObjectLayerSnapshot {
            entity,
            ref_id: layer_ref.0,
            position: transform.translation.truncate(),
        })
        .collect();

    let auto_enter_targets: Vec<PassiveAutoEnterTargetSnapshot> = snapshots
        .iter()
        .copied()
        .filter_map(|snapshot| {
            (can_be_entered(snapshot.kind, snapshot.team, snapshot.stats)
                && !matches!(snapshot.kind, ObjectKind::Vehicle(VehicleType::Apc)))
            .then_some(PassiveAutoEnterTargetSnapshot {
                ref_id: snapshot.ref_id,
                kind: snapshot.kind,
                position: snapshot.position,
            })
        })
        .collect();
    let grenade_box_targets: Vec<PassiveGrenadeBoxSnapshot> = grenade_boxes
        .iter()
        .filter_map(|(object, transform, stats, box_amount)| {
            (!stats.destroyed() && box_amount.amount > 0 && is_grenade_box(object.kind)).then_some(
                PassiveGrenadeBoxSnapshot {
                    ref_id: object.ref_id,
                    position: transform.translation.truncate(),
                },
            )
        })
        .collect();

    for (
        object,
        transform,
        selectable,
        team,
        stats,
        inventory,
        maybe_target,
        maybe_movement,
        maybe_enter,
        maybe_enter_fort,
        maybe_pickup,
        maybe_crane_repair,
        maybe_unit_repair,
        maybe_group,
        maybe_just_left_cannon,
    ) in &objects
    {
        if team.0 == TeamType::Null || stats.destroyed() {
            continue;
        }

        let can_engage = can_passively_engage(
            object.kind,
            team.0,
            *stats,
            selectable.mobile,
            maybe_movement,
        );

        let position = transform.translation.truncate();
        let passive_task_robot = PassiveAutoEnterRobotSnapshot {
            ref_id: object.ref_id,
            position,
            has_waypoint: maybe_movement.is_some(),
            has_attack_target: maybe_target.is_some(),
            has_task_target: maybe_enter.is_some()
                || maybe_enter_fort.is_some()
                || maybe_pickup.is_some()
                || maybe_crane_repair.is_some()
                || maybe_unit_repair.is_some(),
            is_minion: maybe_group.is_some_and(|group| group.leader_ref_id != object.ref_id),
            just_left_cannon: maybe_just_left_cannon.is_some(),
        };

        if can_engage {
            if let Some(target) = maybe_target {
                let target_in_range = snapshots
                    .iter()
                    .find(|snapshot| snapshot.ref_id == target.ref_id)
                    .is_some_and(|target_snapshot| {
                        can_attack_target_with_grenades(
                            team.0,
                            *stats,
                            grenade_attack_amount(
                                object.kind,
                                inventory.map(|inventory| inventory.amount),
                            ),
                            target_snapshot.team,
                            target_snapshot.stats,
                            position.distance(target_snapshot.position),
                        )
                    });

                if !target_in_range {
                    remove_attack_target_for_ref(&mut commands, object.ref_id, &layer_snapshots);
                }
                continue;
            }

            if let Some(target) =
                passive_attack_target_choice(object.ref_id, position, team.0, *stats, &snapshots)
            {
                insert_attack_target_for_ref(
                    &mut commands,
                    object.ref_id,
                    target.ref_id,
                    false,
                    &layer_snapshots,
                );
                continue;
            }

            if matches!(object.kind, ObjectKind::Cannon(_)) || !selectable.mobile {
                continue;
            }

            if let Some(target) =
                passive_agro_target_choice(object.ref_id, position, team.0, *stats, &snapshots)
            {
                insert_movement_route_for_ref(
                    &mut commands,
                    object.ref_id,
                    object.kind,
                    position,
                    target.position,
                    stats.attack_radius,
                    stats.move_speed,
                    &passability,
                    &layer_snapshots,
                );
                insert_attack_target_for_ref(
                    &mut commands,
                    object.ref_id,
                    target.ref_id,
                    false,
                    &layer_snapshots,
                );
                continue;
            }
        }

        if let Some(target) = passive_grenade_pickup_target_choice(
            object.kind,
            selectable.mobile,
            passive_task_robot,
            inventory.map(|inventory| inventory.amount),
            &grenade_box_targets,
        ) {
            insert_grenade_pickup_route_for_ref(
                &mut commands,
                object.ref_id,
                position,
                target,
                stats.move_speed,
                &passability,
                &layer_snapshots,
            );
            continue;
        }

        if let Some(target) = passive_auto_enter_target_choice(
            object.kind,
            selectable.mobile,
            passive_task_robot,
            &auto_enter_targets,
        ) {
            insert_enter_route_for_ref(
                &mut commands,
                object.ref_id,
                position,
                target,
                stats.move_speed,
                &passability,
                &layer_snapshots,
            );
        }
    }
}

fn can_passively_engage(
    kind: ObjectKind,
    team: TeamType,
    stats: ObjectStats,
    mobile: bool,
    movement: Option<&MovementPath>,
) -> bool {
    if team == TeamType::Null || !stats.can_attack() || stats.destroyed() {
        return false;
    }
    if !matches!(
        kind,
        ObjectKind::Cannon(_) | ObjectKind::Vehicle(_) | ObjectKind::Robot(_)
    ) {
        return false;
    }
    if matches!(kind, ObjectKind::Robot(_)) && mobile && movement.is_some() {
        return false;
    }

    true
}

fn passive_attack_target_choice(
    attacker_ref_id: u32,
    attacker_position: Vec2,
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    snapshots: &[CombatObjectSnapshot],
) -> Option<CombatObjectSnapshot> {
    snapshots
        .iter()
        .copied()
        .filter(|target| {
            target.ref_id != attacker_ref_id
                && passive_engage_target_kind(target.kind)
                && target.team != TeamType::Null
                && can_attack_target_identity(
                    attacker_team,
                    attacker_stats,
                    0,
                    target.team,
                    target.stats,
                )
                && attacker_position.distance(target.position) <= attacker_stats.attack_radius
        })
        .next()
}

fn passive_agro_target_choice(
    attacker_ref_id: u32,
    attacker_position: Vec2,
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    snapshots: &[CombatObjectSnapshot],
) -> Option<CombatObjectSnapshot> {
    snapshots
        .iter()
        .copied()
        .filter(|target| {
            target.ref_id != attacker_ref_id
                && passive_engage_target_kind(target.kind)
                && target.team != TeamType::Null
                && can_attack_target_identity(
                    attacker_team,
                    attacker_stats,
                    0,
                    target.team,
                    target.stats,
                )
                && attacker_position.distance(target.position) > attacker_stats.attack_radius
                && attacker_position.distance(target.position)
                    <= attacker_stats.attack_radius + AGRO_DISTANCE
        })
        .min_by(|a, b| {
            attacker_position
                .distance_squared(a.position)
                .total_cmp(&attacker_position.distance_squared(b.position))
        })
}

fn passive_grenade_pickup_target_choice(
    robot_kind: ObjectKind,
    mobile: bool,
    robot: PassiveAutoEnterRobotSnapshot,
    grenade_amount: Option<u8>,
    grenade_boxes: &[PassiveGrenadeBoxSnapshot],
) -> Option<PassiveGrenadeBoxSnapshot> {
    if !matches!(robot_kind, ObjectKind::Robot(_))
        || !mobile
        || !can_pickup_grenades(robot_kind, grenade_amount.unwrap_or(0))
        || robot.has_waypoint
        || robot.has_attack_target
        || robot.has_task_target
        || robot.is_minion
    {
        return None;
    }

    grenade_boxes
        .iter()
        .copied()
        .filter(|target| robot.position.distance(target.position) <= AUTO_GRAB_VEHICLE_DISTANCE)
        .min_by(|a, b| {
            robot
                .position
                .distance_squared(a.position)
                .total_cmp(&robot.position.distance_squared(b.position))
        })
}

fn passive_auto_enter_target_choice(
    robot_kind: ObjectKind,
    mobile: bool,
    robot: PassiveAutoEnterRobotSnapshot,
    targets: &[PassiveAutoEnterTargetSnapshot],
) -> Option<PassiveAutoEnterTargetSnapshot> {
    if !matches!(robot_kind, ObjectKind::Robot(_))
        || !mobile
        || robot.has_waypoint
        || robot.has_attack_target
        || robot.has_task_target
        || robot.is_minion
    {
        return None;
    }

    targets
        .iter()
        .copied()
        .filter(|target| {
            robot.ref_id != target.ref_id
                && robot.position.distance(target.position) <= AUTO_GRAB_VEHICLE_DISTANCE
                && !(robot.just_left_cannon && matches!(target.kind, ObjectKind::Cannon(_)))
        })
        .min_by(|a, b| {
            robot
                .position
                .distance_squared(a.position)
                .total_cmp(&robot.position.distance_squared(b.position))
        })
}

fn passive_engage_target_kind(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Cannon(_) | ObjectKind::Vehicle(_) | ObjectKind::Robot(_)
    )
}

fn insert_attack_target_for_ref(
    commands: &mut Commands,
    ref_id: u32,
    target_ref_id: u32,
    player_given: bool,
    layers: &[ObjectLayerSnapshot],
) {
    for layer in layers.iter().filter(|layer| layer.ref_id == ref_id) {
        commands.entity(layer.entity).insert(AttackTarget {
            ref_id: target_ref_id,
            cooldown: 0.0,
            player_given,
        });
    }
}

fn remove_attack_target_for_ref(
    commands: &mut Commands,
    ref_id: u32,
    layers: &[ObjectLayerSnapshot],
) {
    for layer in layers.iter().filter(|layer| layer.ref_id == ref_id) {
        commands.entity(layer.entity).remove::<AttackTarget>();
    }
}

fn insert_enter_route_for_ref(
    commands: &mut Commands,
    ref_id: u32,
    base_position: Vec2,
    target: PassiveAutoEnterTargetSnapshot,
    move_speed: f32,
    passability: &PassabilityGrid,
    layers: &[ObjectLayerSnapshot],
) {
    let Some(route) = passability.route(base_position, target.position) else {
        return;
    };

    for layer in layers.iter().filter(|layer| layer.ref_id == ref_id) {
        let layer_offset = layer.position - base_position;
        commands
            .entity(layer.entity)
            .remove::<AttackTarget>()
            .remove::<PickupGrenadesTarget>()
            .remove::<EnterFortTarget>()
            .remove::<CraneRepairTarget>()
            .remove::<UnitRepairTarget>()
            .insert(MovementPath::new(
                route
                    .iter()
                    .map(|waypoint| *waypoint + layer_offset)
                    .collect(),
                move_speed,
            ))
            .insert(EnterTarget {
                ref_id: target.ref_id,
            });
    }
}

fn insert_grenade_pickup_route_for_ref(
    commands: &mut Commands,
    ref_id: u32,
    base_position: Vec2,
    target: PassiveGrenadeBoxSnapshot,
    move_speed: f32,
    passability: &PassabilityGrid,
    layers: &[ObjectLayerSnapshot],
) {
    let Some(route) = passability.route(base_position, target.position) else {
        return;
    };

    for layer in layers.iter().filter(|layer| layer.ref_id == ref_id) {
        let layer_offset = layer.position - base_position;
        commands
            .entity(layer.entity)
            .remove::<AttackTarget>()
            .remove::<EnterTarget>()
            .remove::<EnterFortTarget>()
            .remove::<CraneRepairTarget>()
            .remove::<UnitRepairTarget>()
            .insert(
                MovementPath::new(
                    route
                        .iter()
                        .map(|waypoint| *waypoint + layer_offset)
                        .collect(),
                    move_speed,
                )
                .with_run_attempt(),
            )
            .insert(PickupGrenadesTarget {
                ref_id: target.ref_id,
            });
    }
}

fn insert_movement_route_for_ref(
    commands: &mut Commands,
    ref_id: u32,
    kind: ObjectKind,
    base_position: Vec2,
    target_position: Vec2,
    attack_radius: f32,
    move_speed: f32,
    passability: &PassabilityGrid,
    layers: &[ObjectLayerSnapshot],
) {
    let Some(route) = passability.route_to_attack_range_for_object_kind(
        base_position,
        target_position,
        attack_radius,
        kind,
    ) else {
        return;
    };

    for layer in layers.iter().filter(|layer| layer.ref_id == ref_id) {
        let layer_offset = layer.position - base_position;
        commands.entity(layer.entity).insert(MovementPath::new(
            route
                .iter()
                .map(|waypoint| *waypoint + layer_offset)
                .collect(),
            move_speed,
        ));
    }
}

fn process_building_production(
    mut commands: Commands,
    time: Res<Time>,
    game_atlases: Res<GameAtlases>,
    map: Res<CurrentMap>,
    hud_layout: Res<HudLayout>,
    mut next_ref: ResMut<NextObjectRefId>,
    mut queries: ParamSet<(
        Query<(
            &GameObjectEntity,
            &MapGridPosition,
            &ObjectTeam,
            &ObjectStats,
            Option<&BuildingProduction>,
        )>,
        Query<(
            &GameObjectEntity,
            &MapGridPosition,
            &ObjectTeam,
            &ObjectStats,
            &mut BuildingProduction,
        )>,
    )>,
) {
    let team_unit_counts: HashMap<TeamType, usize> = queries
        .p0()
        .iter()
        .filter(|(object, _, _, stats, _)| {
            !stats.destroyed()
                && matches!(
                    object.kind,
                    ObjectKind::Robot(_) | ObjectKind::Vehicle(_) | ObjectKind::Cannon(_)
                )
        })
        .fold(HashMap::new(), |mut counts, (_, _, team, _, _)| {
            if team.0 != TeamType::Null {
                *counts.entry(team.0).or_insert(0) += 1;
            }
            counts
        });

    let mut cannon_zone_snapshots: Vec<(Option<usize>, usize)> = queries
        .p0()
        .iter()
        .filter_map(|(object, grid, _, _, maybe_production)| {
            let zone = zone_at_tile(&map.0, grid.x, grid.y);
            if matches!(object.kind, ObjectKind::Cannon(_)) {
                return Some((zone, 1));
            }
            let stored = maybe_production
                .map(|production| production.stored_cannons.len())
                .unwrap_or(0);
            (stored > 0).then_some((zone, stored))
        })
        .collect();

    for (object, grid, team, stats, mut production) in &mut queries.p1() {
        if team.0 == TeamType::Null || stats.destroyed() {
            continue;
        }
        let ObjectKind::Building(building) = object.kind else {
            continue;
        };
        let Some((create_center, move_target)) = production_world_points(building, grid.x, grid.y)
        else {
            continue;
        };

        let unit_limit_reached =
            team_unit_limit_reached(team_unit_counts.get(&team.0).copied().unwrap_or(0));
        let completed = advance_production_with_unit_limit(
            &mut production,
            time.delta_secs(),
            *stats,
            unit_limit_reached,
        );
        for unit in completed {
            if is_stored_cannon(unit) {
                let zone = zone_at_tile(&map.0, grid.x, grid.y);
                let cannon_count_in_zone = cannon_zone_snapshots
                    .iter()
                    .filter(|(snapshot_zone, _)| *snapshot_zone == zone)
                    .map(|(_, count)| *count)
                    .sum();
                if can_store_cannon_in_zone(cannon_count_in_zone) {
                    store_built_cannon(&mut production, unit);
                    cannon_zone_snapshots.push((zone, 1));
                }
                continue;
            }

            let count = produced_object_count(unit);
            let group_leader_ref_id =
                (matches!(unit, ObjectKind::Robot(_)) && count > 1).then_some(next_ref.0);
            for _ in 0..count {
                let ref_id = next_ref.0;
                next_ref.0 += 1;
                spawn_runtime_object(
                    &mut commands,
                    &game_atlases,
                    map.0.basics.terrain_type,
                    &hud_layout,
                    ref_id,
                    unit,
                    team.0,
                    create_center,
                    100,
                    true,
                    false,
                    Some(move_target),
                    group_leader_ref_id,
                );
            }
        }
    }
}

fn process_flag_captures(
    mut commands: Commands,
    time: Res<Time>,
    game_atlases: Res<GameAtlases>,
    mut timer: ResMut<FlagCaptureTimer>,
    mut zones: ResMut<ZoneOwnership>,
    mut queries: ParamSet<(
        Query<(
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &mut ObjectTeam,
            &ObjectStats,
            Option<&BuildingLevel>,
            Option<&mut BuildingProduction>,
            Option<&mut Sprite>,
            Option<&mut AtlasAnimation>,
        )>,
        Query<(&MinimapDot, &mut Sprite)>,
        Query<(&MinimapZone, &mut Sprite)>,
    )>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let objects: Vec<(u32, ObjectKind, Vec2, Vec2, TeamType, bool)> = queries
        .p0()
        .iter()
        .map(|(object, transform, selectable, team, stats)| {
            (
                object.ref_id,
                object.kind,
                transform.translation.truncate(),
                selectable.selection_size.max(Vec2::splat(TILE_SIZE)),
                team.0,
                stats.destroyed(),
            )
        })
        .collect();

    let flags: Vec<(u32, Vec2, TeamType)> = objects
        .iter()
        .filter_map(|(ref_id, kind, position, _, team, _)| {
            matches!(kind, ObjectKind::MapItem(id) if *id == ItemType::Flag as u8)
                .then_some((*ref_id, *position, *team))
        })
        .collect();
    let mobiles: Vec<(u32, Vec2, Vec2, TeamType)> = objects
        .iter()
        .filter_map(|(ref_id, kind, position, size, team, destroyed)| {
            (!destroyed
                && *team != TeamType::Null
                && matches!(kind, ObjectKind::Robot(_) | ObjectKind::Vehicle(_)))
            .then_some((*ref_id, *position, *size, *team))
        })
        .collect();

    for (flag_ref_id, flag_position, flag_team) in flags {
        let Some((_, _, _, new_team)) = mobiles.iter().find(|(_, position, size, team)| {
            *team != flag_team
                && rects_overlap(flag_position, Vec2::splat(TILE_SIZE), *position, *size)
        }) else {
            continue;
        };

        award_zone_to_team(
            flag_ref_id,
            *new_team,
            &mut commands,
            &game_atlases,
            &mut zones,
            &mut queries,
        );
    }
}

fn award_zone_to_team(
    flag_ref_id: u32,
    new_team: TeamType,
    commands: &mut Commands,
    game_atlases: &GameAtlases,
    zones: &mut ZoneOwnership,
    queries: &mut ParamSet<(
        Query<(
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &mut ObjectTeam,
            &ObjectStats,
            Option<&BuildingLevel>,
            Option<&mut BuildingProduction>,
            Option<&mut Sprite>,
            Option<&mut AtlasAnimation>,
        )>,
        Query<(&MinimapDot, &mut Sprite)>,
        Query<(&MinimapZone, &mut Sprite)>,
    )>,
) {
    let Some(link) = zones.zone_for_flag(flag_ref_id).cloned() else {
        return;
    };

    if zones.owners.get(link.zone_index).copied() == Some(new_team) {
        return;
    }

    if let Some(owner) = zones.owners.get_mut(link.zone_index) {
        *owner = new_team;
    }

    let linked_building_refs = link.building_refs;
    let mut refs_to_update = linked_building_refs.clone();
    refs_to_update.push(flag_ref_id);

    for (
        entity,
        object,
        mut team,
        stats,
        maybe_level,
        maybe_production,
        maybe_sprite,
        maybe_animation,
    ) in &mut queries.p1()
    {
        let linked_object = refs_to_update.contains(&object.ref_id);

        if linked_object {
            team.0 = new_team;
        }

        if let Some(mut production) = maybe_production {
            reset_build_time(&mut production, *stats, zones.team_zone_ownage(team.0));
            if linked_building_refs.contains(&object.ref_id) {
                if let Some(level) = maybe_level {
                    set_default_production(&mut production, object.kind, level.0, *stats);
                }
            }
        } else if linked_building_refs.contains(&object.ref_id) {
            if let Some(level) = maybe_level {
                let mut production =
                    match initial_production_for_building(object.kind, level.0, new_team, *stats) {
                        Some(production) => production,
                        None => continue,
                    };
                reset_build_time(&mut production, *stats, zones.team_zone_ownage(new_team));
                commands.entity(entity).insert(production);
            }
        }

        if !linked_object {
            continue;
        }

        if object.ref_id == flag_ref_id {
            if let Some(mut animation) = maybe_animation {
                if let Some(indices) = game_atlases.flag_animation_indices(new_team) {
                    animation.frames = indices;
                    animation.current = 0;
                    if let Some(mut sprite) = maybe_sprite {
                        if let Some(index) = animation.frames.first().copied() {
                            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                                atlas.index = index;
                            }
                        }
                    }
                }
            }
        }
    }

    for (dot, mut sprite) in &mut queries.p2() {
        if refs_to_update.contains(&dot.ref_id) {
            sprite.color = new_team.color();
        }
    }
    for (zone, mut sprite) in &mut queries.p3() {
        if zone.zone_index == link.zone_index {
            sprite.color = minimap_zone_color(new_team);
        }
    }
}

fn rects_overlap(a_center: Vec2, a_size: Vec2, b_center: Vec2, b_size: Vec2) -> bool {
    let a_half = a_size * 0.5;
    let b_half = b_size * 0.5;
    let a_min = a_center - a_half;
    let a_max = a_center + a_half;
    let b_min = b_center - b_half;
    let b_max = b_center + b_half;

    !(a_min.x >= b_max.x || a_max.x <= b_min.x || a_min.y >= b_max.y || a_max.y <= b_min.y)
}

fn process_fort_under_attack_warning(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut warning: ResMut<FortUnderAttackWarning>,
    object_query: Query<(&GameObjectEntity, &Transform, &ObjectTeam, &ObjectStats)>,
) {
    warning.verbal_cooldown_remaining =
        (warning.verbal_cooldown_remaining - time.delta_secs()).max(0.0);
    warning.danger_check_elapsed += time.delta_secs();

    let mut should_scan = false;
    while warning.danger_check_elapsed >= FORT_UNDER_ATTACK_SCAN_INTERVAL {
        warning.danger_check_elapsed -= FORT_UNDER_ATTACK_SCAN_INTERVAL;
        should_scan = true;
    }

    if !should_scan {
        return;
    }

    let snapshots: Vec<FortWarningSnapshot> = object_query
        .iter()
        .map(|(object, transform, team, stats)| FortWarningSnapshot {
            ref_id: object.ref_id,
            kind: object.kind,
            position: transform.translation.truncate(),
            team: team.0,
            destroyed: stats.destroyed(),
        })
        .collect();

    warning.danger_fort_ref_id = fort_under_attack_target(&snapshots, TeamType::Red);
    if let Some(fort_ref_id) = warning.danger_fort_ref_id {
        if trigger_fort_under_attack_warning(&mut warning, fort_ref_id) {
            play_game_sound(
                &mut commands,
                &asset_server,
                GameSoundKind::ComputerFortUnderAttack,
                None,
            );
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct TeamStanding {
    units_available: i32,
    zone_percentage: f32,
}

#[derive(Clone, Copy)]
struct LosingWarningSnapshot {
    kind: ObjectKind,
    team: TeamType,
    destroyed: bool,
}

fn process_losing_verbal_warning(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    mut warning: ResMut<LosingVerbalWarning>,
    zones: Res<ZoneOwnership>,
    object_query: Query<(&GameObjectEntity, &ObjectTeam, &ObjectStats)>,
) {
    warning.cooldown_remaining = (warning.cooldown_remaining - time.delta_secs()).max(0.0);
    if warning.cooldown_remaining > 0.0 {
        return;
    }

    let snapshots: Vec<LosingWarningSnapshot> = object_query
        .iter()
        .map(|(object, team, stats)| LosingWarningSnapshot {
            kind: object.kind,
            team: team.0,
            destroyed: stats.destroyed(),
        })
        .collect();
    let standings = losing_warning_team_standings(&snapshots, &zones);

    if trigger_losing_verbal_warning(&mut warning, &standings, TeamType::Red) {
        play_game_sound(
            &mut commands,
            &asset_server,
            GameSoundKind::ComputerYouAreLosing,
            Some(&mut rng),
        );
    }
}

fn trigger_losing_verbal_warning(
    warning: &mut LosingVerbalWarning,
    standings: &[TeamStanding; TEAM_TYPE_COUNT],
    our_team: TeamType,
) -> bool {
    if warning.cooldown_remaining > 0.0 || !losing_warning_should_play(standings, our_team) {
        return false;
    }

    warning.cooldown_remaining = LOSING_VERBAL_WARNING_COOLDOWN;
    true
}

fn losing_warning_team_standings(
    snapshots: &[LosingWarningSnapshot],
    zones: &ZoneOwnership,
) -> [TeamStanding; TEAM_TYPE_COUNT] {
    let mut standings = [TeamStanding::default(); TEAM_TYPE_COUNT];

    for snapshot in snapshots {
        if snapshot.team == TeamType::Null
            || snapshot.destroyed
            || !losing_warning_counts_as_unit(snapshot.kind)
        {
            continue;
        }

        standings[team_index(snapshot.team)].units_available += 1;
    }

    for team in playable_teams() {
        standings[team_index(team)].zone_percentage = zones.team_zone_ownage(team);
    }

    standings
}

fn losing_warning_should_play(
    standings: &[TeamStanding; TEAM_TYPE_COUNT],
    our_team: TeamType,
) -> bool {
    let our = standings[team_index(our_team)];
    if our.units_available <= 0 {
        return false;
    }

    let mut next_worst_unit_count: Option<i32> = None;
    let mut next_worst_zone_percentage: Option<f32> = None;
    for team in playable_teams() {
        if team == our_team {
            continue;
        }

        let standing = standings[team_index(team)];
        if standing.units_available <= 0 {
            continue;
        }

        next_worst_unit_count = Some(
            next_worst_unit_count.map_or(standing.units_available, |current| {
                current.min(standing.units_available)
            }),
        );
        next_worst_zone_percentage = Some(
            next_worst_zone_percentage.map_or(standing.zone_percentage, |current| {
                current.min(standing.zone_percentage)
            }),
        );
    }

    let Some(next_worst_unit_count) = next_worst_unit_count else {
        return false;
    };
    let Some(next_worst_zone_percentage) = next_worst_zone_percentage else {
        return false;
    };

    next_worst_unit_count > losing_scaled_unit_count(our.units_available)
        && next_worst_zone_percentage > our.zone_percentage * LOSING_VERBAL_WARNING_FACTOR
}

fn losing_scaled_unit_count(units: i32) -> i32 {
    (units as f32 * LOSING_VERBAL_WARNING_FACTOR) as i32
}

fn losing_warning_counts_as_unit(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Robot(_) | ObjectKind::Vehicle(_) | ObjectKind::Cannon(_)
    )
}

fn playable_teams() -> [TeamType; 8] {
    [
        TeamType::Red,
        TeamType::Blue,
        TeamType::Green,
        TeamType::Yellow,
        TeamType::Purple,
        TeamType::Teal,
        TeamType::White,
        TeamType::Black,
    ]
}

fn team_index(team: TeamType) -> usize {
    team as i8 as usize
}

fn trigger_fort_under_attack_warning(
    warning: &mut FortUnderAttackWarning,
    fort_ref_id: u32,
) -> bool {
    if warning.verbal_cooldown_remaining > 0.0 {
        return false;
    }

    start_fort_under_attack_message(&mut warning.message, fort_ref_id);
    warning.verbal_cooldown_remaining = FORT_UNDER_ATTACK_VERBAL_COOLDOWN;
    true
}

fn fort_under_attack_target(
    snapshots: &[FortWarningSnapshot],
    player_team: TeamType,
) -> Option<u32> {
    snapshots
        .iter()
        .filter(|fort| fort_warning_player_fort(fort, player_team))
        .filter(|fort| {
            snapshots
                .iter()
                .any(|enemy| fort_warning_enemy_threatens_fort(fort, enemy, player_team))
        })
        .map(|fort| fort.ref_id)
        .min()
}

fn fort_warning_enemy_threatens_fort(
    fort: &FortWarningSnapshot,
    enemy: &FortWarningSnapshot,
    player_team: TeamType,
) -> bool {
    fort_warning_enemy(enemy, player_team)
        && fort.position.distance(enemy.position) <= FORT_UNDER_ATTACK_DISTANCE
}

fn fort_warning_player_fort(snapshot: &FortWarningSnapshot, player_team: TeamType) -> bool {
    snapshot.team == player_team
        && !snapshot.destroyed
        && matches!(
            snapshot.kind,
            ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack)
        )
}

fn fort_warning_enemy(snapshot: &FortWarningSnapshot, player_team: TeamType) -> bool {
    snapshot.team != player_team
        && snapshot.team != TeamType::Null
        && !snapshot.destroyed
        && !matches!(
            snapshot.kind,
            ObjectKind::Building(_) | ObjectKind::MapItem(_)
        )
}

fn process_attack_targets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut queries: ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &Transform,
            &ObjectTeam,
            &ObjectStats,
            Option<&mut GrenadeInventory>,
            Option<&DriverHealth>,
            Option<&Selectable>,
            &mut AttackTarget,
        )>,
        Query<(
            &GameObjectEntity,
            &Transform,
            &ObjectTeam,
            &ObjectStats,
            Option<&DriverHealth>,
            Option<&AttackTarget>,
            Option<&Selectable>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &mut ObjectTeam,
            &mut ObjectStats,
            Option<&mut DriverHealth>,
            Option<&mut DamageCauseTimers>,
        )>,
        Query<(
            Entity,
            &ObjectLayerRef,
            Option<&mut ObjectTeam>,
            Option<&mut Sprite>,
            Option<&mut MobileSpriteLayer>,
        )>,
        Query<(&MinimapDot, &mut Sprite)>,
    )>,
    game_atlases: Res<GameAtlases>,
) {
    let snapshots: Vec<CombatObjectSnapshot> = queries
        .p1()
        .iter()
        .map(
            |(object, transform, team, stats, _, attack_target, selectable)| CombatObjectSnapshot {
                ref_id: object.ref_id,
                kind: object.kind,
                position: transform.translation.truncate(),
                size: selectable
                    .map(|selectable| selectable.selection_size)
                    .unwrap_or_else(|| combat_object_default_size(object.kind)),
                team: team.0,
                stats: *stats,
                attack_target_ref: attack_target.map(|attack_target| attack_target.ref_id),
            },
        )
        .collect();

    let mut pending_damage = Vec::new();
    for (
        entity,
        object,
        transform,
        team,
        stats,
        mut inventory,
        driver,
        selectable,
        mut attack_target,
    ) in &mut queries.p0()
    {
        let effective_stats = effective_attack_stats(object.kind, *stats, driver);
        if stats.destroyed() || !effective_stats.can_attack() {
            commands.entity(entity).remove::<AttackTarget>();
            continue;
        }

        let Some(target) = snapshots
            .iter()
            .find(|snapshot| snapshot.ref_id == attack_target.ref_id)
            .copied()
        else {
            commands.entity(entity).remove::<AttackTarget>();
            continue;
        };

        let attacker_pos = transform.translation.truncate();
        let grenade_amount = grenade_attack_amount(
            object.kind,
            inventory.as_ref().map(|inventory| inventory.amount),
        );
        if !can_attack_target_identity(
            team.0,
            effective_stats,
            grenade_amount,
            target.team,
            target.stats,
        ) {
            commands.entity(entity).remove::<AttackTarget>();
            continue;
        }

        if attacker_pos.distance(target.position) > effective_stats.attack_radius {
            continue;
        }

        attack_target.cooldown -= time.delta_secs();
        if attack_target.cooldown > 0.0 {
            continue;
        }

        let mut attack = attack_delivery(effective_stats, grenade_amount);
        let effective_kind = effective_attack_kind(object.kind, driver);
        let damage_multiplier = driver_attack_damage_multiplier(object.kind, driver);
        attack.damage *= damage_multiplier;
        attack_target.cooldown = attack.cooldown.max(0.02);
        let sound_size = selectable
            .map(|selectable| selectable.selection_size)
            .unwrap_or_else(|| combat_object_default_size(object.kind));
        if let Some(sound) =
            attack_sound_for_attack(object.kind, effective_kind, attack.consumes_grenade)
        {
            play_restricted_game_sound(
                &mut commands,
                &asset_server,
                &windows,
                &camera_query,
                sound,
                sound_source_top_left_map(attacker_pos, sound_size),
                sound_size,
                None,
            );
        }
        if attack.missile_speed > 0.0 || attack.radius > 0.0 {
            let visual = damage_missile_visual_for_attack(effective_kind, attack);
            let frames = damage_missile_frame_handles(&asset_server, visual);
            let scatter = Vec2::new(
                rng.scatter(attack.scatter_half_extent),
                rng.scatter(attack.scatter_half_extent),
            );
            let missile_target = target.position + scatter;
            let missile_start = damage_missile_start_position(visual, attacker_pos, missile_target);
            let total_time = missile_start.distance(missile_target) / attack.missile_speed.max(1.0);
            let visual_geometry =
                damage_missile_visual_geometry(visual, missile_start, missile_target);
            let missile_rotation = damage_missile_rotation(visual, missile_start, missile_target);
            let replica_frames = frames.clone();
            commands.spawn((
                damage_missile_sprite(frames.first().cloned()),
                Transform {
                    translation: Vec3::new(
                        missile_start.x + visual_geometry.primary_offset.x,
                        missile_start.y + visual_geometry.primary_offset.y,
                        35.0,
                    ),
                    rotation: missile_rotation,
                    ..default()
                },
                DamageMissile {
                    start: missile_start,
                    target: missile_target,
                    time_remaining: total_time,
                    total_time,
                    damage: attack.damage,
                    radius: attack.radius.max(1.0),
                    team: team.0,
                    visual,
                    frames,
                    frame_time: DAMAGE_MISSILE_FRAME_TIME,
                    frame_elapsed: 0.0,
                    frame: 0,
                    visual_rise: 0.0,
                    angle_degrees_per_sec: 0.0,
                    crater: damage_crater_for_attack(effective_kind, attack),
                    visual_offset: visual_geometry.primary_offset,
                    smoke_offsets: visual_geometry.smoke_offsets.clone(),
                    smoke_time_cursor: 0.0,
                },
                Name::new("damage_missile"),
            ));
            spawn_damage_missile_launch_effect(
                &mut commands,
                &asset_server,
                &mut rng,
                visual,
                missile_start,
            );
            spawn_damage_missile_replicas(
                &mut commands,
                replica_frames,
                missile_start,
                missile_target,
                total_time,
                visual_geometry.replica_offsets,
            );
            if attack.consumes_grenade {
                if let Some(inventory) = inventory.as_mut() {
                    inventory.amount = inventory.amount.saturating_sub(1);
                }
            }
        } else {
            if let Some(projectile) = special_projectile_kind_for_attack(effective_kind) {
                spawn_special_projectile_effect(
                    &mut commands,
                    &asset_server,
                    &mut rng,
                    attacker_pos,
                    target.position,
                    projectile,
                );
            } else if uses_direct_fire_bullet(object.kind) {
                spawn_direct_fire_bullet(&mut commands, attacker_pos, target.position, team.0);
            }
            pending_damage.push(PendingAttackDamage {
                target_ref_id: target.ref_id,
                attacker_kind: effective_kind,
                attacker_stats: effective_stats,
                target_can_be_sniped: target_can_be_sniped(target, &snapshots),
                damage: attack.damage,
                damage_chance: attack.damage_chance,
            });
        }

        if attack_target.player_given && object.ref_id == target.ref_id {
            commands.entity(entity).remove::<AttackTarget>();
        }
    }

    for attack in pending_damage {
        if rng.next_roll() > attack.damage_chance {
            continue;
        }

        let mut neutralize_target = false;
        let mut driver_was_hit = false;
        {
            let mut target_query = queries.p2();
            let Some((
                target_entity,
                target_object,
                _,
                mut target_stats,
                target_driver,
                damage_cause,
            )) = target_query
                .iter_mut()
                .find(|(_, object, _, _, _, _)| object.ref_id == attack.target_ref_id)
            else {
                continue;
            };

            if let Some(mut driver) = target_driver {
                if should_snipe_driver(
                    attack.attacker_kind,
                    attack.attacker_stats,
                    target_object.kind,
                    driver.as_ref(),
                    attack.target_can_be_sniped,
                    rng.next_roll(),
                ) {
                    if let Some(lead_health) = driver.driver_healths.first_mut() {
                        *lead_health -= attack.damage;
                    }
                    driver_was_hit = true;
                    neutralize_target = driver.lead_health() <= 0.0;
                }
            }

            if !driver_was_hit {
                target_stats.health =
                    (target_stats.health - attack.damage).clamp(0.0, target_stats.max_health);
                if matches!(attack.attacker_kind, ObjectKind::Robot(RobotType::Pyro)) {
                    mark_fire_damage(&mut commands, target_entity, damage_cause);
                }
            }
        };

        if neutralize_target {
            neutralize_driverless_object(
                &mut commands,
                &mut queries,
                &game_atlases,
                attack.target_ref_id,
            );
        }
    }
}

#[allow(dead_code)]
fn can_attack_target(
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    target_team: TeamType,
    target_stats: ObjectStats,
    distance: f32,
) -> bool {
    can_attack_target_with_grenades(
        attacker_team,
        attacker_stats,
        0,
        target_team,
        target_stats,
        distance,
    )
}

fn can_attack_target_with_grenades(
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    attacker_grenade_amount: u8,
    target_team: TeamType,
    target_stats: ObjectStats,
    distance: f32,
) -> bool {
    if !can_attack_target_identity(
        attacker_team,
        attacker_stats,
        attacker_grenade_amount,
        target_team,
        target_stats,
    ) {
        return false;
    }

    distance <= attacker_stats.attack_radius
}

fn can_attack_target_identity(
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    attacker_grenade_amount: u8,
    target_team: TeamType,
    target_stats: ObjectStats,
) -> bool {
    if !attacker_stats.can_attack() || target_stats.destroyed() {
        return false;
    }
    if attacker_team == target_team {
        return false;
    }
    if target_stats.attacked_only_by_explosives
        && !attacker_has_explosives(attacker_stats, attacker_grenade_amount)
    {
        return false;
    }

    true
}

fn should_snipe_driver(
    attacker_kind: ObjectKind,
    attacker_stats: ObjectStats,
    target_kind: ObjectKind,
    driver: &DriverHealth,
    target_can_be_sniped: bool,
    roll: f32,
) -> bool {
    can_snipe(attacker_kind, attacker_stats)
        && can_be_sniped(target_kind, driver, target_can_be_sniped)
        && roll <= attacker_stats.snipe_chance
}

fn can_snipe(kind: ObjectKind, stats: ObjectStats) -> bool {
    stats.snipe_chance > 0.0 && can_snipe_flag(kind)
}

fn can_snipe_flag(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Robot(RobotType::Grunt)
            | ObjectKind::Robot(RobotType::Psycho)
            | ObjectKind::Robot(RobotType::Sniper)
            | ObjectKind::Robot(RobotType::Pyro)
            | ObjectKind::Robot(RobotType::Laser)
            | ObjectKind::Vehicle(VehicleType::Jeep)
            | ObjectKind::Cannon(CannonType::Gatling)
    )
}

fn target_can_be_sniped(target: CombatObjectSnapshot, snapshots: &[CombatObjectSnapshot]) -> bool {
    match target.kind {
        ObjectKind::Cannon(_) | ObjectKind::Vehicle(VehicleType::Jeep) => true,
        ObjectKind::Vehicle(VehicleType::Light)
        | ObjectKind::Vehicle(VehicleType::Medium)
        | ObjectKind::Vehicle(VehicleType::Heavy) => target
            .attack_target_ref
            .and_then(|attack_ref| {
                snapshots
                    .iter()
                    .find(|snapshot| snapshot.ref_id == attack_ref)
            })
            .is_some_and(|attack_target| can_snipe_flag(attack_target.kind)),
        _ => false,
    }
}

fn can_be_sniped(kind: ObjectKind, driver: &DriverHealth, target_can_be_sniped: bool) -> bool {
    driver.lead_health() > 0.0
        && target_can_be_sniped
        && matches!(
            kind,
            ObjectKind::Cannon(_)
                | ObjectKind::Vehicle(VehicleType::Jeep)
                | ObjectKind::Vehicle(VehicleType::Light)
                | ObjectKind::Vehicle(VehicleType::Medium)
                | ObjectKind::Vehicle(VehicleType::Heavy)
        )
}

fn neutralize_driverless_object(
    commands: &mut Commands,
    queries: &mut ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &Transform,
            &ObjectTeam,
            &ObjectStats,
            Option<&mut GrenadeInventory>,
            Option<&DriverHealth>,
            Option<&Selectable>,
            &mut AttackTarget,
        )>,
        Query<(
            &GameObjectEntity,
            &Transform,
            &ObjectTeam,
            &ObjectStats,
            Option<&DriverHealth>,
            Option<&AttackTarget>,
            Option<&Selectable>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &mut ObjectTeam,
            &mut ObjectStats,
            Option<&mut DriverHealth>,
            Option<&mut DamageCauseTimers>,
        )>,
        Query<(
            Entity,
            &ObjectLayerRef,
            Option<&mut ObjectTeam>,
            Option<&mut Sprite>,
            Option<&mut MobileSpriteLayer>,
        )>,
        Query<(&MinimapDot, &mut Sprite)>,
    )>,
    game_atlases: &GameAtlases,
    ref_id: u32,
) {
    let target_kind = {
        let mut targets = queries.p2();
        let Some((entity, object, mut team, _, _, _)) = targets
            .iter_mut()
            .find(|(_, object, _, _, _, _)| object.ref_id == ref_id)
        else {
            return;
        };
        team.0 = TeamType::Null;
        commands
            .entity(entity)
            .remove::<DriverHealth>()
            .remove::<AttackTarget>()
            .remove::<MovementPath>()
            .remove::<PickupGrenadesTarget>()
            .remove::<EnterTarget>()
            .remove::<EnterFortTarget>()
            .remove::<CraneRepairTarget>()
            .remove::<UnitRepairTarget>();
        object.kind
    };

    for (entity, layer_ref, maybe_team, maybe_sprite, maybe_mobile) in &mut queries.p3() {
        if layer_ref.0 != ref_id {
            continue;
        }
        if let Some(mut team) = maybe_team {
            team.0 = TeamType::Null;
        }
        commands
            .entity(entity)
            .remove::<AttackTarget>()
            .remove::<MovementPath>()
            .remove::<PickupGrenadesTarget>()
            .remove::<EnterTarget>()
            .remove::<EnterFortTarget>()
            .remove::<CraneRepairTarget>()
            .remove::<UnitRepairTarget>();

        if let Some(mut mobile) = maybe_mobile {
            mobile.team = TeamType::Null;
            if let Some(mut sprite) = maybe_sprite {
                if let Some(frame) = game_atlases.mobile_frame(
                    mobile.kind,
                    TeamType::Null,
                    mobile.role,
                    mobile.rotation,
                    mobile.frame,
                    false,
                ) {
                    apply_sprite_frame(&mut sprite, frame);
                }
            }
            continue;
        }

        if let (ObjectKind::Cannon(cannon), Some(mut sprite)) = (target_kind, maybe_sprite) {
            if let Some(frame) = game_atlases.captured_cannon_frame(cannon, TeamType::Null, 180) {
                apply_sprite_frame(&mut sprite, frame);
            }
        }
    }

    for (dot, mut sprite) in &mut queries.p4() {
        if dot.ref_id == ref_id {
            sprite.color = TeamType::Null.color();
        }
    }
}

fn process_eject_driver_commands(
    mut commands: Commands,
    mut next_ref: ResMut<NextObjectRefId>,
    map: Res<CurrentMap>,
    hud_layout: Res<HudLayout>,
    game_atlases: Res<GameAtlases>,
    mut selection: ResMut<SelectionState>,
    mut queries: ParamSet<(
        Query<
            (
                Entity,
                &GameObjectEntity,
                &Transform,
                &ObjectTeam,
                &mut ObjectStats,
                Option<&DriverHealth>,
            ),
            (With<EjectDriversCommand>, Without<DestroyedObject>),
        >,
        Query<(
            Entity,
            &ObjectLayerRef,
            Option<&mut ObjectTeam>,
            Option<&mut Sprite>,
            Option<&mut MobileSpriteLayer>,
        )>,
        Query<(&MinimapDot, &mut Sprite)>,
    )>,
) {
    let mut ejected_selection: Option<u32> = None;
    let mut neutralized_refs = Vec::new();

    for (entity, object, transform, team, mut stats, driver) in &mut queries.p0() {
        if !can_eject_driver_object(object.kind, team.0, *stats, driver) {
            commands.entity(entity).remove::<EjectDriversCommand>();
            continue;
        }

        let spawn_center = transform.translation.truncate();
        if let Some(driver) = driver {
            let driver_kind = driver.driver_kind;
            let driver_healths = driver.driver_healths.clone();
            let first_ref_id = next_ref.0;
            let group_leader_ref_id = (driver_healths.len() > 1).then_some(first_ref_id);

            for health in &driver_healths {
                let ref_id = next_ref.0;
                next_ref.0 += 1;
                spawn_runtime_object(
                    &mut commands,
                    &game_atlases,
                    map.0.basics.terrain_type,
                    &hud_layout,
                    ref_id,
                    ObjectKind::Robot(driver_kind),
                    team.0,
                    spawn_center,
                    driver_health_percent(ObjectKind::Robot(driver_kind), *health),
                    true,
                    matches!(object.kind, ObjectKind::Cannon(_)),
                    None,
                    group_leader_ref_id,
                );
            }

            if selection.selected_refs.contains(&object.ref_id) && !driver_healths.is_empty() {
                ejected_selection = Some(first_ref_id);
            }
        }

        clear_driver_attack_stats(object.kind, &mut stats);
        commands
            .entity(entity)
            .remove::<DriverHealth>()
            .remove::<EjectDriversCommand>()
            .remove::<AttackTarget>()
            .remove::<MovementPath>()
            .remove::<PickupGrenadesTarget>()
            .remove::<EnterTarget>()
            .remove::<EnterFortTarget>()
            .remove::<CraneRepairTarget>()
            .remove::<UnitRepairTarget>();
        neutralized_refs.push((object.ref_id, object.kind));
    }

    for (neutralized_ref, neutralized_kind) in neutralized_refs {
        for (layer_entity, layer_ref, maybe_team, maybe_sprite, maybe_mobile) in &mut queries.p1() {
            if layer_ref.0 != neutralized_ref {
                continue;
            }

            if let Some(mut layer_team) = maybe_team {
                layer_team.0 = TeamType::Null;
            }
            commands
                .entity(layer_entity)
                .remove::<AttackTarget>()
                .remove::<MovementPath>()
                .remove::<PickupGrenadesTarget>()
                .remove::<EnterTarget>()
                .remove::<EnterFortTarget>()
                .remove::<CraneRepairTarget>()
                .remove::<UnitRepairTarget>();

            if let Some(mut mobile) = maybe_mobile {
                mobile.team = TeamType::Null;
                if let Some(mut sprite) = maybe_sprite {
                    if let Some(frame) = game_atlases.mobile_frame(
                        mobile.kind,
                        TeamType::Null,
                        mobile.role,
                        mobile.rotation,
                        mobile.frame,
                        false,
                    ) {
                        apply_sprite_frame(&mut sprite, frame);
                    }
                }
            } else if let (ObjectKind::Cannon(cannon), Some(mut sprite)) =
                (neutralized_kind, maybe_sprite)
            {
                if let Some(frame) = game_atlases.captured_cannon_frame(cannon, TeamType::Null, 180)
                {
                    apply_sprite_frame(&mut sprite, frame);
                }
            }
        }

        for (dot, mut sprite) in &mut queries.p2() {
            if dot.ref_id == neutralized_ref {
                sprite.color = TeamType::Null.color();
            }
        }
    }

    if let Some(ref_id) = ejected_selection {
        selection.selected_refs.clear();
        selection.selected_refs.push(ref_id);
    }
}

fn can_eject_driver_object(
    kind: ObjectKind,
    team: TeamType,
    stats: ObjectStats,
    _driver: Option<&DriverHealth>,
) -> bool {
    team != TeamType::Null && can_eject_drivers(kind, stats)
}

fn driver_health_percent(kind: ObjectKind, health: f32) -> i32 {
    let max_health = ObjectStats::from_kind(kind, 100).max_health.max(1.0);
    ((health / max_health) * 100.0).round().clamp(0.0, 100.0) as i32
}

fn clear_driver_attack_stats(kind: ObjectKind, stats: &mut ObjectStats) {
    let base = ObjectStats::from_kind(kind, 100);
    stats.attack_radius = base.attack_radius;
    stats.attack_damage = base.attack_damage;
    stats.damage_chance = base.damage_chance;
    stats.damage_radius = base.damage_radius;
    stats.missile_speed = base.missile_speed;
    stats.attack_speed = base.attack_speed;
    stats.snipe_chance = base.snipe_chance;
}

fn apply_sprite_frame(sprite: &mut Sprite, frame: crate::render::atlas::SpriteFrame) {
    sprite.image = frame.image;
    sprite.texture_atlas = Some(TextureAtlas {
        layout: frame.layout,
        index: frame.index,
    });
    sprite.rect = None;
    sprite.custom_size = None;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AttackDelivery {
    damage: f32,
    damage_chance: f32,
    radius: f32,
    missile_speed: f32,
    cooldown: f32,
    scatter_half_extent: f32,
    consumes_grenade: bool,
}

fn grenade_attack_amount(kind: ObjectKind, amount: Option<u8>) -> u8 {
    if can_have_grenades(kind) {
        amount.unwrap_or(0)
    } else {
        0
    }
}

fn attacker_has_explosives(stats: ObjectStats, grenade_amount: u8) -> bool {
    stats.has_explosive_damage() || grenade_amount > 0
}

fn attack_delivery(stats: ObjectStats, grenade_amount: u8) -> AttackDelivery {
    if grenade_amount > 0 {
        return AttackDelivery {
            damage: GRENADE_DAMAGE,
            damage_chance: 0.0,
            radius: GRENADE_DAMAGE_RADIUS,
            missile_speed: GRENADE_MISSILE_SPEED,
            cooldown: GRENADE_ATTACK_SPEED,
            scatter_half_extent: GRENADE_SCATTER_HALF_EXTENT,
            consumes_grenade: true,
        };
    }

    AttackDelivery {
        damage: stats.attack_damage,
        damage_chance: stats.damage_chance,
        radius: stats.damage_radius,
        missile_speed: stats.missile_speed,
        cooldown: stats.attack_speed,
        scatter_half_extent: 16.0,
        consumes_grenade: false,
    }
}

fn effective_attack_stats(
    kind: ObjectKind,
    mut stats: ObjectStats,
    driver: Option<&DriverHealth>,
) -> ObjectStats {
    if let (ObjectKind::Vehicle(VehicleType::Apc), Some(driver)) = (kind, driver) {
        let driver_stats = ObjectStats::from_kind(ObjectKind::Robot(driver.driver_kind), 100);
        stats.attack_radius = driver_stats.attack_radius;
        stats.attack_damage = driver_stats.attack_damage;
        stats.damage_chance = driver_stats.damage_chance;
        stats.damage_radius = driver_stats.damage_radius;
        stats.missile_speed = driver_stats.missile_speed;
        stats.attack_speed = driver_stats.attack_speed;
        stats.snipe_chance = driver_stats.snipe_chance;
    }
    stats
}

fn effective_attack_kind(kind: ObjectKind, driver: Option<&DriverHealth>) -> ObjectKind {
    if matches!(kind, ObjectKind::Vehicle(VehicleType::Apc))
        && let Some(driver) = driver
    {
        return ObjectKind::Robot(driver.driver_kind);
    }
    kind
}

fn driver_attack_damage_multiplier(kind: ObjectKind, driver: Option<&DriverHealth>) -> f32 {
    if matches!(kind, ObjectKind::Vehicle(VehicleType::Apc)) {
        driver.map_or(1.0, |driver| driver.driver_count().max(1) as f32)
    } else {
        1.0
    }
}

#[cfg(test)]
fn attack_sound_for_kind(kind: ObjectKind, consumes_grenade: bool) -> Option<GameSoundKind> {
    units::attack_sound_for_kind(kind, consumes_grenade).map(GameSoundKind::from)
}

fn attack_sound_for_attack(
    source_kind: ObjectKind,
    effective_kind: ObjectKind,
    consumes_grenade: bool,
) -> Option<GameSoundKind> {
    units::attack_sound_for_attack(source_kind, effective_kind, consumes_grenade)
        .map(GameSoundKind::from)
}

fn sound_source_top_left_map(center_world: Vec2, size: Vec2) -> Vec2 {
    world_to_map_point(center_world) - size * 0.5
}

fn process_direct_fire_bullets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut bullets: Query<(Entity, &mut Transform, &mut DirectFireBullet), Without<MainCamera>>,
) {
    for (entity, mut transform, mut bullet) in &mut bullets {
        bullet.time_remaining -= time.delta_secs();
        let progress = direct_fire_bullet_progress(&bullet);
        let position = bullet.start.lerp(bullet.target, progress);
        transform.translation.x = position.x;
        transform.translation.y = position.y;

        if bullet.time_remaining <= 0.0 {
            for _ in 0..direct_fire_bullet_ricochet_particle_count(&mut rng) {
                spawn_unit_particle_effect(
                    &mut commands,
                    &asset_server,
                    &mut rng,
                    world_to_map_point(bullet.target),
                    25.0,
                    25.0,
                );
            }
            play_restricted_game_sound(
                &mut commands,
                &asset_server,
                &windows,
                &camera_query,
                GameSoundKind::Ricochet,
                world_to_map_point(bullet.target),
                Vec2::ZERO,
                None,
            );
            commands.entity(entity).despawn();
        }
    }
}

fn process_special_projectile_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    mut projectiles: Query<(
        Entity,
        &mut Transform,
        &mut Sprite,
        &mut SpecialProjectileEffect,
    )>,
) {
    for (entity, mut transform, mut sprite, mut projectile) in &mut projectiles {
        projectile.time_remaining -= time.delta_secs();
        animate_special_projectile_sprite(
            &mut sprite,
            &mut projectile,
            time.delta_secs(),
            &mut rng,
        );
        let progress = special_projectile_progress(&projectile);
        let position = projectile.start.lerp(projectile.target, progress);
        transform.translation.x = position.x;
        transform.translation.y = position.y;

        if projectile.time_remaining > 0.0 {
            continue;
        }

        if projectile.kind == SpecialProjectileKind::Flame {
            spawn_pyro_fire_effect(&mut commands, &asset_server, &mut rng, projectile.target);
        }
        commands.entity(entity).despawn();
    }
}

fn tick_damage_cause_timers(time: Res<Time>, mut query: Query<&mut DamageCauseTimers>) {
    let delta = time.delta_secs();
    for mut timers in &mut query {
        timers.fire = (timers.fire - delta).max(0.0);
        timers.missile = (timers.missile - delta).max(0.0);
    }
}

fn mark_fire_damage(
    commands: &mut Commands,
    entity: Entity,
    timers: Option<Mut<DamageCauseTimers>>,
) {
    mark_damage_cause(commands, entity, timers, true);
}

fn mark_missile_damage(
    commands: &mut Commands,
    entity: Entity,
    timers: Option<Mut<DamageCauseTimers>>,
) {
    mark_damage_cause(commands, entity, timers, false);
}

fn mark_damage_cause(
    commands: &mut Commands,
    entity: Entity,
    timers: Option<Mut<DamageCauseTimers>>,
    fire: bool,
) {
    if let Some(mut timers) = timers {
        if fire {
            timers.fire = DAMAGE_DEATH_CAUSE_WINDOW;
        } else {
            timers.missile = DAMAGE_DEATH_CAUSE_WINDOW;
        }
        return;
    }

    let mut timers = DamageCauseTimers::default();
    if fire {
        timers.fire = DAMAGE_DEATH_CAUSE_WINDOW;
    } else {
        timers.missile = DAMAGE_DEATH_CAUSE_WINDOW;
    }
    commands.entity(entity).insert(timers);
}

fn spawn_direct_fire_bullet(commands: &mut Commands, start: Vec2, target: Vec2, team: TeamType) {
    let duration = direct_fire_bullet_duration(start, target);
    commands.spawn((
        Sprite::from_color(team.color(), Vec2::splat(2.0)),
        Transform::from_xyz(start.x, start.y, 38.0),
        DirectFireBullet {
            start,
            target,
            time_remaining: duration,
            total_time: duration,
        },
        Name::new("direct_fire_bullet"),
    ));
}

fn spawn_special_projectile_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    start: Vec2,
    target: Vec2,
    kind: SpecialProjectileKind,
) {
    let frame_paths = special_projectile_frame_paths(kind);
    let frames: Vec<_> = frame_paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let duration = special_projectile_duration(start, target);
    let current = rng.index(frames.len());
    let image = frames.get(current).cloned().unwrap_or(first);
    commands.spawn((
        Sprite::from_image(image),
        Transform {
            translation: Vec3::new(start.x, start.y, 38.0),
            rotation: special_projectile_rotation(start, target),
            ..default()
        },
        SpecialProjectileEffect {
            kind,
            start,
            target,
            time_remaining: duration,
            total_time: duration,
            frames,
            frame_time: SPECIAL_PROJECTILE_FRAME_TIME,
            frame_elapsed: 0.0,
            current,
        },
        Name::new(match kind {
            SpecialProjectileKind::Laser => "laser_projectile",
            SpecialProjectileKind::Flame => "pyro_flame_projectile",
        }),
    ));
}

fn animate_special_projectile_sprite(
    sprite: &mut Sprite,
    projectile: &mut SpecialProjectileEffect,
    delta_secs: f32,
    rng: &mut CombatRng,
) {
    if projectile.frames.len() <= 1 {
        return;
    }

    projectile.frame_elapsed += delta_secs;
    while projectile.frame_elapsed >= projectile.frame_time {
        projectile.frame_elapsed -= projectile.frame_time;
        projectile.current = match projectile.kind {
            SpecialProjectileKind::Laser => projectile.current,
            SpecialProjectileKind::Flame => rng.index(projectile.frames.len()),
        };
    }

    if let Some(frame) = projectile.frames.get(projectile.current) {
        sprite.image = frame.clone();
    }
}

fn direct_fire_bullet_duration(start: Vec2, target: Vec2) -> f32 {
    (start.distance(target) / DIRECT_FIRE_BULLET_SPEED).max(0.02)
}

fn special_projectile_duration(start: Vec2, target: Vec2) -> f32 {
    (start.distance(target) / SPECIAL_PROJECTILE_SPEED).max(0.02)
}

fn direct_fire_bullet_progress(bullet: &DirectFireBullet) -> f32 {
    if bullet.total_time <= 0.0 {
        1.0
    } else {
        (1.0 - bullet.time_remaining / bullet.total_time).clamp(0.0, 1.0)
    }
}

fn direct_fire_bullet_ricochet_particle_count(rng: &mut CombatRng) -> usize {
    rng.index(3)
}

fn special_projectile_progress(projectile: &SpecialProjectileEffect) -> f32 {
    if projectile.total_time <= 0.0 {
        1.0
    } else {
        (1.0 - projectile.time_remaining / projectile.total_time).clamp(0.0, 1.0)
    }
}

fn special_projectile_kind_for_attack(kind: ObjectKind) -> Option<SpecialProjectileKind> {
    units::special_projectile_kind_for_attack(kind)
}

fn special_projectile_frame_paths(kind: SpecialProjectileKind) -> Vec<String> {
    units::robots::special_projectile_frame_paths(kind)
}

fn special_projectile_rotation(start: Vec2, target: Vec2) -> Quat {
    let delta = target - start;
    if delta.length_squared() <= f32::EPSILON {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_z(delta.y.atan2(delta.x))
    }
}

fn uses_direct_fire_bullet(kind: ObjectKind) -> bool {
    !matches!(kind, ObjectKind::Robot(RobotType::Laser | RobotType::Pyro))
}

fn damage_missile_visual_for_attack(
    attacker: ObjectKind,
    delivery: AttackDelivery,
) -> DamageMissileVisual {
    units::damage_missile_visual_for_attack(attacker, delivery.consumes_grenade)
}

fn damage_crater_for_attack(
    attacker: ObjectKind,
    delivery: AttackDelivery,
) -> Option<DamageCrater> {
    units::damage_crater_for_attack(attacker, delivery.consumes_grenade)
}

#[cfg(test)]
fn light_rocket_big_crater_chance(extra_large: u8, xx_large: u8) -> Option<f32> {
    units::light_rocket_big_crater_chance(extra_large, xx_large)
}

fn damage_missile_frame_handles(
    asset_server: &AssetServer,
    visual: DamageMissileVisual,
) -> Vec<Handle<Image>> {
    damage_missile_frame_paths(visual)
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect()
}

fn damage_missile_frame_paths(visual: DamageMissileVisual) -> Vec<String> {
    units::damage_missile_frame_paths(visual)
}

fn damage_missile_visual_geometry(
    visual: DamageMissileVisual,
    start: Vec2,
    target: Vec2,
) -> DamageMissileVisualGeometry {
    units::damage_missile_visual_geometry(visual, start, target)
}

fn damage_missile_start_position(
    visual: DamageMissileVisual,
    attacker_center: Vec2,
    target: Vec2,
) -> Vec2 {
    let Some(direction) = direction_index_from_delta(target - attacker_center) else {
        return attacker_center;
    };
    attacker_center + units::damage_missile_muzzle_offset(visual, direction).unwrap_or(Vec2::ZERO)
}

#[cfg(test)]
fn vehicle_rocket_muzzle_offset(direction: usize) -> Vec2 {
    units::vehicle_rocket_muzzle_offset(direction)
}

#[cfg(test)]
fn tough_rocket_muzzle_offset(direction: usize) -> Vec2 {
    units::tough_rocket_muzzle_offset(direction)
}

fn damage_missile_rotation(visual: DamageMissileVisual, start: Vec2, target: Vec2) -> Quat {
    match visual {
        DamageMissileVisual::ToughRocket
        | DamageMissileVisual::LightRocket { .. }
        | DamageMissileVisual::MissileCannon
        | DamageMissileVisual::MissileLauncher => special_projectile_rotation(start, target),
        _ => Quat::IDENTITY,
    }
}

fn light_rocket_init_fire_frame_path(frame: usize) -> String {
    units::light_rocket_init_fire_frame_path(frame)
}

fn damage_missile_sprite(first_frame: Option<Handle<Image>>) -> Sprite {
    first_frame.map_or_else(
        || Sprite::from_color(Color::srgba(1.0, 0.55, 0.1, 0.75), Vec2::splat(4.0)),
        Sprite::from_image,
    )
}

fn spawn_damage_missile_launch_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    visual: DamageMissileVisual,
    world_position: Vec2,
) {
    if !matches!(visual, DamageMissileVisual::LightRocket { .. }) {
        return;
    }

    let frame = asset_server.load(light_rocket_init_fire_frame_path(rng.index(4)));
    let map_top_left = world_to_map_point(world_position) + Vec2::new(-8.0, -7.0);
    let world_top_left = map_top_left_to_world(map_top_left);
    commands.spawn((
        Sprite::from_image(frame.clone()),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(world_top_left.x, world_top_left.y, 34.8),
        ImageEffectAnimation {
            frames: vec![frame],
            frame_time: LIGHT_ROCKET_INIT_FIRE_FRAME_TIME,
            elapsed: 0.0,
            current: 0,
            remaining_advances: Some(1),
        },
        Name::new("light_rocket_init_fire"),
    ));
}

fn spawn_damage_missile_replicas(
    commands: &mut Commands,
    frames: Vec<Handle<Image>>,
    start: Vec2,
    target: Vec2,
    total_time: f32,
    offsets: Vec<Vec2>,
) {
    let Some(first_frame) = frames.first().cloned() else {
        return;
    };
    let rotation = special_projectile_rotation(start, target);
    for offset in offsets {
        let position = start + offset;
        commands.spawn((
            Sprite::from_image(first_frame.clone()),
            Transform {
                translation: Vec3::new(position.x, position.y, 34.95),
                rotation,
                ..default()
            },
            DamageMissileReplica {
                start,
                target,
                time_remaining: total_time,
                total_time,
                offset,
                frames: frames.clone(),
                frame_time: DAMAGE_MISSILE_FRAME_TIME,
                frame_elapsed: 0.0,
                frame: 0,
            },
            Name::new("damage_missile_replica"),
        ));
    }
}

fn animate_damage_missile_sprite(
    sprite: &mut Sprite,
    missile: &mut DamageMissile,
    delta_secs: f32,
) {
    if missile.frames.len() <= 1 {
        return;
    }

    missile.frame_elapsed += delta_secs;
    while missile.frame_elapsed >= missile.frame_time {
        missile.frame_elapsed -= missile.frame_time;
        missile.frame = (missile.frame + 1) % missile.frames.len();
    }

    if let Some(frame) = missile.frames.get(missile.frame) {
        sprite.image = frame.clone();
    }
}

fn spawn_damage_missile_smoke_trails(
    commands: &mut Commands,
    asset_server: &AssetServer,
    missile: &mut DamageMissile,
    elapsed: f32,
) {
    if missile.smoke_offsets.is_empty() {
        return;
    }

    for world_position in robots::tough_rocket_smoke_positions(
        missile.start,
        missile.target,
        missile.total_time,
        &mut missile.smoke_time_cursor,
        elapsed,
        &missile.smoke_offsets,
    ) {
        spawn_tough_smoke_effect(commands, asset_server, world_to_map_point(world_position));
    }
}

#[derive(Clone, Copy)]
struct PendingExplosion {
    position: Vec2,
    damage: f32,
    radius: f32,
    team: TeamType,
    visual: DamageMissileVisual,
    crater: Option<DamageCrater>,
}

fn process_damage_missiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map: Res<CurrentMap>,
    tile_info: Res<CurrentTileInfo>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    mut crater_registry: ResMut<CraterStampRegistry>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut queries: ParamSet<(
        Query<(Entity, &mut Transform, &mut Sprite, &mut DamageMissile), Without<MainCamera>>,
        Query<(
            &GameObjectEntity,
            &Transform,
            &ObjectTeam,
            &ObjectStats,
            Option<&Selectable>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &mut ObjectStats,
            Option<&mut DamageCauseTimers>,
        )>,
    )>,
) {
    let object_snapshots: Vec<CombatObjectSnapshot> = queries
        .p1()
        .iter()
        .map(
            |(object, transform, team, stats, selectable)| CombatObjectSnapshot {
                ref_id: object.ref_id,
                kind: object.kind,
                position: transform.translation.truncate(),
                size: selectable
                    .map(|selectable| selectable.selection_size)
                    .unwrap_or_else(|| combat_object_default_size(object.kind)),
                team: team.0,
                stats: *stats,
                attack_target_ref: None,
            },
        )
        .collect();

    let mut explosions = Vec::new();
    for (entity, mut transform, mut sprite, mut missile) in &mut queries.p0() {
        missile.time_remaining -= time.delta_secs();
        animate_damage_missile_sprite(&mut sprite, &mut missile, time.delta_secs());
        let progress = if missile.total_time <= 0.0 {
            1.0
        } else {
            (1.0 - missile.time_remaining / missile.total_time).clamp(0.0, 1.0)
        };
        let position = missile.start.lerp(missile.target, progress);
        let visual_position = position + missile.visual_offset;
        if matches!(missile.visual, DamageMissileVisual::MapObjectTurrent(_)) {
            let elapsed = (missile.total_time - missile.time_remaining).max(0.0);
            let arc =
                map_object::turrent_arc_size(missile.visual_rise, missile.total_time, elapsed);
            transform.translation.x = visual_position.x;
            transform.translation.y = visual_position.y + arc * 30.0;
            transform.scale = Vec3::splat((1.0 + arc).max(0.1));
            transform.rotation =
                Quat::from_rotation_z((missile.angle_degrees_per_sec * elapsed).to_radians());
        } else {
            transform.translation.x = visual_position.x;
            transform.translation.y = visual_position.y;
        }

        if missile.time_remaining > 0.0 {
            let elapsed = missile.total_time - missile.time_remaining;
            spawn_damage_missile_smoke_trails(&mut commands, &asset_server, &mut missile, elapsed);
            continue;
        }

        explosions.push(PendingExplosion {
            position: missile.target,
            damage: missile.damage,
            radius: missile.radius,
            team: missile.team,
            visual: missile.visual,
            crater: missile.crater,
        });
        commands.entity(entity).despawn();
    }

    for explosion in explosions {
        if let Some(sound) = damage_missile_impact_sound(explosion.visual) {
            play_restricted_game_sound(
                &mut commands,
                &asset_server,
                &windows,
                &camera_query,
                sound,
                world_to_map_point(explosion.position),
                Vec2::ZERO,
                Some(&mut rng),
            );
        }
        if let Some(crater) = explosion.crater {
            spawn_crater_stamp(
                &mut commands,
                &asset_server,
                &map.0,
                &tile_info.0,
                &mut crater_registry,
                &mut rng,
                world_to_map_point(explosion.position),
                crater,
            );
        }
        spawn_damage_missile_impact_effects(
            &mut commands,
            &asset_server,
            &mut rng,
            &object_snapshots,
            explosion.position,
            explosion.visual,
        );
        let affected_refs = explosion_damage_targets(&object_snapshots, explosion);

        for (ref_id, damage) in affected_refs {
            let mut target_query = queries.p2();
            let Some((entity, _, mut target_stats, damage_cause)) = target_query
                .iter_mut()
                .find(|(_, object, _, _)| object.ref_id == ref_id)
            else {
                continue;
            };

            target_stats.health =
                (target_stats.health - damage).clamp(0.0, target_stats.max_health);
            mark_missile_damage(&mut commands, entity, damage_cause);
        }
    }
}

fn damage_missile_impact_sound(visual: DamageMissileVisual) -> Option<GameSoundKind> {
    Some(match visual {
        DamageMissileVisual::MapObjectTurrent(_) => GameSoundKind::TurrentExplosion,
        _ => GameSoundKind::RandomExplosion,
    })
}

fn process_damage_missile_replicas(
    mut commands: Commands,
    time: Res<Time>,
    mut replicas: Query<(
        Entity,
        &mut Transform,
        &mut Sprite,
        &mut DamageMissileReplica,
    )>,
) {
    for (entity, mut transform, mut sprite, mut replica) in &mut replicas {
        replica.time_remaining -= time.delta_secs();
        animate_damage_missile_replica_sprite(&mut sprite, &mut replica, time.delta_secs());
        let progress = if replica.total_time <= 0.0 {
            1.0
        } else {
            (1.0 - replica.time_remaining / replica.total_time).clamp(0.0, 1.0)
        };
        let position = replica.start.lerp(replica.target, progress) + replica.offset;
        transform.translation.x = position.x;
        transform.translation.y = position.y;

        if replica.time_remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_damage_missile_replica_sprite(
    sprite: &mut Sprite,
    replica: &mut DamageMissileReplica,
    delta_secs: f32,
) {
    if replica.frames.len() <= 1 {
        return;
    }

    replica.frame_elapsed += delta_secs;
    while replica.frame_elapsed >= replica.frame_time {
        replica.frame_elapsed -= replica.frame_time;
        replica.frame = (replica.frame + 1) % replica.frames.len();
    }

    if let Some(frame) = replica.frames.get(replica.frame) {
        sprite.image = frame.clone();
    }
}

fn explosion_damage_targets(
    object_snapshots: &[CombatObjectSnapshot],
    explosion: PendingExplosion,
) -> Vec<(u32, f32)> {
    object_snapshots
        .iter()
        .filter_map(|snapshot| {
            if snapshot.stats.destroyed() {
                return None;
            }
            if explosion.team != TeamType::Null && snapshot.team == explosion.team {
                return None;
            }

            let distance = snapshot.position.distance(explosion.position);
            if distance > explosion.radius {
                return None;
            }

            let damage = aoe_damage_at_distance(explosion.damage, explosion.radius, distance);
            (damage > 0.0).then_some((snapshot.ref_id, damage))
        })
        .collect()
}

fn spawn_crater_stamp(
    commands: &mut Commands,
    asset_server: &AssetServer,
    map: &ZMap,
    tile_info: &[original::tileinfo::PaletteTileInfo],
    registry: &mut CraterStampRegistry,
    rng: &mut CombatRng,
    map_position: Vec2,
    crater: DamageCrater,
) {
    let Some(spec) = create_crater_stamp_spec(map, tile_info, registry, rng, map_position, crater)
    else {
        return;
    };

    let top_left_map = Vec2::new(
        spec.tile.x as f32 * TILE_SIZE,
        spec.tile.y as f32 * TILE_SIZE,
    );
    let top_left_world = map_top_left_to_world(top_left_map);
    commands.spawn((
        Sprite::from_image(asset_server.load(spec.asset_path.clone())),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(top_left_world.x, top_left_world.y, 0.4),
        Name::new("crater_stamp"),
    ));

    if spec.is_big {
        for y in spec.tile.y..=spec.tile.y + 1 {
            for x in spec.tile.x..=spec.tile.x + 1 {
                registry.stamped_tiles.insert((x, y));
            }
        }
    }
}

fn create_crater_stamp_spec(
    map: &ZMap,
    tile_info: &[original::tileinfo::PaletteTileInfo],
    registry: &CraterStampRegistry,
    rng: &mut CombatRng,
    map_position: Vec2,
    crater: DamageCrater,
) -> Option<CraterStampSpec> {
    let mut x = map_position.x as i32;
    let mut y = map_position.y as i32;
    let mut is_big = crater
        .big_chance
        .map_or(crater.is_big, |chance| rng.next_roll() <= chance);
    if rng.next_roll() > crater.chance {
        return None;
    }
    if is_big {
        x -= 8;
        y -= 8;
    }

    let mut tile = IVec2::new(x >> 4, y >> 4);
    if !tile_in_map(map, tile) {
        return None;
    }
    if is_big && (tile.x + 1 >= map.basics.width as i32 || tile.y + 1 >= map.basics.height as i32) {
        is_big = false;
    }

    let crater_type = coord_crater_type(map, tile_info, tile)?;
    if is_big && crater_variant_count(map.basics.terrain_type, crater_type, false) == 0 {
        is_big = false;
    }

    if !is_big {
        if registry.stamped_tiles.contains(&(tile.x, tile.y)) {
            return None;
        }
    } else {
        let free_tiles: Vec<IVec2> = crater_big_tiles(tile)
            .into_iter()
            .filter(|candidate| !registry.stamped_tiles.contains(&(candidate.x, candidate.y)))
            .collect();
        if free_tiles.is_empty() {
            return None;
        }
        if free_tiles.len() < 4 {
            is_big = false;
            tile = free_tiles[rng.index(free_tiles.len())];
        }
    }

    if is_big {
        let big_tiles = crater_big_tiles(tile);
        let all_same_type = big_tiles.iter().all(|candidate| {
            coord_crater_type(map, tile_info, *candidate)
                .is_some_and(|candidate_type| candidate_type == crater_type)
        });

        if !all_same_type {
            is_big = false;
            let ok_points: Vec<IVec2> = big_tiles
                .into_iter()
                .filter(|candidate| {
                    coord_crater_type(map, tile_info, *candidate).is_some_and(|candidate_type| {
                        crater_variant_count(map.basics.terrain_type, candidate_type, true) > 0
                    })
                })
                .collect();
            if ok_points.is_empty() {
                return None;
            }
            tile = ok_points[rng.index(ok_points.len())];
        }
    }

    let variant_count = crater_variant_count(map.basics.terrain_type, crater_type, !is_big);
    if variant_count == 0 {
        return None;
    }
    let variant = rng.index(variant_count);
    let asset_path = crater_asset_path(map.basics.terrain_type, crater_type, !is_big, variant);

    Some(CraterStampSpec {
        tile,
        is_big,
        crater_type,
        variant,
        asset_path,
    })
}

fn crater_big_tiles(tile: IVec2) -> [IVec2; 4] {
    [
        tile,
        IVec2::new(tile.x + 1, tile.y),
        IVec2::new(tile.x, tile.y + 1),
        IVec2::new(tile.x + 1, tile.y + 1),
    ]
}

fn tile_in_map(map: &ZMap, tile: IVec2) -> bool {
    tile.x >= 0
        && tile.y >= 0
        && tile.x < map.basics.width as i32
        && tile.y < map.basics.height as i32
}

fn coord_crater_type(
    map: &ZMap,
    tile_info: &[original::tileinfo::PaletteTileInfo],
    tile: IVec2,
) -> Option<i16> {
    if !tile_in_map(map, tile) {
        return None;
    }
    let tile_index = tile.y as usize * map.basics.width as usize + tile.x as usize;
    let tile_id = map.tiles.get(tile_index)?.tile as usize;
    tile_info.get(tile_id).map(|info| info.crater_type)
}

fn crater_variant_count(planet: PlanetType, crater_type: i16, is_small: bool) -> usize {
    if crater_type < 0 {
        return 0;
    }

    let counts = match planet {
        PlanetType::Desert => {
            if is_small {
                &[3, 7, 1, 1, 1, 1, 0][..]
            } else {
                &[0, 3, 0, 0, 0, 0, 0][..]
            }
        }
        PlanetType::Volcanic => {
            if is_small {
                &[4, 4, 0][..]
            } else {
                &[2, 2, 0][..]
            }
        }
        PlanetType::Arctic => {
            if is_small {
                &[2, 4][..]
            } else {
                &[0, 2][..]
            }
        }
        PlanetType::Jungle => {
            if is_small {
                &[2, 4, 4][..]
            } else {
                &[0, 2, 2][..]
            }
        }
        PlanetType::City => {
            if is_small {
                &[2, 4, 4][..]
            } else {
                &[0, 2, 2][..]
            }
        }
    };

    counts.get(crater_type as usize).copied().unwrap_or(0)
}

fn crater_asset_path(
    planet: PlanetType,
    crater_type: i16,
    is_small: bool,
    variant: usize,
) -> String {
    let size = if is_small { "small" } else { "large" };
    let planet = planet_asset_name(planet);
    format!("planets/craters/crater_{size}_{planet}_t{crater_type:02}_n{variant:02}.png")
}

fn process_destroyed_fort_eliminations(
    mut commands: Commands,
    game_atlases: Res<GameAtlases>,
    mut zones: ResMut<ZoneOwnership>,
    mut queries: ParamSet<(
        Query<(&GameObjectEntity, &ObjectTeam, &ObjectStats), Without<DestroyedObject>>,
        Query<(&GameObjectEntity, &ObjectTeam, &mut ObjectStats), Without<DestroyedObject>>,
        Query<(
            Entity,
            &GameObjectEntity,
            &mut ObjectTeam,
            &ObjectStats,
            Option<&BuildingLevel>,
            Option<&mut BuildingProduction>,
            Option<&mut Sprite>,
            Option<&mut AtlasAnimation>,
        )>,
        Query<(&MinimapDot, &mut Sprite)>,
        Query<(&MinimapZone, &mut Sprite)>,
    )>,
) {
    let snapshots: Vec<(u32, ObjectKind, TeamType, bool)> = queries
        .p0()
        .iter()
        .map(|(object, team, stats)| (object.ref_id, object.kind, team.0, stats.destroyed()))
        .collect();

    let mut eliminated_teams: Vec<TeamType> = snapshots
        .iter()
        .filter_map(|(_, kind, team, destroyed)| {
            eliminated_team_for_destroyed_fort(*kind, *team, *destroyed)
        })
        .collect();

    let death_teams = unique_non_null_teams(
        snapshots
            .iter()
            .filter(|(_, _, _, destroyed)| *destroyed)
            .map(|(_, _, team, _)| *team),
    );
    for team in death_teams {
        if !team_has_alive_combat_unit(&snapshots, team) && team_has_fort(&snapshots, team) {
            eliminated_teams.push(team);
        }
    }
    eliminated_teams = unique_non_null_teams(eliminated_teams.into_iter());

    for eliminated_team in eliminated_teams {
        for (object, team, mut stats) in &mut queries.p1() {
            if object_should_be_destroyed_by_fort_elimination(
                object.kind,
                team.0,
                stats.destroyed(),
                eliminated_team,
            ) {
                stats.health = 0.0;
            }
        }

        let flag_refs: Vec<u32> = queries
            .p2()
            .iter_mut()
            .filter_map(|(_, object, team, _, _, _, _, _)| {
                (team.0 == eliminated_team
                    && matches!(object.kind, ObjectKind::MapItem(id) if id == ItemType::Flag as u8))
                .then_some(object.ref_id)
            })
            .collect();

        for flag_ref in flag_refs {
            award_zone_to_team_for_elimination(
                flag_ref,
                TeamType::Null,
                &mut commands,
                &game_atlases,
                &mut zones,
                &mut queries,
            );
        }
    }
}

fn unique_non_null_teams(teams: impl Iterator<Item = TeamType>) -> Vec<TeamType> {
    let mut unique = Vec::new();
    for team in teams {
        if team != TeamType::Null && !unique.contains(&team) {
            unique.push(team);
        }
    }
    unique
}

fn eliminated_team_for_destroyed_fort(
    kind: ObjectKind,
    team: TeamType,
    destroyed: bool,
) -> Option<TeamType> {
    (destroyed
        && team != TeamType::Null
        && matches!(
            kind,
            ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack)
        ))
    .then_some(team)
}

fn team_has_alive_combat_unit(
    snapshots: &[(u32, ObjectKind, TeamType, bool)],
    team: TeamType,
) -> bool {
    snapshots.iter().any(|(_, kind, object_team, destroyed)| {
        *object_team == team
            && !*destroyed
            && matches!(
                kind,
                ObjectKind::Robot(_) | ObjectKind::Vehicle(_) | ObjectKind::Cannon(_)
            )
    })
}

fn team_has_fort(snapshots: &[(u32, ObjectKind, TeamType, bool)], team: TeamType) -> bool {
    snapshots.iter().any(|(_, kind, object_team, destroyed)| {
        *object_team == team
            && !*destroyed
            && matches!(
                kind,
                ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack)
            )
    })
}

fn object_should_be_destroyed_by_fort_elimination(
    kind: ObjectKind,
    team: TeamType,
    destroyed: bool,
    eliminated_team: TeamType,
) -> bool {
    team == eliminated_team && !destroyed && !matches!(kind, ObjectKind::MapItem(_))
}

fn award_zone_to_team_for_elimination(
    flag_ref_id: u32,
    new_team: TeamType,
    commands: &mut Commands,
    game_atlases: &GameAtlases,
    zones: &mut ZoneOwnership,
    queries: &mut ParamSet<(
        Query<(&GameObjectEntity, &ObjectTeam, &ObjectStats), Without<DestroyedObject>>,
        Query<(&GameObjectEntity, &ObjectTeam, &mut ObjectStats), Without<DestroyedObject>>,
        Query<(
            Entity,
            &GameObjectEntity,
            &mut ObjectTeam,
            &ObjectStats,
            Option<&BuildingLevel>,
            Option<&mut BuildingProduction>,
            Option<&mut Sprite>,
            Option<&mut AtlasAnimation>,
        )>,
        Query<(&MinimapDot, &mut Sprite)>,
        Query<(&MinimapZone, &mut Sprite)>,
    )>,
) {
    let Some(link) = zones.zone_for_flag(flag_ref_id).cloned() else {
        return;
    };

    if zones.owners.get(link.zone_index).copied() == Some(new_team) {
        return;
    }

    if let Some(owner) = zones.owners.get_mut(link.zone_index) {
        *owner = new_team;
    }

    let linked_building_refs = link.building_refs;
    let mut refs_to_update = linked_building_refs.clone();
    refs_to_update.push(flag_ref_id);

    for (
        entity,
        object,
        mut team,
        stats,
        maybe_level,
        maybe_production,
        maybe_sprite,
        maybe_animation,
    ) in &mut queries.p2()
    {
        let linked_object = refs_to_update.contains(&object.ref_id);

        if linked_object {
            team.0 = new_team;
        }

        if let Some(mut production) = maybe_production {
            reset_build_time(&mut production, *stats, zones.team_zone_ownage(team.0));
            if linked_building_refs.contains(&object.ref_id) {
                if let Some(level) = maybe_level {
                    set_default_production(&mut production, object.kind, level.0, *stats);
                }
            }
        } else if linked_building_refs.contains(&object.ref_id) {
            if let Some(level) = maybe_level {
                let mut production =
                    match initial_production_for_building(object.kind, level.0, new_team, *stats) {
                        Some(production) => production,
                        None => continue,
                    };
                reset_build_time(&mut production, *stats, zones.team_zone_ownage(new_team));
                commands.entity(entity).insert(production);
            }
        }

        if !linked_object {
            continue;
        }

        if object.ref_id == flag_ref_id {
            if let Some(mut animation) = maybe_animation {
                if let Some(indices) = game_atlases.flag_animation_indices(new_team) {
                    animation.frames = indices;
                    animation.current = 0;
                    if let Some(mut sprite) = maybe_sprite {
                        if let Some(index) = animation.frames.first().copied() {
                            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                                atlas.index = index;
                            }
                        }
                    }
                }
            }
        }
    }

    for (dot, mut sprite) in &mut queries.p3() {
        if refs_to_update.contains(&dot.ref_id) {
            sprite.color = new_team.color();
        }
    }
    for (zone, mut sprite) in &mut queries.p4() {
        if zone.zone_index == link.zone_index {
            sprite.color = minimap_zone_color(new_team);
        }
    }
}

fn process_destroyed_objects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_atlases: Res<GameAtlases>,
    map: Res<CurrentMap>,
    rock_atlas: Res<RockAtlas>,
    mut passability: ResMut<PassabilityGrid>,
    mut rng: ResMut<CombatRng>,
    mut selection: ResMut<SelectionState>,
    mut object_queries: ParamSet<(
        Query<
            (
                &GameObjectEntity,
                &ObjectTeam,
                &ObjectStats,
                &Transform,
                &MapGridPosition,
                Option<&GrenadeBox>,
                Option<&BridgeFootprint>,
                Option<&DamageCauseTimers>,
                Option<&MobileSpriteLayer>,
            ),
            Without<DestroyedObject>,
        >,
        Query<
            (&GameObjectEntity, &Transform, &Selectable, &mut ObjectStats),
            Without<DestroyedObject>,
        >,
        Query<
            (
                &GameObjectEntity,
                &ObjectStats,
                &mut RobotGroup,
                Option<&mut GrenadeInventory>,
            ),
            Without<DestroyedObject>,
        >,
    )>,
    mut layer_query: Query<(
        Entity,
        &ObjectLayerRef,
        Option<&GameObjectEntity>,
        Option<&mut Sprite>,
        &mut Visibility,
    )>,
    rock_pieces: Query<(Entity, &RockRenderPiece)>,
    minimap_dots: Query<(Entity, &MinimapDot)>,
    selection_markers: Query<(Entity, &SelectionMarker)>,
    selection_health_bars: Query<(Entity, &SelectionHealthBar)>,
) {
    let destroyed_refs: Vec<DestroyedObjectSnapshot> = object_queries
        .p0()
        .iter()
        .filter(|(_, _, stats, _, _, _, _, _, _)| stats.destroyed())
        .map(
            |(object, team, _, transform, grid, grenade_box, bridge, cause, mobile)| {
                let (mobile_rotation, mobile_frame) = mobile
                    .filter(|mobile| mobile.role == MobileSpriteRole::VehicleBase)
                    .map_or((180, 0), |mobile| (mobile.rotation, mobile.frame));
                DestroyedObjectSnapshot {
                    ref_id: object.ref_id,
                    kind: object.kind,
                    team: team.0,
                    position: transform.translation.truncate(),
                    grid: *grid,
                    mobile_rotation,
                    mobile_frame,
                    grenade_amount: grenade_box.map_or(0, |grenade_box| grenade_box.amount),
                    bridge: bridge.copied(),
                    do_fire_death: cause.is_some_and(|cause| cause.fire > 0.0),
                    do_missile_death: cause.is_some_and(|cause| cause.missile > 0.0),
                }
            },
        )
        .collect();

    if destroyed_refs.is_empty() {
        return;
    }

    let group_snapshots: Vec<RobotGroupMemberSnapshot> = object_queries
        .p2()
        .iter()
        .map(
            |(object, stats, group, inventory)| RobotGroupMemberSnapshot {
                ref_id: object.ref_id,
                leader_ref_id: group.leader_ref_id,
                destroyed: stats.destroyed(),
                grenade_amount: inventory.map_or(0, |inventory| inventory.amount),
            },
        )
        .collect();
    let destroyed_ref_ids: Vec<u32> = destroyed_refs
        .iter()
        .map(|destroyed| destroyed.ref_id)
        .collect();
    let group_promotions =
        robot_group_promotions_for_removed_refs(&destroyed_ref_ids, &group_snapshots);

    if !group_promotions.is_empty() {
        for (object, stats, mut group, inventory) in &mut object_queries.p2() {
            let Some(promotion) = group_promotions.iter().find(|promotion| {
                group.leader_ref_id == promotion.old_leader_ref_id && !stats.destroyed()
            }) else {
                continue;
            };

            group.leader_ref_id = promotion.new_leader_ref_id;
            if object.ref_id == promotion.new_leader_ref_id && promotion.grenade_amount > 0 {
                if let Some(mut inventory) = inventory {
                    inventory.amount = promotion.grenade_amount;
                }
            }
        }

        remap_selected_refs_for_group_promotions(&mut selection.selected_refs, &group_promotions);
    }

    let planet = map.0.basics.terrain_type;
    let mut bridge_killed_refs = Vec::new();
    for destroyed in &destroyed_refs {
        let Some(bridge) = destroyed.bridge else {
            continue;
        };

        passability.set_bridge_destroyed(bridge.x, bridge.y, bridge.building, bridge.extra_links);
        spawn_bridge_turrent_effects(
            &mut commands,
            &asset_server,
            &mut rng,
            planet,
            bridge,
            false,
        );

        for (object, transform, selectable, mut stats) in &mut object_queries.p1() {
            if object.ref_id == destroyed.ref_id
                || !bridge_destroy_kills_unit(
                    object.kind,
                    *stats,
                    transform.translation.truncate(),
                    selectable.selection_size,
                    bridge,
                )
            {
                continue;
            }

            stats.health = 0.0;
            bridge_killed_refs.push(object.ref_id);
        }
    }

    selection.selected_refs.retain(|ref_id| {
        !destroyed_refs
            .iter()
            .any(|destroyed| destroyed.ref_id == *ref_id)
            && !bridge_killed_refs.contains(ref_id)
    });

    for destroyed in destroyed_refs {
        spawn_grenade_box_destroy_missiles(
            &mut commands,
            &asset_server,
            &mut rng,
            destroyed.kind,
            destroyed.position,
            destroyed.grenade_amount,
        );
        spawn_cannon_death_effect(
            &mut commands,
            &asset_server,
            &mut rng,
            destroyed.kind,
            destroyed.team,
            destroyed.position,
        );
        spawn_vehicle_death_effect(
            &mut commands,
            &asset_server,
            &mut rng,
            destroyed.kind,
            destroyed.team,
            destroyed.position,
            destroyed.mobile_rotation,
            destroyed.mobile_frame,
        );
        spawn_robot_death_effect(
            &mut commands,
            &asset_server,
            &mut rng,
            destroyed.kind,
            destroyed.team,
            destroyed.position,
            destroyed.do_fire_death,
            destroyed.do_missile_death,
        );
        spawn_building_death_effect(
            &mut commands,
            &asset_server,
            &mut rng,
            destroyed.kind,
            destroyed.grid,
            planet,
        );
        spawn_map_object_death_effect(
            &mut commands,
            &asset_server,
            &mut rng,
            destroyed.kind,
            destroyed.grid,
            planet,
        );
        spawn_map_object_turrent_missile(
            &mut commands,
            &asset_server,
            &mut rng,
            destroyed.kind,
            destroyed.grid,
        );

        let destroyable = can_be_destroyed(destroyed.kind);
        let destroyed_asset = destroyed_asset_name(destroyed.kind, destroyed.team, planet);
        let bridge_destroyed_frames = destroyed.bridge.map(|bridge| {
            game_atlases.bridge_layers(
                bridge.building,
                planet,
                bridge.extra_links,
                BridgeVisualState::Destroyed,
            )
        });
        let mut bridge_frame_iter = bridge_destroyed_frames.into_iter().flatten();

        for (entity, layer_ref, maybe_object, maybe_sprite, mut visibility) in &mut layer_query {
            if layer_ref.0 != destroyed.ref_id {
                continue;
            }

            if destroyable {
                commands.entity(entity).despawn();
                continue;
            }

            commands
                .entity(entity)
                .remove::<MovementPath>()
                .remove::<AttackTarget>()
                .insert(DestroyedObject);

            let is_base_layer = maybe_object.is_some();
            if is_base_layer && buildings::auto_repairable_after_destroy(destroyed.kind) {
                commands.entity(entity).insert(AutoRepair {
                    timer: buildings::auto_repair_delay(&mut rng),
                });
            }

            if let Some(asset_name) = destroyed_asset.as_ref() {
                if is_base_layer {
                    if let Some(mut sprite) = maybe_sprite {
                        sprite.image = asset_server.load(asset_name.clone());
                        sprite.texture_atlas = None;
                        sprite.rect = None;
                        sprite.custom_size = None;
                        *visibility = Visibility::Visible;
                        continue;
                    }
                } else {
                    *visibility = Visibility::Hidden;
                    continue;
                }
            }

            if let Some(frame) = bridge_frame_iter.next() {
                if let Some(mut sprite) = maybe_sprite {
                    apply_sprite_frame(&mut sprite, frame);
                    *visibility = Visibility::Visible;
                    continue;
                }
            }

            if is_base_layer {
                *visibility = Visibility::Visible;
            }
        }

        if destroyable {
            if destroyed.kind == ObjectKind::Rock {
                passability.set_walkable_tile(
                    destroyed.grid.x,
                    destroyed.grid.y.saturating_add(2),
                    true,
                );
                for (entity, piece) in &rock_pieces {
                    if piece.x == destroyed.grid.x && piece.y == destroyed.grid.y {
                        commands.entity(entity).despawn();
                    }
                }
                spawn_destroyed_rock_rubble(
                    &mut commands,
                    &rock_atlas,
                    &mut rng,
                    destroyed.grid,
                    planet,
                );
            } else if map_item_blocks_tile(destroyed.kind) {
                passability.set_walkable_tile(destroyed.grid.x, destroyed.grid.y, true);
            }

            for (entity, dot) in &minimap_dots {
                if dot.ref_id == destroyed.ref_id {
                    commands.entity(entity).despawn();
                }
            }
        }

        for (entity, marker) in &selection_markers {
            if marker.ref_id == destroyed.ref_id {
                commands.entity(entity).despawn();
            }
        }
        for (entity, bar) in &selection_health_bars {
            if bar.ref_id == destroyed.ref_id {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn process_building_auto_repairs(
    mut commands: Commands,
    time: Res<Time>,
    game_atlases: Res<GameAtlases>,
    map: Res<CurrentMap>,
    mut passability: ResMut<PassabilityGrid>,
    mut queries: ParamSet<(
        Query<
            (
                Entity,
                &GameObjectEntity,
                &ObjectTeam,
                &MapGridPosition,
                &BuildingLevel,
                Option<&BridgeFootprint>,
                &mut ObjectStats,
                &mut AutoRepair,
            ),
            With<DestroyedObject>,
        >,
        Query<(&GameObjectEntity, &MapGridPosition, &ObjectStats), With<DestroyedObject>>,
    )>,
    mut layers: Query<(
        Entity,
        &ObjectLayerRef,
        Option<&mut Sprite>,
        &mut Visibility,
    )>,
) {
    let destroyed_fort_zones: Vec<usize> = queries
        .p1()
        .iter()
        .filter_map(|(object, grid, stats)| {
            (stats.destroyed() && buildings::auto_repair_blocking_fort(object.kind))
                .then(|| zone_at_tile(&map.0, grid.x, grid.y))
                .flatten()
        })
        .collect();

    let delta = time.delta_secs();
    for (entity, object, team, grid, level, bridge, mut stats, mut repair) in &mut queries.p0() {
        repair.timer -= delta;
        if repair.timer > 0.0 {
            continue;
        }

        let repair_zone = zone_at_tile(&map.0, grid.x, grid.y);
        if repair_zone.is_some_and(|zone| destroyed_fort_zones.contains(&zone)) {
            commands.entity(entity).remove::<AutoRepair>();
            continue;
        }

        stats.health = stats.max_health;
        commands
            .entity(entity)
            .remove::<AutoRepair>()
            .remove::<DestroyedObject>();
        remove_destroyed_marker_for_ref(&mut commands, &mut layers, object.ref_id);

        if let Some(bridge) = bridge.copied() {
            passability.set_bridge_repaired(
                bridge.x,
                bridge.y,
                bridge.building,
                bridge.extra_links,
            );
            commands.entity(entity).insert(BridgeRevivePending {
                bridge,
                timer: buildings::BRIDGE_REVIVE_RERENDER_DELAY,
                spawned_effect: false,
            });
            continue;
        }

        let frames = revived_object_frames(
            &game_atlases,
            map.0.basics.terrain_type,
            object.kind,
            team.0,
            *grid,
            *level,
            0,
        );
        apply_bridge_layers_for_ref(object.ref_id, frames, &mut layers);
    }
}

fn revived_object_frames(
    game_atlases: &GameAtlases,
    planet: PlanetType,
    kind: ObjectKind,
    team: TeamType,
    grid: MapGridPosition,
    level: BuildingLevel,
    extra_links: u16,
) -> Vec<crate::render::atlas::SpriteFrame> {
    let Some((object_type, object_id)) = object_kind_to_map_parts(kind) else {
        return Vec::new();
    };
    let object = MapObject {
        x: grid.x,
        y: grid.y,
        owner: team,
        object_type,
        object_id,
        building_level: level.original(),
        extra_links,
        health_percent: 100,
    };
    game_atlases.sprite_layers_for_object(&object, planet)
}

fn remove_destroyed_marker_for_ref(
    commands: &mut Commands,
    layers: &mut Query<(
        Entity,
        &ObjectLayerRef,
        Option<&mut Sprite>,
        &mut Visibility,
    )>,
    ref_id: u32,
) {
    for (entity, layer_ref, _, _) in layers {
        if layer_ref.0 == ref_id {
            commands.entity(entity).remove::<DestroyedObject>();
        }
    }
}

fn bridge_destroy_kills_unit(
    kind: ObjectKind,
    stats: ObjectStats,
    position: Vec2,
    selection_size: Vec2,
    bridge: BridgeFootprint,
) -> bool {
    buildings::bridge_destroy_kills_unit(kind, stats, position, selection_size, bridge)
}

fn spawn_grenade_box_destroy_missiles(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    kind: ObjectKind,
    position: Vec2,
    grenade_amount: u8,
) {
    if !is_grenade_box(kind) || grenade_amount == 0 {
        return;
    }

    for missile in item_grenades::destroy_missile_rules(position, grenade_amount, rng) {
        let frames = damage_missile_frame_handles(asset_server, DamageMissileVisual::Grenade);
        commands.spawn((
            damage_missile_sprite(frames.first().cloned()),
            Transform::from_xyz(position.x, position.y, 35.0),
            DamageMissile {
                start: position,
                target: missile.target,
                time_remaining: missile.delay,
                total_time: missile.delay,
                damage: item_grenades::destroy_missile_damage(),
                radius: item_grenades::destroy_missile_radius(),
                team: TeamType::Null,
                visual: DamageMissileVisual::Grenade,
                frames,
                frame_time: DAMAGE_MISSILE_FRAME_TIME,
                frame_elapsed: 0.0,
                frame: 0,
                visual_rise: 0.0,
                angle_degrees_per_sec: 0.0,
                crater: Some(item_grenades::destroy_missile_crater()),
                visual_offset: Vec2::ZERO,
                smoke_offsets: Vec::new(),
                smoke_time_cursor: 0.0,
            },
            Name::new("grenade_box_destroy_missile"),
        ));
    }
}

fn spawn_map_object_turrent_missile(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    kind: ObjectKind,
    grid: MapGridPosition,
) {
    let Some(object_i) = map_object::turrent_object_index(kind) else {
        return;
    };
    let top_left = buildings::building_top_left_from_grid(grid);
    let start_map = map_object::turrent_start(top_left, rng);
    let impact_map = map_object::turrent_target(top_left, rng);
    let delay = map_object::turrent_delay(rng);
    let frames = damage_missile_frame_handles(
        asset_server,
        DamageMissileVisual::MapObjectTurrent(object_i),
    );
    let visual_offset = map_object::turrent_visual_offset(object_i);
    let visual_start = map_point_to_world(start_map) + visual_offset;
    let angle_degrees_per_sec = map_object::turrent_spin_degrees_per_sec(rng);

    commands.spawn((
        damage_missile_sprite(frames.first().cloned()),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(visual_start.x, visual_start.y, 35.0),
        DamageMissile {
            start: map_point_to_world(start_map),
            target: map_point_to_world(impact_map),
            time_remaining: delay,
            total_time: delay,
            damage: MAP_ITEM_TURRENT_DAMAGE,
            radius: MAP_ITEM_TURRENT_RADIUS,
            team: TeamType::Null,
            visual: DamageMissileVisual::MapObjectTurrent(object_i),
            frames,
            frame_time: DAMAGE_MISSILE_FRAME_TIME,
            frame_elapsed: 0.0,
            frame: 0,
            visual_rise: map_object::turrent_rise(rng),
            angle_degrees_per_sec,
            crater: None,
            visual_offset,
            smoke_offsets: Vec::new(),
            smoke_time_cursor: 0.0,
        },
        Name::new("map_object_turrent_missile"),
    ));
}

fn can_be_destroyed(kind: ObjectKind) -> bool {
    match kind {
        ObjectKind::Building(_) | ObjectKind::Bridge(_) => false,
        ObjectKind::MapItem(id) if id == ItemType::Flag as u8 => false,
        ObjectKind::Rock
        | ObjectKind::Cannon(_)
        | ObjectKind::Vehicle(_)
        | ObjectKind::Robot(_)
        | ObjectKind::Animal(_)
        | ObjectKind::MapItem(_) => true,
    }
}

fn map_item_blocks_tile(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::MapItem(id)
            if id == ItemType::Hut as u8 || id >= ItemType::MapObjectStart as u8
    )
}

fn planet_asset_name(planet: PlanetType) -> &'static str {
    match planet {
        PlanetType::Desert => "desert",
        PlanetType::Volcanic => "volcanic",
        PlanetType::Arctic => "arctic",
        PlanetType::Jungle => "jungle",
        PlanetType::City => "city",
    }
}

fn destroyed_asset_name(kind: ObjectKind, team: TeamType, planet: PlanetType) -> Option<String> {
    match kind {
        ObjectKind::Building(building) => buildings::destroyed_asset_path(building, planet),
        ObjectKind::Bridge(_) => None,
        ObjectKind::Vehicle(vehicle) => vehicles::destroyed_asset_path(vehicle, team),
        ObjectKind::Cannon(cannon) => cannons::destroyed_asset_path(cannon, team),
        _ => None,
    }
}

fn aoe_damage_at_distance(damage: f32, radius: f32, distance: f32) -> f32 {
    if radius <= 0.0 || distance > radius {
        0.0
    } else {
        damage * (1.0 - distance / radius)
    }
}

fn move_commanded_objects(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    game_atlases: Res<GameAtlases>,
    map: Res<CurrentMap>,
    tile_info: Res<CurrentTileInfo>,
    passability: Res<PassabilityGrid>,
    mut rng: ResMut<CombatRng>,
    mut queries: ParamSet<(
        Query<
            (
                &GameObjectEntity,
                &Transform,
                &ObjectStats,
                Option<&mut MovementStamina>,
            ),
            Without<DestroyedObject>,
        >,
        Query<
            (
                Entity,
                &mut Transform,
                &mut MovementPath,
                &ObjectLayerRef,
                Option<&GameObjectEntity>,
                Option<&ObjectStats>,
                Option<&mut VehicleEffectDropTimer>,
                Option<&mut Sprite>,
                Option<&mut MobileSpriteLayer>,
            ),
            Without<DestroyedObject>,
        >,
        Query<
            (&MovementPath, &ObjectLayerRef, Option<&GameObjectEntity>),
            Without<DestroyedObject>,
        >,
    )>,
) {
    let delta_secs = time.delta_secs();
    let run_requests: Vec<MovementRunRequest> = queries
        .p2()
        .iter()
        .filter_map(|(path, layer_ref, maybe_object)| {
            let _ = maybe_object?;
            path.waypoints
                .last()
                .copied()
                .map(|target| MovementRunRequest {
                    ref_id: layer_ref.0,
                    target,
                    speed: path.speed,
                    attempt_run: path.attempt_run,
                })
        })
        .collect();

    let mut speed_snapshots = Vec::new();
    for (object, transform, stats, maybe_stamina) in &mut queries.p0() {
        let request = run_requests
            .iter()
            .find(|request| request.ref_id == object.ref_id)
            .copied();
        let mut running = false;

        if let Some(mut stamina) = maybe_stamina {
            process_run_stamina(&mut stamina, delta_secs);
            if let Some(request) = request {
                if request.attempt_run {
                    stamina.running = false;
                    let can_reach = can_reach_target_running(
                        request.speed,
                        stamina.current,
                        transform.translation.truncate(),
                        request.target,
                    );
                    attempt_start_run(&mut stamina, can_reach, rng.next_roll());
                }
            }
            running = stamina.running;
        }

        speed_snapshots.push(MovementSpeedSnapshot {
            ref_id: object.ref_id,
            multiplier: movement_speed_multiplier(object.kind, *stats, running),
        });
    }

    for (
        entity,
        mut transform,
        mut path,
        layer_ref,
        maybe_object,
        maybe_stats,
        maybe_drop_timer,
        mut maybe_sprite,
        mut maybe_mobile,
    ) in &mut queries.p1()
    {
        let Some(target) = path.waypoints.first().copied() else {
            if let (Some(sprite), Some(mobile)) = (&mut maybe_sprite, &mut maybe_mobile) {
                update_mobile_sprite(&game_atlases, sprite, mobile, false, delta_secs);
            }
            commands.entity(entity).remove::<MovementPath>();
            continue;
        };

        let position = transform.translation.truncate();
        let to_target = target - position;
        let terrain_speed = passability.walk_speed_at_world(position);
        let multiplier = speed_snapshots
            .iter()
            .find(|snapshot| snapshot.ref_id == layer_ref.0)
            .map_or(1.0, |snapshot| snapshot.multiplier);
        let max_step = path.speed * terrain_speed * multiplier * delta_secs;
        let moving = to_target.length_squared() > 0.01;
        path.attempt_run = false;

        if terrain_speed <= 0.0 {
            continue;
        }

        let direction = direction_index_from_delta(to_target);
        if let (Some(sprite), Some(mobile)) = (&mut maybe_sprite, &mut maybe_mobile) {
            if let Some(direction) = direction {
                mobile.rotation = rotation_for_direction(direction);
            }
            update_mobile_sprite(&game_atlases, sprite, mobile, moving, delta_secs);
        }

        if moving {
            if let (Some(object), Some(stats), Some(mut drop_timer), Some(direction)) =
                (maybe_object, maybe_stats, maybe_drop_timer, direction)
            {
                maybe_spawn_vehicle_movement_effects(
                    &mut commands,
                    &asset_server,
                    &map.0,
                    &tile_info.0,
                    &mut rng,
                    object.kind,
                    *stats,
                    transform.translation.truncate(),
                    direction,
                    &mut drop_timer,
                    delta_secs,
                );
            }
        }

        if to_target.length_squared() <= max_step * max_step {
            transform.translation.x = target.x;
            transform.translation.y = target.y;
            path.waypoints.remove(0);
            if path.waypoints.is_empty() {
                if let (Some(sprite), Some(mobile)) = (&mut maybe_sprite, &mut maybe_mobile) {
                    update_mobile_sprite(&game_atlases, sprite, mobile, false, delta_secs);
                }
                commands.entity(entity).remove::<MovementPath>();
            }
        } else {
            let step = to_target.normalize() * max_step;
            transform.translation.x += step.x;
            transform.translation.y += step.y;
        }
    }
}

fn maybe_spawn_vehicle_movement_effects(
    commands: &mut Commands,
    asset_server: &AssetServer,
    map: &ZMap,
    tile_info: &[original::tileinfo::PaletteTileInfo],
    rng: &mut CombatRng,
    kind: ObjectKind,
    stats: ObjectStats,
    position: Vec2,
    direction: usize,
    timer: &mut VehicleEffectDropTimer,
    delta_secs: f32,
) {
    if !tick_vehicle_effect_drop_timer(timer, delta_secs) {
        return;
    }

    let ObjectKind::Vehicle(vehicle) = kind else {
        return;
    };

    let map_center = Vec2::new(position.x, -position.y);
    let track_points = vehicles::track_points(map_center, direction, rng);
    let lay_tracks = track_points
        .map(|point| !track_point_is_near_road(map, tile_info, point) && rng.next_roll() < 0.8);

    if lay_tracks[0] || lay_tracks[1] {
        if let Some(track_paths) =
            vehicles::track_effect_frame_paths(vehicle, map.basics.terrain_type, direction)
        {
            let track_frames: Vec<_> = track_paths
                .into_iter()
                .map(|path| asset_server.load(path))
                .collect();
            let start_delay = vehicles::track_start_delay(rng);
            for (point, lay_track) in track_points.into_iter().zip(lay_tracks) {
                if lay_track {
                    spawn_vehicle_track_effect(
                        commands,
                        track_frames.clone(),
                        map_top_left_to_world(point),
                        start_delay,
                    );
                    if rng.next_roll() < 0.75 {
                        spawn_tank_dirt_effect(
                            commands,
                            asset_server,
                            map.basics.terrain_type,
                            point,
                            rng,
                        );
                    }
                }
            }
        }
    }

    let health_ratio = if stats.max_health <= 0.0 {
        0.0
    } else {
        stats.health / stats.max_health
    };

    if vehicles::show_partially_damaged_effects(health_ratio) && rng.next_roll() < 1.0 / 3.0 {
        spawn_tank_smoke_effect(commands, asset_server, map_center, direction, false, rng);
    }

    if vehicles::show_heavily_damaged_effects(health_ratio) {
        if rng.next_roll() < 1.0 / 3.0 {
            spawn_tank_smoke_effect(commands, asset_server, map_center, direction, true, rng);
        }
        if rng.next_roll() < 1.0 / 16.0 {
            spawn_tank_oil_effect(commands, asset_server, map_center, direction, rng);
        }
        if rng.next_roll() < 1.0 / 48.0 {
            spawn_tank_spark_effect(commands, asset_server, map_center, direction, rng);
        }
    }
}

fn tick_vehicle_effect_drop_timer(timer: &mut VehicleEffectDropTimer, delta_secs: f32) -> bool {
    vehicles::effect_drop_timer_ready(&mut timer.elapsed, delta_secs)
}

fn track_point_is_near_road(
    map: &ZMap,
    tile_info: &[original::tileinfo::PaletteTileInfo],
    point: Vec2,
) -> bool {
    [
        point,
        point + Vec2::new(0.0, -16.0),
        point + Vec2::new(0.0, 16.0),
        point + Vec2::new(-16.0, 0.0),
        point + Vec2::new(16.0, 0.0),
    ]
    .into_iter()
    .any(|candidate| coord_is_road(map, tile_info, candidate))
}

fn coord_is_road(
    map: &ZMap,
    tile_info: &[original::tileinfo::PaletteTileInfo],
    point: Vec2,
) -> bool {
    let tx = (point.x / TILE_SIZE).floor() as i32;
    let ty = (point.y / TILE_SIZE).floor() as i32;
    if tx < 0 || ty < 0 || tx >= map.basics.width as i32 || ty >= map.basics.height as i32 {
        return false;
    }

    let Some(tile) = map
        .tiles
        .get(ty as usize * map.basics.width as usize + tx as usize)
    else {
        return false;
    };
    tile_info
        .get(tile.tile as usize)
        .is_some_and(|info| info.is_road)
}

fn spawn_tank_dirt_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    planet: PlanetType,
    point: Vec2,
    rng: &mut CombatRng,
) {
    let Some(frame_paths) = vehicles::tank_dirt_frame_paths(planet, rng) else {
        return;
    };
    let frames: Vec<_> = frame_paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();

    spawn_image_effect(
        commands,
        frames,
        map_point_to_world(point),
        bevy::sprite::Anchor::BOTTOM_CENTER,
        vehicles::TANK_DUST_FRAME_TIME,
        None,
        3.0,
    );
}

fn spawn_tank_smoke_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    center: Vec2,
    direction: usize,
    spark_first: bool,
    rng: &mut CombatRng,
) {
    let frames: Vec<_> = vehicles::tank_smoke_frame_paths(direction, spark_first)
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();

    spawn_image_effect(
        commands,
        frames,
        map_top_left_to_world(vehicles::tank_smoke_top_left(center, direction, rng)),
        bevy::sprite::Anchor::TOP_LEFT,
        vehicles::TANK_DUST_FRAME_TIME,
        None,
        3.5,
    );
}

fn spawn_tank_oil_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    center: Vec2,
    direction: usize,
    rng: &mut CombatRng,
) {
    let frames: Vec<_> = vehicles::tank_oil_frame_paths(rng)
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let frame_time = vehicles::tank_oil_frame_time(rng);
    spawn_image_effect(
        commands,
        frames,
        map_top_left_to_world(center + vehicles::tank_oil_offset(direction, rng)),
        bevy::sprite::Anchor::TOP_LEFT,
        frame_time,
        None,
        2.5,
    );
}

fn spawn_tank_spark_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    center: Vec2,
    direction: usize,
    rng: &mut CombatRng,
) {
    let frames: Vec<_> = vehicles::tank_spark_frame_paths()
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let lifetime_frames = vehicles::tank_spark_lifetime_frames(rng);
    spawn_image_effect(
        commands,
        frames,
        map_top_left_to_world(center + vehicles::tank_spark_offset(direction, rng)),
        bevy::sprite::Anchor::TOP_LEFT,
        vehicles::TANK_SPARK_FRAME_TIME,
        Some(lifetime_frames),
        3.6,
    );
}

fn spawn_image_effect(
    commands: &mut Commands,
    frames: Vec<Handle<Image>>,
    position: Vec2,
    anchor: bevy::sprite::Anchor,
    frame_time: f32,
    remaining_advances: Option<usize>,
    z: f32,
) {
    let Some(first) = frames.first().cloned() else {
        return;
    };
    commands.spawn((
        Sprite {
            image: first,
            ..default()
        },
        anchor,
        Transform::from_xyz(position.x, position.y, z),
        ImageEffectAnimation {
            frames,
            frame_time,
            elapsed: 0.0,
            current: 0,
            remaining_advances,
        },
        Name::new("vehicle_movement_effect"),
    ));
}

fn spawn_vehicle_track_effect(
    commands: &mut Commands,
    frames: Vec<Handle<Image>>,
    position: Vec2,
    start_delay: f32,
) {
    let Some(first) = frames.first().cloned() else {
        return;
    };
    commands.spawn((
        Sprite {
            image: first,
            ..default()
        },
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(position.x, position.y, 2.0),
        VehicleTrackEffect {
            frames,
            elapsed: 0.0,
            start_delay,
            current: 0,
        },
        Name::new("vehicle_track_effect"),
    ));
}

fn animate_vehicle_track_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Sprite, &mut VehicleTrackEffect)>,
) {
    for (entity, mut sprite, mut effect) in &mut effects {
        effect.elapsed += time.delta_secs();
        let delta = effect.elapsed - effect.start_delay;
        let Some(frame) = vehicles::track_frame_for_delta(delta) else {
            commands.entity(entity).despawn();
            continue;
        };
        if frame != effect.current {
            effect.current = frame.min(effect.frames.len().saturating_sub(1));
            if let Some(image) = effect.frames.get(effect.current) {
                sprite.image = image.clone();
            }
        }
    }
}

fn animate_image_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Sprite, &mut ImageEffectAnimation)>,
) {
    for (entity, mut sprite, mut animation) in &mut effects {
        animation.elapsed += time.delta_secs();
        if animation.elapsed < animation.frame_time {
            continue;
        }

        animation.elapsed %= animation.frame_time;
        if let Some(remaining) = animation.remaining_advances.as_mut() {
            if *remaining == 0 {
                commands.entity(entity).despawn();
                continue;
            }
            *remaining -= 1;
            if *remaining == 0 {
                commands.entity(entity).despawn();
                continue;
            }
            animation.current = (animation.current + 1) % animation.frames.len();
            sprite.image = animation.frames[animation.current].clone();
            continue;
        }

        if animation.current + 1 >= animation.frames.len() {
            commands.entity(entity).despawn();
            continue;
        }

        animation.current += 1;
        sprite.image = animation.frames[animation.current].clone();
    }
}

fn sync_crane_conco_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<CombatRng>,
    cranes: Query<(
        &GameObjectEntity,
        &ObjectTeam,
        &Transform,
        &Selectable,
        Option<&CraneRepairTarget>,
    )>,
    mut effects: Query<(Entity, &mut CraneConcoEffect)>,
    mut parts: Query<(Entity, &mut CraneConcoPart)>,
) {
    let crane_states: HashMap<u32, CraneConcoCraneSnapshot> = cranes
        .iter()
        .filter_map(|(object, team, transform, selectable, target)| {
            if !matches!(object.kind, ObjectKind::Vehicle(VehicleType::Crane)) {
                return None;
            }
            let target = target.and_then(|target| {
                (target.stage != CraneRepairStage::GotoEntrance).then_some(
                    CraneConcoTargetSnapshot {
                        ref_id: target.ref_id,
                        top_left_map: target.target_top_left_map,
                        size: target.target_size,
                        is_bridge: target.target_is_bridge,
                    },
                )
            });
            Some((
                object.ref_id,
                CraneConcoCraneSnapshot {
                    ref_id: object.ref_id,
                    team: team.0,
                    top_left_map: object_top_left_map(
                        transform.translation.truncate(),
                        selectable.selection_size,
                    ),
                    target,
                },
            ))
        })
        .collect();

    let mut effect_refs = HashSet::new();
    let mut spawn_requests = Vec::new();

    for (entity, mut effect) in &mut effects {
        effect_refs.insert(effect.crane_ref_id);
        let Some(crane) = crane_states.get(&effect.crane_ref_id).copied() else {
            commands.entity(entity).despawn();
            despawn_crane_conco_parts(&mut commands, effect.crane_ref_id, &mut parts);
            continue;
        };

        match (crane.target, effect.phase) {
            (Some(target), CraneConcoPhase::Returning) => {
                commands.entity(entity).despawn();
                despawn_crane_conco_parts(&mut commands, effect.crane_ref_id, &mut parts);
                spawn_requests.push((crane, target));
            }
            (Some(_), _) => {}
            (None, CraneConcoPhase::Returning) => {}
            (None, _) => begin_crane_conco_return(&mut effect, crane.top_left_map, &mut parts),
        }
    }

    for crane in crane_states.values().copied() {
        if crane.team == TeamType::Null || effect_refs.contains(&crane.ref_id) {
            continue;
        }
        if let Some(target) = crane.target {
            spawn_requests.push((crane, target));
        }
    }

    for (crane, target) in spawn_requests {
        spawn_crane_conco_effect(
            &mut commands,
            &asset_server,
            &mut rng,
            crane.ref_id,
            crane.team,
            crane.top_left_map,
            target,
        );
    }
}

fn spawn_crane_conco_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    crane_ref_id: u32,
    team: TeamType,
    crane_top_left_map: Vec2,
    target: CraneConcoTargetSnapshot,
) {
    let effect = CraneConcoEffect::new(crane_ref_id, team);
    commands.spawn((
        effect,
        Name::new(format!(
            "crane_conco_effect_{}_target_{}",
            crane_ref_id, target.ref_id
        )),
    ));

    for destination in crane_unit::static_destinations(
        crane_top_left_map,
        target.top_left_map,
        target.size,
        target.is_bridge,
    ) {
        spawn_crane_conco_part(
            commands,
            asset_server,
            crane_ref_id,
            team,
            destination.item,
            crane_unit::start_top_left(crane_top_left_map),
            destination.top_left_map,
            destination.size,
            CraneConcoPhase::TravelingTo,
            7,
        );
    }

    for item in [CraneConcoRenderItem::Jack, CraneConcoRenderItem::Paper] {
        let destination = crane_unit::bot_destination(
            crane_top_left_map,
            target.top_left_map,
            target.size,
            target.is_bridge,
            rng,
        );
        spawn_crane_conco_part(
            commands,
            asset_server,
            crane_ref_id,
            team,
            item,
            crane_unit::start_top_left(crane_top_left_map),
            destination,
            crane_unit::CONCO_BOT_SIZE,
            CraneConcoPhase::TravelingTo,
            7,
        );
    }
}

fn spawn_crane_conco_part(
    commands: &mut Commands,
    asset_server: &AssetServer,
    crane_ref_id: u32,
    team: TeamType,
    item: CraneConcoRenderItem,
    start_map: Vec2,
    dest_map: Vec2,
    size: Vec2,
    phase: CraneConcoPhase,
    frame: usize,
) {
    let surface =
        crane_unit::surface_for_part(item, phase, frame, 0, 0, false, dest_map.x - start_map.x);
    let world = map_top_left_to_world(start_map);
    commands.spawn((
        Sprite::from_image(asset_server.load(crane_unit::surface_asset_path(team, surface))),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(world.x, world.y, crane_unit::z(start_map, size)),
        CraneConcoPart {
            crane_ref_id,
            item,
            start_map,
            dest_map,
            current_map: start_map,
            size,
            w_dist: dest_map.x - start_map.x,
        },
        Name::new(format!("crane_conco_{crane_ref_id}_{item:?}")),
    ));
}

fn begin_crane_conco_return(
    effect: &mut CraneConcoEffect,
    crane_top_left_map: Vec2,
    parts: &mut Query<(Entity, &mut CraneConcoPart)>,
) {
    effect.phase = CraneConcoPhase::Returning;
    effect.elapsed = 0.0;
    effect.frame = 0;

    for (_, mut part) in parts.iter_mut() {
        if part.crane_ref_id != effect.crane_ref_id {
            continue;
        }
        part.start_map = part.current_map;
        part.dest_map = crane_unit::return_top_left(crane_top_left_map, part.size);
        part.w_dist = part.dest_map.x - part.start_map.x;
    }
}

fn despawn_crane_conco_parts(
    commands: &mut Commands,
    crane_ref_id: u32,
    parts: &mut Query<(Entity, &mut CraneConcoPart)>,
) {
    for (part_entity, part) in parts.iter_mut() {
        if part.crane_ref_id == crane_ref_id {
            commands.entity(part_entity).despawn();
        }
    }
}

fn animate_crane_conco_effects(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<CombatRng>,
    mut effects: Query<(Entity, &mut CraneConcoEffect)>,
    mut parts: Query<(Entity, &mut CraneConcoPart, &mut Transform, &mut Sprite)>,
) {
    let delta = time.delta_secs();
    let mut finished = HashSet::new();

    for (entity, mut effect) in &mut effects {
        effect.elapsed += delta;
        match effect.phase {
            CraneConcoPhase::TravelingTo => {
                if effect.elapsed >= crane_unit::CONCO_TRAVEL_TIME {
                    effect.phase = CraneConcoPhase::Working;
                    effect.elapsed = 0.0;
                    effect.frame = 0;
                } else {
                    effect.frame = crane_unit::travel_frame(
                        effect.elapsed / crane_unit::CONCO_TRAVEL_TIME,
                        false,
                    );
                }
            }
            CraneConcoPhase::Working => {
                effect.frame = 0;
                crane_unit::advance_bots(&mut effect, delta, &mut rng);
            }
            CraneConcoPhase::Returning => {
                if effect.elapsed >= crane_unit::CONCO_TRAVEL_TIME {
                    finished.insert(effect.crane_ref_id);
                    commands.entity(entity).despawn();
                } else {
                    effect.frame = crane_unit::travel_frame(
                        effect.elapsed / crane_unit::CONCO_TRAVEL_TIME,
                        true,
                    );
                }
            }
        }
    }

    let effect_states: HashMap<u32, (TeamType, CraneConcoPhase, usize, usize, usize, bool)> =
        effects
            .iter()
            .map(|(_, effect)| {
                (
                    effect.crane_ref_id,
                    (
                        effect.team,
                        effect.phase,
                        effect.frame,
                        effect.jackbot_i,
                        effect.pbot_i,
                        effect.pbot_pointing,
                    ),
                )
            })
            .collect();

    for (entity, mut part, mut transform, mut sprite) in &mut parts {
        if finished.contains(&part.crane_ref_id) {
            commands.entity(entity).despawn();
            continue;
        }
        let Some((team, phase, frame, jackbot_i, pbot_i, pbot_pointing)) =
            effect_states.get(&part.crane_ref_id).copied()
        else {
            commands.entity(entity).despawn();
            continue;
        };
        let progress = crane_unit::phase_progress(phase, frame);
        part.current_map = part.start_map + (part.dest_map - part.start_map) * progress;
        let surface = crane_unit::surface_for_part(
            part.item,
            phase,
            frame,
            jackbot_i,
            pbot_i,
            pbot_pointing,
            part.w_dist,
        );
        sprite.image = asset_server.load(crane_unit::surface_asset_path(team, surface));
        let world = map_top_left_to_world(part.current_map);
        transform.translation =
            Vec3::new(world.x, world.y, crane_unit::z(part.current_map, part.size));
    }
}

fn object_top_left_map(center_world: Vec2, size: Vec2) -> Vec2 {
    world_to_map_point(center_world) - size * 0.5
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MissileObjectParticleTarget {
    kind: ObjectKind,
    top_left_map: Vec2,
    size: Vec2,
}

fn rocket_impact_profile(visual: DamageMissileVisual) -> Option<RocketImpactProfile> {
    units::rocket_impact_profile(visual)
}

fn missile_object_particle_radius(radius: f32) -> f32 {
    radius * 0.8
}

fn missile_object_particle_targets(
    object_snapshots: &[CombatObjectSnapshot],
    center_map: Vec2,
    radius: f32,
) -> Vec<MissileObjectParticleTarget> {
    let radius = missile_object_particle_radius(radius);
    object_snapshots
        .iter()
        .filter_map(|snapshot| {
            if !matches!(
                snapshot.kind,
                ObjectKind::Cannon(_) | ObjectKind::Vehicle(_) | ObjectKind::Robot(_)
            ) {
                return None;
            }

            let size = snapshot.size.max(Vec2::splat(1.0));
            let center_map = center_map;
            let object_center_map = world_to_map_point(snapshot.position);
            let top_left_map = object_center_map - size * 0.5;
            if top_left_map.x > center_map.x + radius
                || top_left_map.x + size.x < center_map.x - radius
                || top_left_map.y > center_map.y + radius
                || top_left_map.y + size.y < center_map.y - radius
            {
                return None;
            }

            Some(MissileObjectParticleTarget {
                kind: snapshot.kind,
                top_left_map,
                size,
            })
        })
        .collect()
}

fn missile_object_particle_amount(
    kind: ObjectKind,
    particles: usize,
    rng: &mut CombatRng,
) -> usize {
    let mut amount = 14 + rng.index(particles.max(1));
    if matches!(kind, ObjectKind::Robot(_)) {
        amount /= 2;
    }
    amount
}

fn spawn_missile_object_particles(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    object_snapshots: &[CombatObjectSnapshot],
    center_map: Vec2,
    radius: f32,
    particles: usize,
) {
    for target in missile_object_particle_targets(object_snapshots, center_map, radius) {
        let amount = missile_object_particle_amount(target.kind, particles, rng);
        let width = target.size.x.max(1.0) as usize;
        let height = target.size.y.max(1.0) as usize;
        for _ in 0..amount {
            let anchor_map =
                target.top_left_map + Vec2::new(rng.index(width) as f32, rng.index(height) as f32);
            spawn_unit_particle_effect(commands, asset_server, rng, anchor_map, 25.0, 25.0);
        }
    }
}

fn spawn_damage_missile_impact_effects(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    object_snapshots: &[CombatObjectSnapshot],
    world_position: Vec2,
    visual: DamageMissileVisual,
) {
    let map_position = world_to_map_point(world_position);
    match visual {
        DamageMissileVisual::LightRocket { .. }
        | DamageMissileVisual::MissileCannon
        | DamageMissileVisual::MissileLauncher => {
            let profile = rocket_impact_profile(visual).expect("rocket visual has impact profile");
            for _ in 0..profile.xx_large_mushrooms {
                let offset = Vec2::new(9.0 - rng.index(18) as f32, -(rng.index(18) as f32));
                spawn_tough_mushroom_effect(commands, asset_server, map_position + offset, 1.5);
            }
            for _ in 0..profile.large_mushrooms {
                let offset = Vec2::new(7.0 - rng.index(14) as f32, -(rng.index(14) as f32));
                spawn_tough_mushroom_effect(commands, asset_server, map_position + offset, 1.3);
            }
            for _ in 0..profile.small_mushrooms {
                let offset = Vec2::new(5.0 - rng.index(10) as f32, -(rng.index(10) as f32));
                spawn_tough_mushroom_effect(commands, asset_server, map_position + offset, 1.0);
            }
            spawn_tough_mushroom_effect(commands, asset_server, map_position, 1.0);
            spawn_missile_object_particles(
                commands,
                asset_server,
                rng,
                object_snapshots,
                map_position,
                profile.unit_particle_radius,
                profile.unit_particle_amount,
            );
        }
        DamageMissileVisual::ToughRocket => {
            spawn_tough_mushroom_effect(commands, asset_server, map_position, 1.0);
        }
        DamageMissileVisual::MapObjectTurrent(_) => {
            spawn_tough_mushroom_effect(commands, asset_server, map_position, 1.0);
            for _ in 0..map_object::death_unit_particle_count(rng) {
                spawn_unit_particle_effect(commands, asset_server, rng, map_position, 65.0, 55.0);
            }
        }
        DamageMissileVisual::Generic | DamageMissileVisual::Grenade => {
            let mushroom_offset = Vec2::new(7.0 - rng.index(14) as f32, -(rng.index(14) as f32));
            spawn_tough_mushroom_effect(
                commands,
                asset_server,
                map_position + mushroom_offset,
                1.3,
            );

            let side_offset = Vec2::new(24.0 - rng.index(48) as f32, 24.0 - rng.index(48) as f32);
            spawn_side_explosion_effect(commands, asset_server, rng, map_position + side_offset);
        }
    }
}

fn spawn_tough_mushroom_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    map_center: Vec2,
    scale: f32,
) {
    spawn_dynamic_image_effect(
        commands,
        robots::tough_mushroom_frame_paths()
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        robots::tough_mushroom_base_top_left(map_center, scale),
        Vec2::ZERO,
        robots::tough_mushroom_frame_offsets(scale),
        scale,
        robots::tough_mushroom_effect_profile().frame_time,
        34.0,
        "tough_mushroom_impact",
    );
}

fn spawn_tough_smoke_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    map_top_left: Vec2,
) {
    spawn_image_effect(
        commands,
        robots::tough_smoke_frame_paths()
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        map_top_left_to_world(map_top_left),
        bevy::sprite::Anchor::TOP_LEFT,
        robots::tough_smoke_effect_profile().frame_time,
        None,
        34.2,
    );
}

fn spawn_pyro_fire_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    world_center: Vec2,
) {
    let profile = robots::pyro_fire_impact_profile();
    let family = rng.index(profile.family_count);
    let frames: Vec<_> = robots::pyro_fire_impact_frame_paths(family)
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let Some(first) = frames.first().cloned() else {
        return;
    };
    commands.spawn((
        Sprite::from_image(first),
        bevy::sprite::Anchor::CENTER,
        Transform::from_xyz(world_center.x, world_center.y, profile.z),
        ImageEffectAnimation {
            frames,
            frame_time: profile.frame_time,
            elapsed: 0.0,
            current: 0,
            remaining_advances: None,
        },
        Name::new("pyro_fire_impact"),
    ));
}

fn spawn_side_explosion_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    map_center: Vec2,
) {
    spawn_side_explosion_effect_scaled(commands, asset_server, rng, map_center, 1.0);
}

fn spawn_side_explosion_effect_scaled(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    map_center: Vec2,
    scale: f32,
) {
    let speed = 20.0 + rng.index(10) as f32;
    let end = map_center + Vec2::new(20.0 - rng.index(40) as f32, 20.0 - rng.index(40) as f32);
    let mut direction = end - map_center;
    let mag = direction.length();
    if mag <= f32::EPSILON {
        direction = Vec2::X;
    } else {
        direction /= mag;
    }

    spawn_dynamic_image_effect(
        commands,
        side_explosion_frame_paths()
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        side_explosion_base_top_left_scaled(map_center, scale),
        direction * speed,
        Vec::new(),
        scale,
        SIDE_EXPLOSION_FRAME_TIME,
        34.1,
        "side_explosion_impact",
    );
}

fn spawn_dynamic_image_effect(
    commands: &mut Commands,
    frames: Vec<Handle<Image>>,
    base_map_position: Vec2,
    velocity_map: Vec2,
    frame_offsets: Vec<Vec2>,
    scale: f32,
    frame_time: f32,
    z: f32,
    name: &'static str,
) {
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let world = map_top_left_to_world(base_map_position);
    commands.spawn((
        Sprite::from_image(first),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform {
            translation: Vec3::new(world.x, world.y, z),
            scale: Vec3::splat(scale),
            ..default()
        },
        DynamicImageEffect {
            frames,
            frame_time,
            elapsed: 0.0,
            age: 0.0,
            current: 0,
            base_map_position,
            velocity_map,
            frame_offsets,
            scale,
        },
        Name::new(name),
    ));
}

fn animate_dynamic_image_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Sprite, &mut Transform, &mut DynamicImageEffect)>,
) {
    let delta = time.delta_secs();
    for (entity, mut sprite, mut transform, mut effect) in &mut effects {
        effect.age += delta;
        effect.elapsed += delta;

        while effect.elapsed >= effect.frame_time {
            effect.elapsed -= effect.frame_time;
            if effect.current + 1 >= effect.frames.len() {
                commands.entity(entity).despawn();
                break;
            }
            effect.current += 1;
            sprite.image = effect.frames[effect.current].clone();
        }

        let frame_offset = effect
            .frame_offsets
            .get(effect.current)
            .copied()
            .unwrap_or(Vec2::ZERO);
        let map_position =
            effect.base_map_position + effect.velocity_map * effect.age + frame_offset;
        let world = map_top_left_to_world(map_position);
        transform.translation.x = world.x;
        transform.translation.y = world.y;
        transform.scale = Vec3::splat(effect.scale);
    }
}

fn side_explosion_frame_paths() -> Vec<String> {
    (0..7)
        .map(|frame| format!("other/explosions/side_explosion_n{frame:02}.png"))
        .collect()
}

fn side_explosion_base_top_left_scaled(map_center: Vec2, scale: f32) -> Vec2 {
    map_center - Vec2::splat(16.0 * scale)
}

fn spawn_vehicle_death_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    kind: ObjectKind,
    team: TeamType,
    center: Vec2,
    rotation: u16,
    frame: usize,
) {
    let ObjectKind::Vehicle(vehicle) = kind else {
        return;
    };
    let Some(wreck_path) = vehicles::death_wreck_asset_path(vehicle, team, rotation, frame) else {
        return;
    };

    let top_left_world = vehicles::death_top_left_world(center);
    let top_left_map = world_to_map_point(top_left_world);
    let lifetime = vehicles::death_lifetime(rng);

    spawn_vehicle_death_standard_effects(
        commands,
        asset_server,
        rng,
        vehicle,
        top_left_map,
        lifetime,
    );
    spawn_vehicle_turrent_missile_effect(commands, asset_server, rng, vehicle, team, top_left_map);

    commands.spawn((
        Sprite::from_image(asset_server.load(wreck_path)),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(top_left_world.x, top_left_world.y, 33.8),
        VehicleDeathEffect {
            timer: lifetime,
            top_left_map,
        },
        Name::new("vehicle_death_effect"),
    ));
}

fn process_vehicle_death_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    mut effects: Query<(Entity, &mut VehicleDeathEffect)>,
) {
    for (entity, mut effect) in &mut effects {
        effect.timer -= time.delta_secs();
        if effect.timer > 0.0 {
            continue;
        }

        let spark_count = vehicles::death_spark_count(&mut rng);
        spawn_death_sparks(
            &mut commands,
            &asset_server,
            &mut rng,
            effect.top_left_map + Vec2::splat(16.0),
            spark_count,
        );
        commands.entity(entity).despawn();
    }
}

fn spawn_vehicle_death_standard_effects(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    vehicle: VehicleType,
    top_left_map: Vec2,
    lifetime: f32,
) {
    let Some(bounds) = vehicles::death_effect_bounds(vehicle) else {
        return;
    };
    for _ in 0..(3 + rng.index(3)) {
        let point = top_left_map + vehicles::random_death_point(bounds, rng);
        spawn_vehicle_death_standard_effect(
            commands,
            asset_server,
            rng,
            vehicles::VehicleDeathStandardKind::LittleFire,
            point,
            lifetime,
        );
    }
    for _ in 0..(1 + rng.index(2)) {
        let point = top_left_map + vehicles::random_death_point(bounds, rng);
        spawn_vehicle_death_standard_effect(
            commands,
            asset_server,
            rng,
            vehicles::VehicleDeathStandardKind::BigSmoke,
            point,
            lifetime,
        );
    }
    for _ in 0..rng.index(2) {
        let point = top_left_map + vehicles::random_death_point(bounds, rng);
        spawn_vehicle_death_standard_effect(
            commands,
            asset_server,
            rng,
            vehicles::VehicleDeathStandardKind::SmallFireSmoke,
            point,
            lifetime,
        );
    }
}

fn spawn_vehicle_death_standard_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    kind: vehicles::VehicleDeathStandardKind,
    anchor_map: Vec2,
    lifetime: f32,
) {
    let Some((base_position, paths)) = vehicles::death_standard_effect_shape(kind, anchor_map)
    else {
        return;
    };
    let frames: Vec<_> = paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let world = map_top_left_to_world(base_position);
    let current = rng.index(frames.len());

    commands.spawn((
        Sprite::from_image(frames.get(current).cloned().unwrap_or(first)),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(world.x, world.y, 34.0 + anchor_map.y * 0.0001),
        LoopingImageEffect {
            frames,
            frame_time: VEHICLE_DEATH_STANDARD_FRAME_TIME,
            elapsed: estandard_initial_elapsed(),
            lifetime,
            current,
        },
        Name::new("vehicle_death_standard_effect"),
    ));
}

fn animate_looping_image_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Sprite, &mut LoopingImageEffect)>,
) {
    let delta = time.delta_secs();
    for (entity, mut sprite, mut effect) in &mut effects {
        effect.lifetime -= delta;
        if effect.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        effect.elapsed += delta;
        while effect.elapsed >= effect.frame_time {
            effect.elapsed -= effect.frame_time;
            effect.current = (effect.current + 1) % effect.frames.len();
            sprite.image = effect.frames[effect.current].clone();
        }
    }
}

fn estandard_initial_elapsed() -> f32 {
    0.05
}

fn process_building_standard_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map: Res<CurrentMap>,
    mut rng: ResMut<CombatRng>,
    buildings: Query<(
        Entity,
        &GameObjectEntity,
        &ObjectStats,
        &MapGridPosition,
        Option<&BuildingEffectState>,
    )>,
    effects: Query<(Entity, &BuildingStandardEffect)>,
) {
    let planet = map.0.basics.terrain_type;
    for (entity, object, stats, grid, state) in &buildings {
        let active_effects: Vec<Entity> = effects
            .iter()
            .filter_map(|(effect_entity, effect)| {
                (effect.ref_id == object.ref_id).then_some(effect_entity)
            })
            .collect();
        let Some(profile) = buildings::death_profile(object.kind, planet) else {
            for effect_entity in active_effects {
                commands.entity(effect_entity).despawn();
            }
            continue;
        };

        let max_effects = state.map_or_else(
            || {
                let max_effects = buildings::standard_max_effects(profile, &mut rng);
                commands
                    .entity(entity)
                    .insert(BuildingEffectState { max_effects });
                max_effects
            },
            |state| state.max_effects,
        );
        let health_ratio = if stats.max_health <= 0.0 {
            0.0
        } else {
            (stats.health / stats.max_health).clamp(0.0, 1.0)
        };
        let should_effects = (max_effects as f32 * (1.0 - health_ratio)) as usize;
        if should_effects == 0 {
            for effect_entity in active_effects {
                commands.entity(effect_entity).despawn();
            }
            continue;
        }

        let top_left = buildings::building_top_left_from_grid(*grid);
        for _ in active_effects.len()..should_effects {
            let anchor = buildings::death_effect_point(top_left, profile.effect_box, &mut rng);
            let kind = vehicles::standard_effect_kind(&mut rng);
            spawn_building_standard_effect(
                &mut commands,
                &asset_server,
                &mut rng,
                object.ref_id,
                kind,
                anchor,
            );
        }
    }
}

fn spawn_building_standard_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    ref_id: u32,
    kind: vehicles::VehicleDeathStandardKind,
    anchor_map: Vec2,
) {
    let Some((base_position, paths)) = vehicles::death_standard_effect_shape(kind, anchor_map)
    else {
        return;
    };
    let frames: Vec<_> = paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let world = map_top_left_to_world(base_position);
    let current = rng.index(frames.len());

    commands.spawn((
        Sprite::from_image(frames.get(current).cloned().unwrap_or(first)),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(world.x, world.y, 34.0 + anchor_map.y * 0.0001),
        LoopingImageEffect {
            frames,
            frame_time: VEHICLE_DEATH_STANDARD_FRAME_TIME,
            elapsed: estandard_initial_elapsed(),
            lifetime: f32::INFINITY,
            current,
        },
        BuildingStandardEffect { ref_id },
        Name::new("building_standard_effect"),
    ));
}

fn spawn_vehicle_turrent_missile_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    vehicle: VehicleType,
    team: TeamType,
    top_left_map: Vec2,
) {
    let Some(frame_paths) = vehicles::turrent_frame_paths(vehicle, team) else {
        return;
    };
    let frames: Vec<_> = frame_paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let start_map = top_left_map + Vec2::splat(8.0);
    let end_map = vehicles::turrent_target(top_left_map + Vec2::splat(16.0), rng);
    let final_time = vehicles::turrent_flight_time(rng);
    let rise = turrent_rise(rng);
    spawn_turrent_missile_effect(
        commands,
        rng,
        frames,
        start_map,
        end_map,
        final_time,
        rise,
        "vehicle_turrent_missile_effect",
    );
}

fn spawn_cannon_death_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    kind: ObjectKind,
    team: TeamType,
    center: Vec2,
) {
    let ObjectKind::Cannon(cannon) = kind else {
        return;
    };
    if team == TeamType::Null {
        return;
    }

    let Some(path) = cannons::death_wreck_asset_path(cannon) else {
        return;
    };
    let image = asset_server.load(path);
    let top_left_world = cannons::death_top_left_world_for(cannon, center);
    let top_left_map = world_to_map_point(top_left_world);
    let center_map = world_to_map_point(center);
    let delay = cannons::death_delay_for(cannon, rng);
    let missile_offset_time = cannons::death_missile_offset_time_for(cannon, rng);
    let end_map = center_map + cannons::turrent_target_offset_for(cannon, rng);
    let rise = cannons::turrent_rise(cannon, rng);

    commands.spawn((
        Sprite::from_image(image.clone()),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(top_left_world.x, top_left_world.y, 34.0),
        CannonDeathEffect {
            cannon,
            timer: delay,
            start_map: top_left_map,
            end_map,
            missile_time: (missile_offset_time - delay).max(0.1),
            rise,
            image,
        },
        Name::new("cannon_death_effect"),
    ));
}

fn process_cannon_death_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    mut effects: Query<(Entity, &mut CannonDeathEffect)>,
) {
    for (entity, mut effect) in &mut effects {
        effect.timer -= time.delta_secs();
        if effect.timer > 0.0 {
            continue;
        }

        let spark_center = effect.start_map + Vec2::splat(16.0);
        let spark_count = cannons::death_spark_count_for(effect.cannon, &mut rng);
        spawn_death_sparks(
            &mut commands,
            &asset_server,
            &mut rng,
            spark_center,
            spark_count,
        );
        spawn_cannon_turrent_missile_effect(
            &mut commands,
            &mut rng,
            effect.image.clone(),
            effect.cannon,
            effect.start_map,
            effect.end_map,
            effect.missile_time,
            effect.rise,
        );
        commands.entity(entity).despawn();
    }
}

fn spawn_cannon_turrent_missile_effect(
    commands: &mut Commands,
    rng: &mut CombatRng,
    image: Handle<Image>,
    cannon: CannonType,
    start_map: Vec2,
    end_map: Vec2,
    final_time: f32,
    rise: f32,
) {
    let start_map = start_map + cannons::turrent_start_jitter(cannon, rng);
    let world = map_point_to_world(start_map);
    commands.spawn((
        Sprite::from_image(image),
        Transform::from_xyz(world.x, world.y, 34.2),
        CannonTurrentMissileEffect {
            cannon,
            start_map,
            end_map,
            elapsed: 0.0,
            final_time: final_time.max(0.1),
            rise,
            angle_degrees_per_sec: cannons::turrent_spin_degrees_per_sec(cannon, rng),
        },
        Name::new("cannon_turrent_missile_effect"),
    ));
}

fn animate_cannon_turrent_missile_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map: Res<CurrentMap>,
    tile_info: Res<CurrentTileInfo>,
    time: Res<Time>,
    mut crater_registry: ResMut<CraterStampRegistry>,
    mut rng: ResMut<CombatRng>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut effects: Query<
        (Entity, &mut Transform, &mut CannonTurrentMissileEffect),
        Without<MainCamera>,
    >,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut effect) in &mut effects {
        effect.elapsed += delta;
        if effect.elapsed >= effect.final_time {
            play_restricted_game_sound(
                &mut commands,
                &asset_server,
                &windows,
                &camera_query,
                GameSoundKind::TurrentExplosion,
                effect.end_map,
                Vec2::ZERO,
                None,
            );
            spawn_turrent_crater(
                &mut commands,
                &asset_server,
                &map.0,
                &tile_info.0,
                &mut crater_registry,
                &mut rng,
                effect.end_map,
            );
            spawn_damage_missile_impact_effects(
                &mut commands,
                &asset_server,
                &mut rng,
                &[],
                map_point_to_world(effect.end_map),
                DamageMissileVisual::Generic,
            );
            commands.entity(entity).despawn();
            continue;
        }

        let t = effect.elapsed;
        let ratio = (t / effect.final_time).clamp(0.0, 1.0);
        let mut map_position = effect.start_map.lerp(effect.end_map, ratio);
        let size = cannons::turrent_arc_size_for(effect.cannon, effect.rise, effect.final_time, t);
        let lift = cannons::turrent_arc_lift_pixels(effect.cannon);
        map_position.y -= size * lift;
        map_position.y += lift;
        let world = map_point_to_world(map_position);
        transform.translation.x = world.x;
        transform.translation.y = world.y;
        transform.scale = Vec3::splat(size.max(0.1));
        transform.rotation = Quat::from_rotation_z((effect.angle_degrees_per_sec * t).to_radians());
    }
}

fn spawn_death_sparks(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    map_center: Vec2,
    spark_count: usize,
) {
    let frames: Vec<_> = vehicles::death_spark_frame_paths()
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    for _ in 0..spark_count {
        spawn_death_spark(commands, rng, frames.clone(), map_center);
    }
}

fn spawn_death_spark(
    commands: &mut Commands,
    rng: &mut CombatRng,
    frames: Vec<Handle<Image>>,
    map_center: Vec2,
) {
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let lifetime = 1.5 + rng.index(3) as f32 * 0.1;
    let spark = map_center + Vec2::new(2.0 - rng.index(5) as f32, 2.0 - rng.index(5) as f32);
    let start_map = spark - Vec2::new(8.0, 5.0);
    let end_map = spark + Vec2::new(180.0 - rng.index(360) as f32, 150.0 - rng.index(220) as f32);
    let velocity_map = (end_map - start_map) / lifetime;
    let world = map_point_to_world(start_map);

    commands.spawn((
        Sprite::from_image(first),
        Transform::from_xyz(world.x, world.y, 34.4),
        DeathSparkEffect {
            frames,
            start_map,
            velocity_map,
            elapsed: 0.0,
            final_time: lifetime,
            rise: 3.0 + rng.index(300) as f32 * 0.01,
            frame_elapsed: 0.0,
            current: 0,
        },
        Name::new("death_spark_effect"),
    ));
}

fn animate_death_spark_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Sprite, &mut Transform, &mut DeathSparkEffect)>,
) {
    let delta = time.delta_secs();
    for (entity, mut sprite, mut transform, mut effect) in &mut effects {
        effect.elapsed += delta;
        if effect.elapsed >= effect.final_time {
            commands.entity(entity).despawn();
            continue;
        }

        effect.frame_elapsed += delta;
        while effect.frame_elapsed >= vehicles::DEATH_SPARK_FRAME_TIME {
            effect.frame_elapsed -= vehicles::DEATH_SPARK_FRAME_TIME;
            effect.current = (effect.current + 1) % effect.frames.len();
            sprite.image = effect.frames[effect.current].clone();
        }

        let t = effect.elapsed;
        let mut map_position = effect.start_map + effect.velocity_map * t;
        let size = vehicles::death_spark_arc_size(effect.rise, effect.final_time, t);
        map_position.y -= size * 30.0;
        let world = map_point_to_world(map_position);
        transform.translation.x = world.x;
        transform.translation.y = world.y;
        transform.scale = Vec3::splat(size.max(0.2));
    }
}

fn spawn_building_death_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    kind: ObjectKind,
    grid: MapGridPosition,
    planet: PlanetType,
) {
    let Some(profile) = buildings::death_profile(kind, planet) else {
        return;
    };

    let top_left = buildings::building_top_left_from_grid(grid);
    let fireball_count = buildings::death_fireball_count(profile, rng);
    for _ in 0..fireball_count {
        let center = buildings::death_effect_point(top_left, profile.effect_box, rng);
        spawn_side_explosion_effect_scaled(commands, asset_server, rng, center, 1.3);
    }

    let piece_count = buildings::death_piece_count(profile, rng);
    for _ in 0..piece_count {
        let start = buildings::death_effect_point(top_left, profile.effect_box, rng);
        let end = buildings::death_piece_target(top_left, profile, rng);
        let final_time = buildings::piece_flight_time(profile, rng);
        let rise = buildings::building_turrent_rise(rng);
        let piece_index = rng.index(profile.piece_variants);
        spawn_building_turrent_missile_effect(
            commands,
            asset_server,
            rng,
            piece_index,
            profile.piece_variants,
            start,
            end,
            final_time,
            rise,
        );
    }
}

fn turrent_rise(rng: &mut CombatRng) -> f32 {
    1.0 + rng.index(300) as f32 * 0.01
}

fn turrent_spin_degrees_per_sec(rng: &mut CombatRng) -> f32 {
    240.0 - rng.index(480) as f32
}

fn spawn_building_turrent_missile_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    piece_index: usize,
    piece_variants: usize,
    start_map: Vec2,
    end_map: Vec2,
    final_time: f32,
    rise: f32,
) {
    let frames: Vec<_> =
        buildings::building_piece_frame_paths_for_variants(piece_index, piece_variants)
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect();
    spawn_turrent_missile_effect(
        commands,
        rng,
        frames,
        start_map,
        end_map,
        final_time,
        rise,
        "building_turrent_missile_effect",
    );
}

fn spawn_turrent_missile_effect(
    commands: &mut Commands,
    rng: &mut CombatRng,
    frames: Vec<Handle<Image>>,
    start_map: Vec2,
    end_map: Vec2,
    final_time: f32,
    rise: f32,
    name: &'static str,
) {
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let start_map = start_map + Vec2::new(5.0 - rng.index(10) as f32, 5.0 - rng.index(10) as f32);
    let world = map_point_to_world(start_map);
    let angle_degrees_per_sec = turrent_spin_degrees_per_sec(rng);
    commands.spawn((
        Sprite::from_image(first),
        Transform::from_xyz(world.x, world.y, 34.3),
        BuildingTurrentMissileEffect {
            frames,
            start_map,
            end_map,
            elapsed: 0.0,
            final_time: final_time.max(0.1),
            rise,
            angle_degrees_per_sec,
            frame_elapsed: 0.0,
            current: 0,
        },
        Name::new(name),
    ));
}

fn animate_building_turrent_missile_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map: Res<CurrentMap>,
    tile_info: Res<CurrentTileInfo>,
    time: Res<Time>,
    mut crater_registry: ResMut<CraterStampRegistry>,
    mut rng: ResMut<CombatRng>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut effects: Query<
        (
            Entity,
            &mut Sprite,
            &mut Transform,
            &mut BuildingTurrentMissileEffect,
        ),
        Without<MainCamera>,
    >,
) {
    let delta = time.delta_secs();
    for (entity, mut sprite, mut transform, mut effect) in &mut effects {
        effect.elapsed += delta;
        if effect.elapsed >= effect.final_time {
            play_restricted_game_sound(
                &mut commands,
                &asset_server,
                &windows,
                &camera_query,
                GameSoundKind::TurrentExplosion,
                effect.end_map,
                Vec2::ZERO,
                None,
            );
            spawn_turrent_crater(
                &mut commands,
                &asset_server,
                &map.0,
                &tile_info.0,
                &mut crater_registry,
                &mut rng,
                effect.end_map,
            );
            spawn_building_piece_impact_effects(
                &mut commands,
                &asset_server,
                &mut rng,
                effect.end_map,
            );
            commands.entity(entity).despawn();
            continue;
        }

        effect.frame_elapsed += delta;
        while effect.frame_elapsed >= buildings::BUILDING_TURRENT_FRAME_TIME {
            effect.frame_elapsed -= buildings::BUILDING_TURRENT_FRAME_TIME;
            effect.current = (effect.current + 1) % effect.frames.len();
            sprite.image = effect.frames[effect.current].clone();
        }

        let t = effect.elapsed;
        let ratio = (t / effect.final_time).clamp(0.0, 1.0);
        let mut map_position = effect.start_map.lerp(effect.end_map, ratio);
        let size = cannons::turrent_arc_size(effect.rise, effect.final_time, t);
        map_position.y -= size * 30.0;
        map_position.y += 30.0;
        let world = map_point_to_world(map_position);
        transform.translation.x = world.x;
        transform.translation.y = world.y;
        transform.scale = Vec3::splat(size.max(0.1));
        transform.rotation = Quat::from_rotation_z((effect.angle_degrees_per_sec * t).to_radians());
    }
}

fn spawn_turrent_crater(
    commands: &mut Commands,
    asset_server: &AssetServer,
    map: &ZMap,
    tile_info: &[original::tileinfo::PaletteTileInfo],
    registry: &mut CraterStampRegistry,
    rng: &mut CombatRng,
    map_position: Vec2,
) {
    spawn_crater_stamp(
        commands,
        asset_server,
        map,
        tile_info,
        registry,
        rng,
        map_position,
        turrent_crater(),
    );
}

fn turrent_crater() -> DamageCrater {
    DamageCrater {
        is_big: false,
        chance: 0.35,
        big_chance: None,
    }
}

fn spawn_building_piece_impact_effects(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    map_center: Vec2,
) {
    let mushroom_offset = Vec2::new(7.0 - rng.index(14) as f32, -(rng.index(14) as f32));
    spawn_tough_mushroom_effect(commands, asset_server, map_center + mushroom_offset, 1.3);

    let side_offset = Vec2::new(24.0 - rng.index(48) as f32, 24.0 - rng.index(48) as f32);
    spawn_side_explosion_effect(commands, asset_server, rng, map_center + side_offset);

    let spark_count = 30 + rng.index(30);
    spawn_death_sparks(
        commands,
        asset_server,
        rng,
        map_center + Vec2::splat(16.0),
        spark_count,
    );
}

fn spawn_bridge_turrent_effects(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    planet: PlanetType,
    bridge: BridgeFootprint,
    reversed: bool,
) {
    let points = buildings::bridge_turrent_spawn_points(bridge, rng);
    for point in points {
        spawn_bridge_turrent_effect(commands, asset_server, rng, planet, point, reversed);
    }
}

fn process_bridge_revive_pending(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_atlases: Res<GameAtlases>,
    map: Res<CurrentMap>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    mut pending: Query<(Entity, &ObjectLayerRef, &mut BridgeRevivePending)>,
    mut layers: Query<(
        Entity,
        &ObjectLayerRef,
        Option<&mut Sprite>,
        &mut Visibility,
    )>,
) {
    let planet = map.0.basics.terrain_type;
    for (entity, layer_ref, mut revive) in &mut pending {
        if !revive.spawned_effect {
            spawn_bridge_turrent_effects(
                &mut commands,
                &asset_server,
                &mut rng,
                planet,
                revive.bridge,
                true,
            );
            revive.spawned_effect = true;
        }

        revive.timer -= time.delta_secs();
        if revive.timer > 0.0 {
            continue;
        }

        let frames = game_atlases.bridge_layers(
            revive.bridge.building,
            planet,
            revive.bridge.extra_links,
            BridgeVisualState::Live,
        );
        apply_bridge_layers_for_ref(layer_ref.0, frames, &mut layers);
        commands.entity(entity).remove::<BridgeRevivePending>();
    }
}

fn apply_bridge_layers_for_ref(
    ref_id: u32,
    frames: Vec<crate::render::atlas::SpriteFrame>,
    layers: &mut Query<(
        Entity,
        &ObjectLayerRef,
        Option<&mut Sprite>,
        &mut Visibility,
    )>,
) {
    let mut frame_iter = frames.into_iter();
    for (_, layer_ref, maybe_sprite, mut visibility) in layers {
        if layer_ref.0 != ref_id {
            continue;
        }
        let Some(frame) = frame_iter.next() else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if let Some(mut sprite) = maybe_sprite {
            apply_sprite_frame(&mut sprite, frame);
        }
        *visibility = Visibility::Visible;
    }
}

fn spawn_bridge_turrent_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    planet: PlanetType,
    anchor_map: Vec2,
    reversed: bool,
) {
    let frames: Vec<_> = buildings::bridge_turrent_frame_paths(planet)
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let trajectory = buildings::bridge_turrent_trajectory(anchor_map, reversed, rng);
    let world = map_point_to_world(trajectory.start);

    commands.spawn((
        Sprite::from_image(first),
        Transform::from_xyz(world.x, world.y, 34.3),
        BridgeTurrentEffect {
            frames,
            start_map: trajectory.start,
            end_map: trajectory.end,
            elapsed: 0.0,
            final_time: trajectory.final_time,
            rise: trajectory.rise,
            angle_degrees_per_sec: buildings::turrent_spin_degrees_per_sec(rng),
            frame_elapsed: 0.0,
            current: 0,
            reversed,
            planet,
        },
        Name::new("bridge_turrent_effect"),
    ));
}

fn animate_bridge_turrent_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    mut effects: Query<(
        Entity,
        &mut Sprite,
        &mut Transform,
        &mut BridgeTurrentEffect,
    )>,
) {
    let delta = time.delta_secs();
    for (entity, mut sprite, mut transform, mut effect) in &mut effects {
        effect.elapsed += delta;
        if effect.elapsed >= effect.final_time {
            if !effect.reversed {
                spawn_bridge_end_particles(
                    &mut commands,
                    &asset_server,
                    &mut rng,
                    effect.planet,
                    effect.end_map,
                );
            }
            commands.entity(entity).despawn();
            continue;
        }

        effect.frame_elapsed += delta;
        while effect.frame_elapsed >= buildings::BRIDGE_TURRENT_FRAME_TIME {
            effect.frame_elapsed -= buildings::BRIDGE_TURRENT_FRAME_TIME;
            effect.current = (effect.current + 1) % effect.frames.len();
            sprite.image = effect.frames[effect.current].clone();
        }

        let t = effect.elapsed;
        let ratio = (t / effect.final_time).clamp(0.0, 1.0);
        let mut map_position = effect.start_map.lerp(effect.end_map, ratio);
        let size = buildings::bridge_turrent_arc_size(effect.rise, effect.final_time, t);
        map_position.y -= (size - 1.0) * 30.0;
        let world = map_point_to_world(map_position);
        transform.translation.x = world.x;
        transform.translation.y = world.y;
        transform.scale = Vec3::splat(size.max(0.1));
        transform.rotation = Quat::from_rotation_z((effect.angle_degrees_per_sec * t).to_radians());
    }
}

fn spawn_bridge_end_particles(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    planet: PlanetType,
    map_center: Vec2,
) {
    let frames: Vec<_> = buildings::bridge_rock_particle_frame_paths(planet)
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    for _ in 0..buildings::bridge_end_particle_count(rng) {
        spawn_bridge_rock_particle(commands, rng, frames.clone(), map_center);
    }
}

fn spawn_map_object_death_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    kind: ObjectKind,
    grid: MapGridPosition,
    planet: PlanetType,
) {
    let top_left = buildings::building_top_left_from_grid(grid);
    match kind {
        ObjectKind::Rock => {
            for _ in 0..item_rock::small_particle_count(rng) {
                spawn_rock_particle(
                    commands,
                    asset_server,
                    rng,
                    planet,
                    item_rock::RockParticleKind::Small,
                    top_left,
                    80.0,
                    60.0,
                );
            }
            for _ in 0..item_rock::mid_particle_count(rng) {
                spawn_rock_particle(
                    commands,
                    asset_server,
                    rng,
                    planet,
                    item_rock::RockParticleKind::Mid,
                    top_left,
                    40.0,
                    40.0,
                );
            }
            for _ in 0..item_rock::large_turrent_count(rng) {
                spawn_rock_turrent_effect(
                    commands,
                    asset_server,
                    rng,
                    planet,
                    top_left,
                    140.0,
                    140.0,
                );
            }
        }
        ObjectKind::MapItem(id) if id >= ItemType::MapObjectStart as u8 => {
            for _ in 0..map_object::death_unit_particle_count(rng) {
                spawn_unit_particle_effect(commands, asset_server, rng, top_left, 65.0, 55.0);
            }
            let sparks = map_object::death_spark_count(rng);
            spawn_death_sparks(
                commands,
                asset_server,
                rng,
                top_left + Vec2::splat(16.0),
                sparks,
            );
        }
        _ => {}
    }
}

fn spawn_destroyed_rock_rubble(
    commands: &mut Commands,
    rock_atlas: &RockAtlas,
    rng: &mut CombatRng,
    grid: MapGridPosition,
    planet: PlanetType,
) {
    let image = match planet {
        PlanetType::Desert => rock_atlas.desert.clone(),
        PlanetType::Volcanic => rock_atlas.volcanic.clone(),
        PlanetType::Arctic => rock_atlas.arctic.clone(),
        PlanetType::Jungle => rock_atlas.jungle.clone(),
        PlanetType::City => rock_atlas.city.clone(),
    };
    let index = item_rock::destroyed_rubble_index(rng);
    let position = item_rock::destroyed_rubble_world_position(grid);

    commands.spawn((
        Sprite {
            image,
            texture_atlas: Some(TextureAtlas {
                layout: rock_atlas.layout.clone(),
                index,
            }),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 3.95),
        Name::new("destroyed_rock_rubble"),
    ));
}

fn spawn_unit_particle_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    anchor_map: Vec2,
    max_horz: f32,
    max_vert: f32,
) {
    let frames: Vec<_> = map_object::unit_particle_frame_paths()
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let trajectory =
        map_object::death_unit_particle_trajectory(anchor_map, max_horz, max_vert, rng);
    let velocity_map = (trajectory.end - trajectory.start) / trajectory.final_time;
    let world = map_point_to_world(trajectory.start);

    commands.spawn((
        Sprite::from_image(first),
        Transform::from_xyz(world.x, world.y, 34.35),
        UnitParticleEffect {
            frames,
            start_map: trajectory.start,
            velocity_map,
            elapsed: 0.0,
            final_time: trajectory.final_time,
            rise: trajectory.rise,
            frame_elapsed: 0.0,
            current: 0,
        },
        Name::new("unit_particle_effect"),
    ));
}

fn animate_unit_particle_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Sprite, &mut Transform, &mut UnitParticleEffect)>,
) {
    let delta = time.delta_secs();
    for (entity, mut sprite, mut transform, mut effect) in &mut effects {
        effect.elapsed += delta;
        if effect.elapsed >= effect.final_time {
            commands.entity(entity).despawn();
            continue;
        }

        effect.frame_elapsed += delta;
        while effect.frame_elapsed >= map_object::UNIT_PARTICLE_FRAME_TIME {
            effect.frame_elapsed -= map_object::UNIT_PARTICLE_FRAME_TIME;
            effect.current = (effect.current + 1) % effect.frames.len();
            sprite.image = effect.frames[effect.current].clone();
        }

        let t = effect.elapsed;
        let mut map_position = effect.start_map + effect.velocity_map * t;
        let size = buildings::bridge_rock_particle_arc_size(effect.rise, effect.final_time, t);
        map_position.y -= (size - 1.0) * 65.0;
        let world = map_point_to_world(map_position);
        transform.translation.x = world.x;
        transform.translation.y = world.y;
    }
}

fn spawn_rock_particle(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    planet: PlanetType,
    kind: item_rock::RockParticleKind,
    anchor_map: Vec2,
    max_horz: f32,
    max_vert: f32,
) {
    let frames: Vec<_> = item_rock::particle_frame_paths(planet, kind, rng)
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let trajectory = item_rock::particle_trajectory(anchor_map, max_horz, max_vert, rng);
    let velocity_map = (trajectory.end - trajectory.start) / trajectory.final_time;
    let world = map_point_to_world(trajectory.start);

    commands.spawn((
        Sprite::from_image(first),
        Transform::from_xyz(world.x, world.y, 34.35),
        BridgeRockParticleEffect {
            frames,
            start_map: trajectory.start,
            velocity_map,
            elapsed: 0.0,
            final_time: trajectory.final_time,
            rise: trajectory.rise,
            frame_elapsed: 0.0,
            current: 0,
        },
        Name::new("rock_particle_effect"),
    ));
}

fn spawn_rock_turrent_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    planet: PlanetType,
    anchor_map: Vec2,
    max_horz: f32,
    max_vert: f32,
) {
    let frames: Vec<_> = item_rock::turrent_frame_paths(planet, rng)
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let trajectory = item_rock::turrent_trajectory(anchor_map, max_horz, max_vert, rng);
    let velocity_map = (trajectory.end - trajectory.start) / trajectory.final_time;
    let world = map_point_to_world(trajectory.start);

    commands.spawn((
        Sprite::from_image(first),
        Transform::from_xyz(world.x, world.y, 34.4),
        RockTurrentEffect {
            frames,
            start_map: trajectory.start,
            velocity_map,
            elapsed: 0.0,
            final_time: trajectory.final_time,
            rise: trajectory.rise,
            frame_elapsed: 0.0,
            current: 0,
            angle_degrees_per_sec: item_rock::turrent_spin_degrees_per_sec(rng),
            planet,
        },
        Name::new("rock_turrent_effect"),
    ));
}

fn animate_rock_turrent_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    mut effects: Query<(Entity, &mut Sprite, &mut Transform, &mut RockTurrentEffect)>,
) {
    let delta = time.delta_secs();
    for (entity, mut sprite, mut transform, mut effect) in &mut effects {
        effect.elapsed += delta;
        if effect.elapsed >= effect.final_time {
            for _ in 0..item_rock::turrent_end_small_particle_count(&mut rng) {
                spawn_rock_particle(
                    &mut commands,
                    &asset_server,
                    &mut rng,
                    effect.planet,
                    item_rock::RockParticleKind::Small,
                    effect.start_map + effect.velocity_map * effect.final_time,
                    80.0,
                    60.0,
                );
            }
            commands.entity(entity).despawn();
            continue;
        }

        effect.frame_elapsed += delta;
        while effect.frame_elapsed >= item_rock::ROCK_PARTICLE_FRAME_TIME {
            effect.frame_elapsed -= item_rock::ROCK_PARTICLE_FRAME_TIME;
            effect.current = (effect.current + 1) % effect.frames.len();
            sprite.image = effect.frames[effect.current].clone();
        }

        let t = effect.elapsed;
        let mut map_position = effect.start_map + effect.velocity_map * t;
        let size = item_rock::arc_size(effect.rise, effect.final_time, t);
        map_position.y -= (size - 1.0) * 30.0;
        let world = map_point_to_world(map_position);
        transform.translation.x = world.x;
        transform.translation.y = world.y;
        transform.scale = Vec3::splat(size.max(0.1));
        transform.rotation = Quat::from_rotation_z((effect.angle_degrees_per_sec * t).to_radians());
    }
}

fn spawn_bridge_rock_particle(
    commands: &mut Commands,
    rng: &mut CombatRng,
    frames: Vec<Handle<Image>>,
    map_center: Vec2,
) {
    let Some(first) = frames.first().cloned() else {
        return;
    };
    let trajectory = buildings::bridge_rock_particle_trajectory(map_center, rng);
    let velocity_map = (trajectory.end - trajectory.start) / trajectory.final_time;
    let world = map_point_to_world(trajectory.start);

    commands.spawn((
        Sprite::from_image(first),
        Transform::from_xyz(world.x, world.y, 34.35),
        BridgeRockParticleEffect {
            frames,
            start_map: trajectory.start,
            velocity_map,
            elapsed: 0.0,
            final_time: trajectory.final_time,
            rise: trajectory.rise,
            frame_elapsed: 0.0,
            current: 0,
        },
        Name::new("bridge_rock_particle_effect"),
    ));
}

fn animate_bridge_rock_particle_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(
        Entity,
        &mut Sprite,
        &mut Transform,
        &mut BridgeRockParticleEffect,
    )>,
) {
    let delta = time.delta_secs();
    for (entity, mut sprite, mut transform, mut effect) in &mut effects {
        effect.elapsed += delta;
        if effect.elapsed >= effect.final_time {
            commands.entity(entity).despawn();
            continue;
        }

        effect.frame_elapsed += delta;
        while effect.frame_elapsed >= buildings::BRIDGE_ROCK_PARTICLE_FRAME_TIME {
            effect.frame_elapsed -= buildings::BRIDGE_ROCK_PARTICLE_FRAME_TIME;
            let frame_count = effect.frames.len().min(6);
            effect.current = (effect.current + 1) % frame_count;
            sprite.image = effect.frames[effect.current].clone();
        }

        let t = effect.elapsed;
        let mut map_position = effect.start_map + effect.velocity_map * t;
        let size = buildings::bridge_rock_particle_arc_size(effect.rise, effect.final_time, t);
        map_position.y -= (size - 1.0) * 150.0;
        let world = map_point_to_world(map_position);
        transform.translation.x = world.x;
        transform.translation.y = world.y;
        transform.scale = Vec3::splat(size.max(0.1));
    }
}

fn spawn_robot_death_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    kind: ObjectKind,
    team: TeamType,
    position: Vec2,
    do_fire_death: bool,
    do_missile_death: bool,
) {
    match robots::death_effect_choice(kind, team, do_fire_death, do_missile_death) {
        robots::DeathEffectChoice::None => {}
        robots::DeathEffectChoice::Turrent => {
            spawn_robot_turrent_death_effect(commands, asset_server, rng, team, position);
        }
        robots::DeathEffectChoice::Melt => {
            if let Some(paths) = robots::melt_death_frame_paths(team) {
                spawn_robot_image_death_effect(commands, asset_server, paths, position);
            }
        }
        robots::DeathEffectChoice::Normal => {
            let variant = rng.index(4);
            if let Some(paths) = robots::normal_death_frame_paths(team, variant) {
                spawn_robot_image_death_effect(commands, asset_server, paths, position);
            }
        }
    }
}

fn spawn_robot_image_death_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    paths: Vec<String>,
    center: Vec2,
) {
    let frames: Vec<_> = paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    spawn_image_effect(
        commands,
        frames,
        robots::death_top_left_world(center),
        bevy::sprite::Anchor::TOP_LEFT,
        robots::DEATH_FRAME_TIME,
        None,
        34.0,
    );
}

fn spawn_robot_turrent_death_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut CombatRng,
    team: TeamType,
    center: Vec2,
) {
    let Some(paths) = robots::turrent_death_frame_paths(team) else {
        return;
    };
    let frames: Vec<_> = paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let center_map = Vec2::new(center.x, -center.y);
    let trajectory = robots::turrent_death_trajectory(center_map, rng);
    let world = map_point_to_world(trajectory.start_map);
    let Some(first) = frames.first().cloned() else {
        return;
    };

    commands.spawn((
        Sprite::from_image(first),
        Transform::from_xyz(world.x, world.y, 34.0),
        RobotTurrentDeathEffect {
            frames,
            start: trajectory.start_map,
            end: trajectory.end_map,
            elapsed: 0.0,
            final_time: trajectory.final_time,
            rise: trajectory.rise,
            frame_elapsed: 0.0,
            current: 0,
            landed: false,
        },
        Name::new("robot_turrent_death_effect"),
    ));
}

fn animate_robot_turrent_death_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(
        Entity,
        &mut Transform,
        &mut Sprite,
        &mut RobotTurrentDeathEffect,
    )>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut sprite, mut effect) in &mut effects {
        effect.elapsed += delta;
        effect.frame_elapsed += delta;

        if effect.elapsed < effect.final_time {
            while effect.frame_elapsed >= robots::FLIP_AIR_FRAME_TIME {
                effect.frame_elapsed -= robots::FLIP_AIR_FRAME_TIME;
                effect.current = (effect.current + 1) % 8;
            }

            let t = effect.elapsed;
            let ratio = (t / effect.final_time).clamp(0.0, 1.0);
            let mut map_position = effect.start.lerp(effect.end, ratio);
            let size = robots::turrent_death_arc_size(effect.rise, effect.final_time, t);
            map_position.y -= (size - 1.0) * 30.0;
            let world = map_point_to_world(map_position);
            transform.translation.x = world.x;
            transform.translation.y = world.y;
            transform.scale = Vec3::splat(size.max(0.1));
        } else {
            if !effect.landed {
                effect.landed = true;
                effect.current = effect.current.max(8);
                effect.frame_elapsed = 0.0;
                let world = map_point_to_world(effect.end);
                transform.translation.x = world.x;
                transform.translation.y = world.y;
                transform.scale = Vec3::ONE;
            }

            while effect.frame_elapsed >= robots::FLIP_LAND_FRAME_TIME {
                effect.frame_elapsed -= robots::FLIP_LAND_FRAME_TIME;
                effect.current += 1;
                if effect.current > 32 {
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }

        if let Some(frame) = effect.frames.get(effect.current) {
            sprite.image = frame.clone();
        }
    }
}

pub(crate) fn map_point_to_world(point: Vec2) -> Vec2 {
    Vec2::new(point.x, -point.y)
}

fn world_to_map_point(point: Vec2) -> Vec2 {
    Vec2::new(point.x, -point.y)
}

pub(crate) fn map_top_left_to_world(point: Vec2) -> Vec2 {
    map_point_to_world(point)
}

fn process_run_stamina(stamina: &mut MovementStamina, delta_secs: f32) {
    if stamina.running {
        stamina.current -= delta_secs;
        if stamina.current < 0.0 {
            stamina.current = 0.0;
            stamina.running = false;
        }
    } else {
        stamina.current = (stamina.current + delta_secs * RUN_RECHARGE_RATE).min(stamina.max);
    }
}

fn can_reach_target_running(speed: f32, stamina: f32, position: Vec2, target: Vec2) -> bool {
    position.distance(target) <= speed * stamina
}

fn attempt_start_run(stamina: &mut MovementStamina, can_reach: bool, roll: f32) {
    const MIN_STAMINA: f32 = 0.3;

    if stamina.running || !can_reach || roll < 0.2 || stamina.current < MIN_STAMINA {
        return;
    }

    stamina.running = true;
}

fn movement_speed_multiplier(kind: ObjectKind, stats: ObjectStats, running: bool) -> f32 {
    damaged_speed_multiplier(kind, stats) * run_speed_multiplier(kind, stats, running)
}

fn damaged_speed_multiplier(kind: ObjectKind, stats: ObjectStats) -> f32 {
    if !matches!(kind, ObjectKind::Vehicle(_)) || stats.max_health <= 0.0 {
        return 1.0;
    }

    let ratio = stats.health / stats.max_health;
    vehicles::damaged_speed_multiplier_for_ratio(ratio)
}

fn run_speed_multiplier(kind: ObjectKind, stats: ObjectStats, running: bool) -> f32 {
    if let ObjectKind::Vehicle(_) = kind
        && stats.max_health > 0.0
    {
        return vehicles::run_speed_multiplier_for_ratio(stats.health / stats.max_health, running);
    }

    if running { RUN_UNIT_SPEED } else { 1.0 }
}

fn update_mobile_sprite(
    game_atlases: &GameAtlases,
    sprite: &mut Sprite,
    mobile: &mut MobileSpriteLayer,
    moving: bool,
    delta_secs: f32,
) {
    if moving {
        mobile.elapsed += delta_secs;
        let frame_time = mobile_frame_time(mobile.role);
        if frame_time > 0.0 && mobile.elapsed >= frame_time {
            mobile.elapsed %= frame_time;
            mobile.frame = (mobile.frame + 1) % mobile_frame_count(mobile.kind, mobile.role);
        }
    } else {
        mobile.elapsed = 0.0;
        mobile.frame = 0;
    }

    let Some(index) = game_atlases.mobile_frame_index(
        mobile.kind,
        mobile.team,
        mobile.role,
        mobile.rotation,
        mobile.frame,
        moving,
    ) else {
        return;
    };

    if let Some(atlas) = sprite.texture_atlas.as_mut() {
        atlas.index = index;
    }
}

fn mobile_frame_time(role: MobileSpriteRole) -> f32 {
    match role {
        MobileSpriteRole::Robot => 0.3,
        MobileSpriteRole::VehicleBase => 0.1,
        MobileSpriteRole::VehicleTop => 0.0,
    }
}

fn mobile_frame_count(kind: ObjectKind, role: MobileSpriteRole) -> usize {
    match (kind, role) {
        (ObjectKind::Robot(_), MobileSpriteRole::Robot) => 4,
        (ObjectKind::Vehicle(VehicleType::Jeep), MobileSpriteRole::VehicleBase) => 2,
        (ObjectKind::Vehicle(_), MobileSpriteRole::VehicleBase) => 3,
        _ => 1,
    }
}

pub(crate) fn rotation_for_direction(direction: usize) -> u16 {
    const ROTATION: [u16; 8] = [0, 45, 90, 135, 180, 225, 270, 315];
    ROTATION[direction.min(7)]
}

pub(crate) fn direction_index_from_delta(delta: Vec2) -> Option<usize> {
    let dx = delta.x;
    let dy = -delta.y;

    if dx.abs() <= f32::EPSILON && dy.abs() <= f32::EPSILON {
        return None;
    }

    let mut angle = dy.atan2(dx);
    if angle < 0.0 {
        angle += std::f32::consts::TAU;
    }
    angle += std::f32::consts::PI * 0.125;

    let pi = std::f32::consts::PI;
    Some(if angle < pi * 0.25 {
        0
    } else if angle < pi * 0.5 {
        7
    } else if angle < pi * 0.75 {
        6
    } else if angle < pi {
        5
    } else if angle < pi * 1.25 {
        4
    } else if angle < pi * 1.5 {
        3
    } else if angle < pi * 1.75 {
        2
    } else if angle < pi * 2.0 {
        1
    } else {
        0
    })
}

fn animate_atlas_sprites(time: Res<Time>, mut query: Query<(&mut Sprite, &mut AtlasAnimation)>) {
    for (mut sprite, mut animation) in &mut query {
        animation.elapsed += time.delta_secs();
        if animation.elapsed < animation.frame_time {
            continue;
        }

        animation.elapsed %= animation.frame_time;
        animation.current = (animation.current + 1) % animation.frames.len();
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = animation.frames[animation.current];
        }
    }
}

fn animate_radar_overlays(
    time: Res<Time>,
    objects: Query<(&GameObjectEntity, &ObjectTeam, &ObjectStats)>,
    mut overlays: Query<(
        &mut RadarOverlayLayer,
        &mut Transform,
        &mut Sprite,
        &mut Visibility,
    )>,
) {
    let object_states: HashMap<u32, (TeamType, ObjectStats)> = objects
        .iter()
        .map(|(object, team, stats)| (object.ref_id, (team.0, *stats)))
        .collect();

    for (mut overlay, mut transform, mut sprite, mut visibility) in &mut overlays {
        if let Some((team, stats)) = object_states.get(&overlay.ref_id).copied() {
            *visibility = if radar_overlay_should_be_visible(overlay.kind, team, stats) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }

        overlay.elapsed += time.delta_secs();
        if overlay.elapsed >= overlay.frame_time {
            overlay.elapsed %= overlay.frame_time;
            overlay.current = (overlay.current + 1) % overlay.frames.len();
        }

        apply_radar_overlay_frame(&overlay, &mut transform, &mut sprite);
    }
}

fn apply_radar_overlay_frame(
    overlay: &RadarOverlayLayer,
    transform: &mut Transform,
    sprite: &mut Sprite,
) {
    let Some(frame) = overlay.frames.get(overlay.current) else {
        return;
    };

    sprite.image = frame.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: frame.layout.clone(),
        index: frame.index,
    });
    let world = radar_overlay_world_position(overlay.top_left_map, frame);
    transform.translation.x = world.x;
    transform.translation.y = world.y;
}

fn radar_overlay_world_position(
    top_left_map: Vec2,
    frame: &crate::render::atlas::SpriteFrame,
) -> Vec2 {
    Vec2::new(
        top_left_map.x + frame.world_offset.x + frame.source_offset.x + frame.frame_size.x * 0.5,
        -(top_left_map.y + frame.world_offset.y + frame.source_offset.y + frame.frame_size.y * 0.5),
    )
}

fn radar_overlay_should_be_visible(
    kind: RadarOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
) -> bool {
    !stats.destroyed() && (owner != TeamType::Null || matches!(kind, RadarOverlayKind::FrontLight))
}

fn animate_repair_overlays(
    time: Res<Time>,
    objects: Query<(&GameObjectEntity, &ObjectTeam, &ObjectStats)>,
    repairing_units: Query<&RepairingUnit>,
    mut overlays: Query<(
        &mut RepairOverlayLayer,
        &mut Transform,
        &mut Sprite,
        &mut Visibility,
    )>,
) {
    let object_states: HashMap<u32, (TeamType, ObjectStats)> = objects
        .iter()
        .map(|(object, team, stats)| (object.ref_id, (team.0, *stats)))
        .collect();
    let busy_repair_buildings: HashSet<u32> = repairing_units
        .iter()
        .map(|repairing| repairing.building_ref_id)
        .collect();

    for (mut overlay, mut transform, mut sprite, mut visibility) in &mut overlays {
        let repairing_unit = busy_repair_buildings.contains(&overlay.ref_id);
        if let Some((team, stats)) = object_states.get(&overlay.ref_id).copied() {
            *visibility =
                if repair_overlay_should_be_visible(overlay.kind, team, stats, repairing_unit) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            advance_repair_overlay_frame(&mut overlay, time.delta_secs(), team, repairing_unit);
        } else {
            advance_repair_overlay_frame(
                &mut overlay,
                time.delta_secs(),
                TeamType::Null,
                repairing_unit,
            );
        }

        apply_repair_overlay_frame(&overlay, &mut transform, &mut sprite);
    }
}

fn advance_repair_overlay_frame(
    overlay: &mut RepairOverlayLayer,
    delta_secs: f32,
    owner: TeamType,
    repairing_unit: bool,
) {
    overlay.elapsed += delta_secs;
    if overlay.elapsed >= overlay.frame_time {
        overlay.elapsed %= overlay.frame_time;
        overlay.current = (overlay.current + 1) % overlay.frames.len();
    }

    if let Some(forced) = repair_overlay_forced_frame(overlay.kind, owner, repairing_unit) {
        overlay.current = forced.min(overlay.frames.len().saturating_sub(1));
    }
}

fn apply_repair_overlay_frame(
    overlay: &RepairOverlayLayer,
    transform: &mut Transform,
    sprite: &mut Sprite,
) {
    let Some(frame) = overlay.frames.get(overlay.current) else {
        return;
    };

    sprite.image = frame.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: frame.layout.clone(),
        index: frame.index,
    });
    let world = radar_overlay_world_position(overlay.top_left_map, frame);
    transform.translation.x = world.x;
    transform.translation.y = world.y;
}

fn repair_overlay_should_be_visible(
    kind: RepairOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    repairing_unit: bool,
) -> bool {
    if stats.destroyed() {
        return false;
    }

    if owner == TeamType::Null {
        return matches!(kind, RepairOverlayKind::SmokeStack);
    }

    match kind {
        RepairOverlayKind::SmokeStack => repairing_unit,
        RepairOverlayKind::FrontLight
        | RepairOverlayKind::SideLight
        | RepairOverlayKind::Bulb
        | RepairOverlayKind::TextBox => true,
    }
}

fn repair_overlay_forced_frame(
    kind: RepairOverlayKind,
    owner: TeamType,
    repairing_unit: bool,
) -> Option<usize> {
    match kind {
        RepairOverlayKind::FrontLight | RepairOverlayKind::SideLight => Some(1),
        RepairOverlayKind::Bulb | RepairOverlayKind::SmokeStack
            if owner == TeamType::Null || !repairing_unit =>
        {
            Some(0)
        }
        _ => None,
    }
}

fn animate_factory_overlays(
    time: Res<Time>,
    mut rng: ResMut<CombatRng>,
    objects: Query<(
        &GameObjectEntity,
        &ObjectTeam,
        &ObjectStats,
        Option<&BuildingProduction>,
    )>,
    mut overlays: Query<(
        &mut FactoryOverlayLayer,
        &mut Transform,
        &mut Sprite,
        &mut Visibility,
    )>,
) {
    let object_states: HashMap<u32, (TeamType, ObjectStats, bool)> = objects
        .iter()
        .map(|(object, team, stats, production)| {
            let production_active = production
                .map(|production| production.status != BuildingProductionStatus::Select)
                .unwrap_or(false);
            (object.ref_id, (team.0, *stats, production_active))
        })
        .collect();
    let mut robot_light_updates = HashMap::new();

    for (mut overlay, mut transform, mut sprite, mut visibility) in &mut overlays {
        let (owner, stats, production_active) =
            object_states.get(&overlay.ref_id).copied().unwrap_or((
                TeamType::Null,
                ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 0),
                false,
            ));

        advance_factory_overlay_frame(
            &mut overlay,
            time.delta_secs(),
            &mut rng,
            &mut robot_light_updates,
        );
        apply_factory_overlay_frame(&overlay, &mut transform, &mut sprite);

        let visible = factory_overlay_should_be_visible(
            overlay.kind,
            owner,
            stats,
            production_active,
            overlay.current,
        );
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn advance_factory_overlay_frame(
    overlay: &mut FactoryOverlayLayer,
    delta_secs: f32,
    rng: &mut CombatRng,
    robot_light_updates: &mut HashMap<u32, Option<[usize; 3]>>,
) {
    if overlay.frames.len() <= 1 {
        overlay.current = 0;
        return;
    }

    overlay.elapsed += delta_secs;
    if overlay.elapsed >= overlay.frame_time {
        overlay.elapsed %= overlay.frame_time;
        overlay.current = factory_overlay_next_frame(
            overlay.ref_id,
            overlay.kind,
            overlay.current,
            overlay.frames.len(),
            rng,
            robot_light_updates,
        );
    }
}

fn factory_overlay_next_frame(
    ref_id: u32,
    kind: FactoryOverlayKind,
    current: usize,
    frame_count: usize,
    rng: &mut CombatRng,
    robot_light_updates: &mut HashMap<u32, Option<[usize; 3]>>,
) -> usize {
    if let Some(light_index) = factory_overlay_robot_single_light_index(kind) {
        let update = *robot_light_updates
            .entry(ref_id)
            .or_insert_with(|| factory_robot_single_light_update(rng));
        return update.map(|states| states[light_index]).unwrap_or(current);
    }

    (current + 1) % frame_count
}

fn factory_robot_single_light_update(rng: &mut CombatRng) -> Option<[usize; 3]> {
    (rng.index(3) == 0).then(|| [rng.index(2), rng.index(2), rng.index(2)])
}

fn apply_factory_overlay_frame(
    overlay: &FactoryOverlayLayer,
    transform: &mut Transform,
    sprite: &mut Sprite,
) {
    let Some(frame) = overlay.frames.get(overlay.current) else {
        return;
    };

    sprite.image = frame.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: frame.layout.clone(),
        index: frame.index,
    });
    let world = radar_overlay_world_position(overlay.top_left_map, frame);
    transform.translation.x = world.x;
    transform.translation.y = world.y;
}

fn factory_overlay_should_be_visible(
    kind: FactoryOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    production_active: bool,
    current: usize,
) -> bool {
    if stats.destroyed() {
        return false;
    }

    if owner == TeamType::Null || !production_active {
        return matches!(
            kind,
            FactoryOverlayKind::RobotBody | FactoryOverlayKind::VehicleTank
        );
    }

    if factory_overlay_is_robot_single_light(kind) && current == 0 {
        return false;
    }

    true
}

fn factory_overlay_is_robot_single_light(kind: FactoryOverlayKind) -> bool {
    factory_overlay_robot_single_light_index(kind).is_some()
}

fn factory_overlay_robot_single_light_index(kind: FactoryOverlayKind) -> Option<usize> {
    match kind {
        FactoryOverlayKind::RobotSingleLight0 => Some(0),
        FactoryOverlayKind::RobotSingleLight1 => Some(1),
        FactoryOverlayKind::RobotSingleLight2 => Some(2),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::settings::{GRENADE_BOX_EXPLOSION_DELAY, GRENADE_BOX_SCATTER_HALF_EXTENT};

    fn test_tile_info(is_road: bool) -> original::tileinfo::PaletteTileInfo {
        original::tileinfo::PaletteTileInfo {
            is_water: false,
            is_passable: true,
            is_usable: false,
            is_road,
            is_effect: false,
            is_water_effect: false,
            next_tile_in_effect: 0,
            takes_tank_tracks: !is_road,
            crater_type: 0,
            is_starter_tile: false,
        }
    }

    fn tiny_tile_map(tile_ids: Vec<u16>, planet: PlanetType) -> ZMap {
        ZMap {
            basics: original::map::MapBasics {
                width: 2,
                height: 2,
                map_name: "tiny".to_string(),
                player_count: 2,
                object_count: 0,
                terrain_type: planet,
                zone_count: 0,
            },
            zones: Vec::new(),
            objects: Vec::new(),
            tiles: tile_ids
                .into_iter()
                .map(|tile| original::map::MapTile { tile })
                .collect(),
        }
    }

    fn test_ambient_bird(position_map: Vec2, map_size: Vec2) -> AmbientBird {
        AmbientBird {
            planet: PlanetType::Desert,
            map_size,
            position_map,
            fractional_shift: Vec2::ZERO,
            angle_degrees: 0.0,
            dangle: 0.0,
            rise: 1.0,
            render_frame: 0,
            speed: 20.0,
            next_render_time: 999.0,
            last_process_time: 0.0,
            next_dangle_time: 999.0,
            next_caw_sound_time: 999.0,
            next_height_change_time: 999.0,
            rise_change_end: 0.0,
            rise_change_start_time: 0.0,
            rise_change_target: 1.0,
            rise_change_start: 1.0,
        }
    }

    fn fort_warning_snapshot(
        ref_id: u32,
        kind: ObjectKind,
        position: Vec2,
        team: TeamType,
        destroyed: bool,
    ) -> FortWarningSnapshot {
        FortWarningSnapshot {
            ref_id,
            kind,
            position,
            team,
            destroyed,
        }
    }

    fn losing_warning_snapshot(
        kind: ObjectKind,
        team: TeamType,
        destroyed: bool,
    ) -> LosingWarningSnapshot {
        LosingWarningSnapshot {
            kind,
            team,
            destroyed,
        }
    }

    fn test_zone_ownership(owners: &[TeamType]) -> ZoneOwnership {
        ZoneOwnership {
            owners: owners.to_vec(),
            links: owners
                .iter()
                .enumerate()
                .map(|(zone_index, _)| ZoneLink {
                    zone_index,
                    flag_ref_id: zone_index as u32 + 1,
                    building_refs: Vec::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn ambient_bird_count_matches_original_density() {
        assert_eq!(ambient_bird_count(64, 64), 6);
        assert_eq!(ambient_bird_count(25, 25), 0);
        assert_eq!(ambient_bird_count(50, 26), 2);
    }

    #[test]
    fn ambient_bird_points_towards_map_center_on_reset_side() {
        let map_size = Vec2::new(320.0, 320.0);

        assert_eq!(
            ambient_bird_angle_to_center(Vec2::new(-20.0, 160.0), map_size),
            0.0
        );
        assert_eq!(
            ambient_bird_angle_to_center(Vec2::new(340.0, 160.0), map_size),
            180.0
        );
        assert_eq!(
            ambient_bird_angle_to_center(Vec2::new(160.0, -20.0), map_size),
            270.0
        );
        assert_eq!(
            ambient_bird_angle_to_center(Vec2::new(160.0, 340.0), map_size),
            90.0
        );
    }

    #[test]
    fn ambient_bird_frame_interval_and_paths_match_original_assets() {
        assert_eq!(
            ambient_bird_frame_interval(PlanetType::City),
            BIRD_CITY_FRAME_TIME
        );
        assert_eq!(
            ambient_bird_frame_interval(PlanetType::Desert),
            BIRD_DEFAULT_FRAME_TIME
        );
        assert_eq!(
            ambient_bird_frame_path(PlanetType::Volcanic, 7),
            "other/birds/bird_volcanic_r000_n02.png"
        );
    }

    #[test]
    fn ambient_bird_movement_preserves_fractional_shift_like_original() {
        let mut bird = test_ambient_bird(Vec2::ZERO, Vec2::new(640.0, 640.0));
        let mut rng = CombatRng::default();

        let sound = advance_ambient_bird_state(&mut bird, 0.025, &mut rng);
        assert_eq!(sound, None);
        assert_eq!(bird.position_map, Vec2::ZERO);
        assert_eq!(bird.fractional_shift, Vec2::new(0.5, 0.0));

        let sound = advance_ambient_bird_state(&mut bird, 0.05, &mut rng);
        assert_eq!(sound, None);
        assert_eq!(bird.position_map, Vec2::new(1.0, 0.0));
        assert_eq!(bird.fractional_shift, Vec2::ZERO);
    }

    #[test]
    fn ambient_bird_sound_assets_match_original_planet_rules() {
        assert_eq!(
            ambient_bird_caw_sound(PlanetType::City),
            AmbientBirdSoundKind::BatChirp
        );
        assert_eq!(
            ambient_bird_sound_asset_path(AmbientBirdSoundKind::BatChirp),
            "sounds/BATCHIRP.wav"
        );

        for planet in [
            PlanetType::Desert,
            PlanetType::Volcanic,
            PlanetType::Arctic,
            PlanetType::Jungle,
        ] {
            assert_eq!(ambient_bird_caw_sound(planet), AmbientBirdSoundKind::Crow);
        }
        assert_eq!(
            ambient_bird_sound_asset_path(AmbientBirdSoundKind::Crow),
            "sounds/CROW2.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::AmbientBird(AmbientBirdSoundKind::Crow), None),
            "sounds/CROW2.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::RifleFire, None),
            "sounds/RIFLE3.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::PsychoFire, None),
            "sounds/MACHGUN2.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::ToughFire, None),
            "sounds/MOBIMISS.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::PyroFire, None),
            "sounds/FLAMER.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::LaserFire, None),
            "sounds/LASERGUN.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::GunFire, None),
            "sounds/LTGUN.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::GatlingFire, None),
            "sounds/GATTGUN.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::JeepFire, None),
            "sounds/JEEPMGUN.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::LightFire, None),
            "sounds/LTANKGUN.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::MediumFire, None),
            "sounds/MTANKGUN.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::HeavyFire, None),
            "sounds/HTANKGUN.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::MobileMissileFire, None),
            "sounds/MOBIMIS2.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::Ricochet, None),
            "sounds/RICOCH1.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::TurrentExplosion, None),
            "sounds/METGRND.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::ThrowGrenade, None),
            "sounds/GRENLOBX.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::ComputerFortUnderAttack, None),
            "sounds/comp_fort_under_attack.wav"
        );
        assert_eq!(
            game_sound_asset_path(GameSoundKind::ComputerYouAreLosing, None),
            "sounds/comp_youre_losing_00.wav"
        );
        let mut losing_rng = CombatRng::default();
        let losing_path =
            game_sound_asset_path(GameSoundKind::ComputerYouAreLosing, Some(&mut losing_rng));
        assert!(losing_path.starts_with("sounds/comp_youre_losing_"));
        assert!(losing_path.ends_with(".wav"));
        let mut rng = CombatRng::default();
        let random_path = game_sound_asset_path(GameSoundKind::RandomExplosion, Some(&mut rng));
        assert!(random_path.starts_with("sounds/explosion_"));
        assert!(random_path.ends_with(".wav"));
    }

    #[test]
    fn losing_warning_standings_count_original_unit_families_and_zone_percentages() {
        let zones = test_zone_ownership(&[
            TeamType::Red,
            TeamType::Blue,
            TeamType::Blue,
            TeamType::Green,
        ]);
        let snapshots = [
            losing_warning_snapshot(ObjectKind::Robot(RobotType::Grunt), TeamType::Red, false),
            losing_warning_snapshot(ObjectKind::Vehicle(VehicleType::Jeep), TeamType::Red, false),
            losing_warning_snapshot(ObjectKind::Cannon(CannonType::Gun), TeamType::Blue, false),
            losing_warning_snapshot(
                ObjectKind::Building(BuildingType::Radar),
                TeamType::Blue,
                false,
            ),
            losing_warning_snapshot(ObjectKind::Robot(RobotType::Psycho), TeamType::Blue, true),
            losing_warning_snapshot(
                ObjectKind::MapItem(ItemType::Flag as u8),
                TeamType::Green,
                false,
            ),
        ];

        let standings = losing_warning_team_standings(&snapshots, &zones);

        assert_eq!(standings[team_index(TeamType::Red)].units_available, 2);
        assert_eq!(standings[team_index(TeamType::Blue)].units_available, 1);
        assert_eq!(standings[team_index(TeamType::Green)].units_available, 0);
        assert!((standings[team_index(TeamType::Red)].zone_percentage - 0.25).abs() < f32::EPSILON);
        assert!((standings[team_index(TeamType::Blue)].zone_percentage - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn losing_warning_uses_original_strict_factor_and_truncated_unit_scale() {
        let mut standings = [TeamStanding::default(); TEAM_TYPE_COUNT];
        standings[team_index(TeamType::Red)] = TeamStanding {
            units_available: 1,
            zone_percentage: 0.20,
        };
        standings[team_index(TeamType::Blue)] = TeamStanding {
            units_available: 2,
            zone_percentage: 0.35,
        };

        assert_eq!(losing_scaled_unit_count(1), 1);
        assert!(losing_warning_should_play(&standings, TeamType::Red));

        standings[team_index(TeamType::Blue)].zone_percentage = 0.34;
        assert!(!losing_warning_should_play(&standings, TeamType::Red));

        standings[team_index(TeamType::Blue)] = TeamStanding {
            units_available: 1,
            zone_percentage: 0.50,
        };
        assert!(!losing_warning_should_play(&standings, TeamType::Red));
    }

    #[test]
    fn losing_warning_uses_eight_second_gate_like_original() {
        let mut warning = LosingVerbalWarning::default();
        let mut standings = [TeamStanding::default(); TEAM_TYPE_COUNT];
        standings[team_index(TeamType::Red)] = TeamStanding {
            units_available: 1,
            zone_percentage: 0.20,
        };
        standings[team_index(TeamType::Blue)] = TeamStanding {
            units_available: 2,
            zone_percentage: 0.50,
        };

        assert!(trigger_losing_verbal_warning(
            &mut warning,
            &standings,
            TeamType::Red
        ));
        assert_eq!(warning.cooldown_remaining, LOSING_VERBAL_WARNING_COOLDOWN);
        assert!(!trigger_losing_verbal_warning(
            &mut warning,
            &standings,
            TeamType::Red
        ));

        warning.cooldown_remaining = 0.0;
        standings[team_index(TeamType::Blue)].units_available = 0;
        assert!(!trigger_losing_verbal_warning(
            &mut warning,
            &standings,
            TeamType::Red
        ));
    }

    #[test]
    fn fort_under_attack_warns_for_enemy_non_building_within_original_distance() {
        let snapshots = [
            fort_warning_snapshot(
                10,
                ObjectKind::Building(BuildingType::FortFront),
                Vec2::ZERO,
                TeamType::Red,
                false,
            ),
            fort_warning_snapshot(
                20,
                ObjectKind::Robot(RobotType::Grunt),
                Vec2::new(FORT_UNDER_ATTACK_DISTANCE, 0.0),
                TeamType::Blue,
                false,
            ),
        ];

        assert_eq!(
            fort_under_attack_target(&snapshots, TeamType::Red),
            Some(10)
        );
    }

    #[test]
    fn fort_under_attack_ignores_excluded_or_non_danger_objects() {
        let fort = fort_warning_snapshot(
            10,
            ObjectKind::Building(BuildingType::FortBack),
            Vec2::ZERO,
            TeamType::Red,
            false,
        );

        for ignored in [
            fort_warning_snapshot(
                20,
                ObjectKind::Robot(RobotType::Grunt),
                Vec2::new(FORT_UNDER_ATTACK_DISTANCE + 0.1, 0.0),
                TeamType::Blue,
                false,
            ),
            fort_warning_snapshot(
                21,
                ObjectKind::Building(BuildingType::RobotFactory),
                Vec2::new(10.0, 0.0),
                TeamType::Blue,
                false,
            ),
            fort_warning_snapshot(
                22,
                ObjectKind::MapItem(ItemType::Flag as u8),
                Vec2::new(10.0, 0.0),
                TeamType::Blue,
                false,
            ),
            fort_warning_snapshot(
                23,
                ObjectKind::Robot(RobotType::Grunt),
                Vec2::new(10.0, 0.0),
                TeamType::Null,
                false,
            ),
            fort_warning_snapshot(
                24,
                ObjectKind::Robot(RobotType::Grunt),
                Vec2::new(10.0, 0.0),
                TeamType::Blue,
                true,
            ),
        ] {
            assert_eq!(
                fort_under_attack_target(&[fort, ignored], TeamType::Red),
                None
            );
        }

        let destroyed_fort = fort_warning_snapshot(
            11,
            ObjectKind::Building(BuildingType::FortFront),
            Vec2::ZERO,
            TeamType::Red,
            true,
        );
        let enemy = fort_warning_snapshot(
            25,
            ObjectKind::Robot(RobotType::Grunt),
            Vec2::new(10.0, 0.0),
            TeamType::Blue,
            false,
        );

        assert_eq!(
            fort_under_attack_target(&[destroyed_fort, enemy], TeamType::Red),
            None
        );
    }

    #[test]
    fn fort_under_attack_verbal_warning_uses_ten_second_gate() {
        let mut warning = FortUnderAttackWarning::default();

        assert!(trigger_fort_under_attack_warning(&mut warning, 10));
        assert_eq!(warning.message.target_ref_id, Some(10));
        assert_eq!(
            warning.verbal_cooldown_remaining,
            FORT_UNDER_ATTACK_VERBAL_COOLDOWN
        );

        assert!(!trigger_fort_under_attack_warning(&mut warning, 11));
        assert_eq!(warning.message.target_ref_id, Some(10));

        warning.verbal_cooldown_remaining = 0.0;
        assert!(trigger_fort_under_attack_warning(&mut warning, 11));
        assert_eq!(warning.message.target_ref_id, Some(11));
    }

    #[test]
    fn ambient_bird_caw_request_matches_original_chance_and_position() {
        let mut no_sound = CombatRng::default();
        assert_eq!(
            ambient_bird_caw_request(PlanetType::City, Vec2::new(12.0, 34.0), &mut no_sound),
            None
        );

        let mut sound_rng = CombatRng(3);
        assert_eq!(
            ambient_bird_caw_request(PlanetType::City, Vec2::new(12.0, 34.0), &mut sound_rng),
            Some(AmbientBirdSoundRequest {
                kind: AmbientBirdSoundKind::BatChirp,
                position_map: Vec2::new(12.0, 34.0),
                restricted_size: BIRD_SOUND_SIZE,
            })
        );
    }

    #[test]
    fn ambient_bird_state_emits_caw_request_after_timer_like_original() {
        let mut bird = test_ambient_bird(Vec2::new(12.0, 34.0), Vec2::new(640.0, 640.0));
        bird.planet = PlanetType::City;
        bird.next_caw_sound_time = 1.0;
        bird.speed = 0.0;
        let mut rng = CombatRng(4);

        let sound = advance_ambient_bird_state(&mut bird, 1.0, &mut rng);
        assert_eq!(
            sound,
            Some(AmbientBirdSoundRequest {
                kind: AmbientBirdSoundKind::BatChirp,
                position_map: Vec2::new(12.0, 34.0),
                restricted_size: BIRD_SOUND_SIZE,
            })
        );
        assert!(bird.next_caw_sound_time >= 16.0);
        assert!(bird.next_caw_sound_time < 21.0);
    }

    #[test]
    fn ambient_bird_sound_view_matches_original_within_view_edges() {
        let view = MapViewRect {
            top_left: Vec2::new(100.0, 200.0),
            size: Vec2::new(300.0, 400.0),
        };

        assert!(map_rect_intersects_view(
            Vec2::new(400.0, 600.0),
            BIRD_SOUND_SIZE,
            view
        ));
        assert!(map_rect_intersects_view(
            Vec2::new(84.0, 184.0),
            BIRD_SOUND_SIZE,
            view
        ));
        assert!(!map_rect_intersects_view(
            Vec2::new(400.1, 600.0),
            BIRD_SOUND_SIZE,
            view
        ));
        assert!(!map_rect_intersects_view(
            Vec2::new(83.9, 184.0),
            BIRD_SOUND_SIZE,
            view
        ));
    }

    #[test]
    fn ambient_bird_resets_when_it_leaves_original_padding() {
        let map_size = Vec2::new(320.0, 320.0);
        assert!(!ambient_bird_out_of_bounds(
            Vec2::new(-160.0, 100.0),
            map_size
        ));
        assert!(ambient_bird_out_of_bounds(
            Vec2::new(-161.0, 100.0),
            map_size
        ));
        assert!(ambient_bird_out_of_bounds(
            Vec2::new(100.0, 481.0),
            map_size
        ));
    }

    #[test]
    fn radar_overlay_visibility_matches_original_owner_and_destroyed_rules() {
        let live = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 100);
        let destroyed = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 0);

        assert!(radar_overlay_should_be_visible(
            RadarOverlayKind::FrontLight,
            TeamType::Null,
            live
        ));
        assert!(!radar_overlay_should_be_visible(
            RadarOverlayKind::Dish,
            TeamType::Null,
            live
        ));
        assert!(radar_overlay_should_be_visible(
            RadarOverlayKind::BoxSpinner,
            TeamType::Red,
            live
        ));
        assert!(!radar_overlay_should_be_visible(
            RadarOverlayKind::FrontLight,
            TeamType::Red,
            destroyed
        ));
    }

    #[test]
    fn repair_overlay_visibility_matches_original_owner_destroyed_and_busy_rules() {
        let live = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Repair), 100);
        let destroyed = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Repair), 0);

        assert!(repair_overlay_should_be_visible(
            RepairOverlayKind::SmokeStack,
            TeamType::Null,
            live,
            false
        ));
        assert!(!repair_overlay_should_be_visible(
            RepairOverlayKind::TextBox,
            TeamType::Null,
            live,
            false
        ));
        assert!(repair_overlay_should_be_visible(
            RepairOverlayKind::TextBox,
            TeamType::Red,
            live,
            false
        ));
        assert!(!repair_overlay_should_be_visible(
            RepairOverlayKind::SmokeStack,
            TeamType::Red,
            live,
            false
        ));
        assert!(repair_overlay_should_be_visible(
            RepairOverlayKind::SmokeStack,
            TeamType::Red,
            live,
            true
        ));
        assert!(!repair_overlay_should_be_visible(
            RepairOverlayKind::FrontLight,
            TeamType::Red,
            destroyed,
            true
        ));
    }

    #[test]
    fn repair_overlay_forced_frames_match_original_after_effects() {
        assert_eq!(
            repair_overlay_forced_frame(RepairOverlayKind::FrontLight, TeamType::Red, true),
            Some(1)
        );
        assert_eq!(
            repair_overlay_forced_frame(RepairOverlayKind::SideLight, TeamType::Red, false),
            Some(1)
        );
        assert_eq!(
            repair_overlay_forced_frame(RepairOverlayKind::Bulb, TeamType::Red, false),
            Some(0)
        );
        assert_eq!(
            repair_overlay_forced_frame(RepairOverlayKind::SmokeStack, TeamType::Null, false),
            Some(0)
        );
        assert_eq!(
            repair_overlay_forced_frame(RepairOverlayKind::TextBox, TeamType::Red, false),
            None
        );
        assert_eq!(
            repair_overlay_forced_frame(RepairOverlayKind::Bulb, TeamType::Red, true),
            None
        );
    }

    #[test]
    fn factory_overlay_visibility_matches_original_owner_building_and_destroyed_rules() {
        let live_robot =
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let destroyed_robot =
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 0);

        assert!(factory_overlay_should_be_visible(
            FactoryOverlayKind::RobotBody,
            TeamType::Null,
            live_robot,
            false,
            0
        ));
        assert!(!factory_overlay_should_be_visible(
            FactoryOverlayKind::RobotSpin,
            TeamType::Null,
            live_robot,
            false,
            0
        ));
        assert!(factory_overlay_should_be_visible(
            FactoryOverlayKind::RobotSpin,
            TeamType::Red,
            live_robot,
            true,
            0
        ));
        assert!(!factory_overlay_should_be_visible(
            FactoryOverlayKind::RobotBody,
            TeamType::Red,
            destroyed_robot,
            true,
            0
        ));
        assert!(!factory_overlay_should_be_visible(
            FactoryOverlayKind::RobotSingleLight0,
            TeamType::Red,
            live_robot,
            true,
            0
        ));
        assert!(factory_overlay_should_be_visible(
            FactoryOverlayKind::RobotSingleLight0,
            TeamType::Red,
            live_robot,
            true,
            1
        ));

        let live_vehicle =
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::VehicleFactory), 100);
        assert!(factory_overlay_should_be_visible(
            FactoryOverlayKind::VehicleTank,
            TeamType::Red,
            live_vehicle,
            false,
            0
        ));
        assert!(!factory_overlay_should_be_visible(
            FactoryOverlayKind::VehicleVent,
            TeamType::Red,
            live_vehicle,
            false,
            0
        ));
        assert!(factory_overlay_should_be_visible(
            FactoryOverlayKind::VehicleVent,
            TeamType::Red,
            live_vehicle,
            true,
            0
        ));
    }

    #[test]
    fn factory_robot_single_lights_use_original_random_on_skip_behavior() {
        let mut no_reroll = CombatRng::default();
        let mut no_updates = HashMap::new();
        assert_eq!(
            factory_overlay_next_frame(
                10,
                FactoryOverlayKind::RobotSingleLight0,
                0,
                2,
                &mut no_reroll,
                &mut no_updates,
            ),
            0
        );
        assert_eq!(
            factory_overlay_next_frame(
                10,
                FactoryOverlayKind::RobotSingleLight1,
                1,
                2,
                &mut no_reroll,
                &mut no_updates,
            ),
            1
        );

        let mut reroll_on = CombatRng(3);
        let mut light_updates = HashMap::new();
        assert_eq!(
            factory_overlay_next_frame(
                20,
                FactoryOverlayKind::RobotSingleLight0,
                0,
                2,
                &mut reroll_on,
                &mut light_updates,
            ),
            1
        );
        assert_eq!(
            factory_overlay_next_frame(
                20,
                FactoryOverlayKind::RobotSingleLight1,
                0,
                2,
                &mut reroll_on,
                &mut light_updates,
            ),
            1
        );
        assert_eq!(
            factory_overlay_next_frame(
                20,
                FactoryOverlayKind::RobotSingleLight2,
                0,
                2,
                &mut reroll_on,
                &mut light_updates,
            ),
            1
        );

        let mut non_light = CombatRng::default();
        let mut non_light_updates = HashMap::new();
        assert_eq!(
            factory_overlay_next_frame(
                30,
                FactoryOverlayKind::RobotBody,
                1,
                2,
                &mut non_light,
                &mut non_light_updates,
            ),
            0
        );
    }

    #[test]
    fn passability_route_goes_around_blocked_tiles() {
        let grid = PassabilityGrid {
            width: 3,
            height: 3,
            walkable: vec![true, true, true, true, false, true, true, true, true],
            vehicle_walkable: vec![true, true, true, true, false, true, true, true, true],
            walk_speed: vec![1.0; 9],
        };

        let route = grid
            .route(
                grid.tile_center(IVec2::new(0, 1)),
                grid.tile_center(IVec2::new(2, 1)),
            )
            .expect("route around center blocker");

        assert!(!route.contains(&grid.tile_center(IVec2::new(1, 1))));
        assert_eq!(
            route.last().copied(),
            Some(grid.tile_center(IVec2::new(2, 1)))
        );
    }

    #[test]
    fn attack_range_route_stops_before_blocked_target_tile() {
        let grid = PassabilityGrid {
            width: 5,
            height: 3,
            walkable: vec![
                true, true, true, true, true, true, true, true, true, false, true, true, true,
                true, true,
            ],
            vehicle_walkable: vec![
                true, true, true, true, true, true, true, true, true, false, true, true, true,
                true, true,
            ],
            walk_speed: vec![1.0; 15],
        };

        let route = grid
            .route_to_attack_range(
                grid.tile_center(IVec2::new(0, 1)),
                grid.tile_center(IVec2::new(4, 1)),
                TILE_SIZE + 1.0,
            )
            .expect("route to tile next to blocked target");

        assert_eq!(
            route.last().copied(),
            Some(grid.tile_center(IVec2::new(3, 1)))
        );
    }

    #[test]
    fn attack_filter_matches_original_team_explosive_and_range_rules() {
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let tough = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Tough), 100);
        let enemy_jeep = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let building = ObjectStats::from_kind(
            ObjectKind::Building(original::objects::BuildingType::Radar),
            100,
        );

        assert!(!can_attack_target(
            TeamType::Red,
            grunt,
            TeamType::Red,
            enemy_jeep,
            20.0
        ));
        assert!(!can_attack_target(
            TeamType::Red,
            grunt,
            TeamType::Blue,
            building,
            20.0
        ));
        assert!(can_attack_target(
            TeamType::Red,
            tough,
            TeamType::Blue,
            building,
            20.0
        ));
        assert!(!can_attack_target(
            TeamType::Red,
            tough,
            TeamType::Blue,
            building,
            tough.attack_radius + 1.0
        ));
    }

    #[test]
    fn grenade_attack_makes_robot_count_as_explosive() {
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let building = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 100);

        assert!(!can_attack_target_identity(
            TeamType::Red,
            grunt,
            0,
            TeamType::Blue,
            building
        ));
        assert!(can_attack_target_identity(
            TeamType::Red,
            grunt,
            1,
            TeamType::Blue,
            building
        ));
    }

    #[test]
    fn direct_fire_snipe_rules_match_driver_gate() {
        let sniper_stats = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Sniper), 100);
        let driver = DriverHealth::new(RobotType::Grunt, 50.0);

        assert!(should_snipe_driver(
            ObjectKind::Robot(RobotType::Sniper),
            sniper_stats,
            ObjectKind::Vehicle(VehicleType::Jeep),
            &driver,
            true,
            0.5
        ));
        assert!(!should_snipe_driver(
            ObjectKind::Robot(RobotType::Sniper),
            sniper_stats,
            ObjectKind::Vehicle(VehicleType::Jeep),
            &driver,
            true,
            0.95
        ));
        assert!(!should_snipe_driver(
            ObjectKind::Robot(RobotType::Tough),
            ObjectStats::from_kind(ObjectKind::Robot(RobotType::Tough), 100),
            ObjectKind::Vehicle(VehicleType::Jeep),
            &driver,
            true,
            0.0
        ));
    }

    #[test]
    fn apc_driver_count_multiplies_driver_damage_like_original() {
        let apc = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Apc), 100);
        let driver = DriverHealth::with_driver_healths(RobotType::Grunt, vec![50.0, 45.0, 30.0]);
        let effective =
            effective_attack_stats(ObjectKind::Vehicle(VehicleType::Apc), apc, Some(&driver));
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let attack = attack_delivery(effective, 0);

        assert_eq!(effective.attack_radius, grunt.attack_radius);
        assert_eq!(
            attack.damage
                * driver_attack_damage_multiplier(
                    ObjectKind::Vehicle(VehicleType::Apc),
                    Some(&driver)
                ),
            grunt.attack_damage * 3.0
        );
    }

    #[test]
    fn ejected_driver_health_converts_to_robot_health_percent() {
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);

        assert_eq!(
            driver_health_percent(ObjectKind::Robot(RobotType::Grunt), grunt.max_health),
            100
        );
        assert_eq!(
            driver_health_percent(ObjectKind::Robot(RobotType::Grunt), grunt.max_health * 0.5),
            50
        );
        assert_eq!(
            driver_health_percent(ObjectKind::Robot(RobotType::Grunt), -1.0),
            0
        );
    }

    #[test]
    fn ejectable_driver_objects_match_original_apc_and_cannon_scope() {
        assert!(can_eject_drivers(
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Apc), 100),
        ));
        assert!(can_eject_drivers(
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gatling), 100),
        ));
        let mut fort_turret = ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gatling), 100);
        fort_turret.cannon_ejectable = false;
        assert!(!can_eject_drivers(
            ObjectKind::Cannon(CannonType::Gatling),
            fort_turret,
        ));
        assert!(!can_eject_drivers(
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100),
        ));
    }

    #[test]
    fn eject_clears_apc_driver_attack_stats_to_base_values() {
        let mut apc = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Apc), 100);
        let driver = DriverHealth::new(RobotType::Grunt, 50.0);
        apc = effective_attack_stats(ObjectKind::Vehicle(VehicleType::Apc), apc, Some(&driver));
        assert!(apc.can_attack());

        clear_driver_attack_stats(ObjectKind::Vehicle(VehicleType::Apc), &mut apc);

        assert_eq!(
            apc.attack_damage,
            ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Apc), 100).attack_damage
        );
        assert!(!apc.can_attack());
    }

    #[test]
    fn lid_vehicles_are_snipable_only_while_attacking_snipe_capable_targets() {
        let heavy = CombatObjectSnapshot {
            ref_id: 2,
            kind: ObjectKind::Vehicle(VehicleType::Heavy),
            position: Vec2::ZERO,
            size: Vec2::splat(32.0),
            team: TeamType::Blue,
            stats: ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Heavy), 100),
            attack_target_ref: None,
        };
        let sniper = CombatObjectSnapshot {
            ref_id: 3,
            kind: ObjectKind::Robot(RobotType::Sniper),
            position: Vec2::ZERO,
            size: Vec2::splat(14.0),
            team: TeamType::Red,
            stats: ObjectStats::from_kind(ObjectKind::Robot(RobotType::Sniper), 100),
            attack_target_ref: None,
        };

        assert!(!target_can_be_sniped(heavy, &[heavy, sniper]));

        let open_heavy = CombatObjectSnapshot {
            attack_target_ref: Some(3),
            ..heavy
        };
        assert!(target_can_be_sniped(open_heavy, &[open_heavy, sniper]));
    }

    #[test]
    fn grenade_attack_delivery_matches_original_settings() {
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let delivery = attack_delivery(grunt, 3);

        assert_eq!(delivery.damage, GRENADE_DAMAGE);
        assert_eq!(delivery.radius, GRENADE_DAMAGE_RADIUS);
        assert_eq!(delivery.missile_speed, GRENADE_MISSILE_SPEED);
        assert_eq!(delivery.cooldown, GRENADE_ATTACK_SPEED);
        assert_eq!(delivery.scatter_half_extent, GRENADE_SCATTER_HALF_EXTENT);
        assert!(delivery.consumes_grenade);
    }

    #[test]
    fn non_grenade_attack_delivery_keeps_unit_stats() {
        let heavy = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Heavy), 100);
        let delivery = attack_delivery(heavy, 0);

        assert_eq!(delivery.damage, heavy.attack_damage);
        assert_eq!(delivery.radius, heavy.damage_radius);
        assert_eq!(delivery.missile_speed, heavy.missile_speed);
        assert_eq!(delivery.cooldown, heavy.attack_speed);
        assert_eq!(delivery.scatter_half_extent, 16.0);
        assert!(!delivery.consumes_grenade);
    }

    #[test]
    fn attack_sounds_match_original_weapon_families() {
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Grunt), false),
            Some(GameSoundKind::RifleFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Sniper), false),
            Some(GameSoundKind::RifleFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Psycho), false),
            Some(GameSoundKind::PsychoFire)
        );
        assert_eq!(
            attack_sound_for_attack(
                ObjectKind::Vehicle(VehicleType::Apc),
                ObjectKind::Robot(RobotType::Psycho),
                false
            ),
            Some(GameSoundKind::RifleFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Tough), false),
            Some(GameSoundKind::ToughFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Pyro), false),
            Some(GameSoundKind::PyroFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Laser), false),
            Some(GameSoundKind::LaserFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Vehicle(VehicleType::Jeep), false),
            Some(GameSoundKind::JeepFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Vehicle(VehicleType::Light), false),
            Some(GameSoundKind::LightFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Vehicle(VehicleType::Medium), false),
            Some(GameSoundKind::MediumFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Vehicle(VehicleType::Heavy), false),
            Some(GameSoundKind::HeavyFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Vehicle(VehicleType::MissileLauncher), false),
            Some(GameSoundKind::MobileMissileFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Cannon(CannonType::Gatling), false),
            Some(GameSoundKind::GatlingFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Cannon(CannonType::Gun), false),
            Some(GameSoundKind::GunFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Cannon(CannonType::Howitzer), false),
            Some(GameSoundKind::HeavyFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Cannon(CannonType::MissileCannon), false),
            Some(GameSoundKind::MobileMissileFire)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Robot(RobotType::Grunt), true),
            Some(GameSoundKind::ThrowGrenade)
        );
        assert_eq!(
            attack_sound_for_kind(ObjectKind::Vehicle(VehicleType::Crane), false),
            None
        );
        assert_eq!(
            sound_source_top_left_map(Vec2::new(100.0, -50.0), Vec2::new(20.0, 10.0)),
            Vec2::new(90.0, 45.0)
        );
        assert_eq!(
            damage_missile_impact_sound(DamageMissileVisual::MapObjectTurrent(0)),
            Some(GameSoundKind::TurrentExplosion)
        );
        assert_eq!(
            damage_missile_impact_sound(DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 0,
                xx_large: 0
            }),
            Some(GameSoundKind::RandomExplosion)
        );
    }

    #[test]
    fn direct_fire_bullet_matches_original_speed_and_progress() {
        let start = Vec2::ZERO;
        let target = Vec2::new(300.0, 0.0);
        let duration = direct_fire_bullet_duration(start, target);
        assert_eq!(duration, 1.0);

        let bullet = DirectFireBullet {
            start,
            target,
            time_remaining: 0.25,
            total_time: duration,
        };
        assert_eq!(direct_fire_bullet_progress(&bullet), 0.75);

        let mut rng = CombatRng::default();
        for _ in 0..16 {
            assert!(direct_fire_bullet_ricochet_particle_count(&mut rng) < 3);
        }
    }

    #[test]
    fn direct_fire_bullet_uses_original_common_effect_scope() {
        assert!(uses_direct_fire_bullet(ObjectKind::Robot(RobotType::Grunt)));
        assert!(uses_direct_fire_bullet(ObjectKind::Robot(
            RobotType::Psycho
        )));
        assert!(uses_direct_fire_bullet(ObjectKind::Vehicle(
            VehicleType::Jeep
        )));
        assert!(uses_direct_fire_bullet(ObjectKind::Cannon(
            CannonType::Gatling
        )));
        assert!(!uses_direct_fire_bullet(ObjectKind::Robot(
            RobotType::Laser
        )));
        assert!(!uses_direct_fire_bullet(ObjectKind::Robot(RobotType::Pyro)));
    }

    #[test]
    fn special_robot_projectiles_match_original_effect_assets_and_speed() {
        assert_eq!(
            special_projectile_kind_for_attack(ObjectKind::Robot(RobotType::Laser)),
            Some(SpecialProjectileKind::Laser)
        );
        assert_eq!(
            special_projectile_kind_for_attack(ObjectKind::Robot(RobotType::Pyro)),
            Some(SpecialProjectileKind::Flame)
        );
        assert_eq!(
            special_projectile_kind_for_attack(ObjectKind::Robot(RobotType::Grunt)),
            None
        );
        assert_eq!(
            special_projectile_frame_paths(SpecialProjectileKind::Laser),
            vec![
                "units/robots/laser/bullet_n00.png",
                "units/robots/laser/bullet_n01.png",
            ]
        );
        assert_eq!(
            special_projectile_frame_paths(SpecialProjectileKind::Flame),
            vec![
                "units/robots/pyro/bullet_n00.png",
                "units/robots/pyro/bullet_n01.png",
                "units/robots/pyro/bullet_n02.png",
                "units/robots/pyro/bullet_n03.png",
            ]
        );
        assert_eq!(
            special_projectile_duration(Vec2::ZERO, Vec2::new(300.0, 0.0)),
            1.0
        );
    }

    #[test]
    fn pyro_fire_impact_frames_match_original_families() {
        let profile = robots::pyro_fire_impact_profile();
        assert_eq!(profile.frame_counts, [4, 4, 4, 6, 6]);
        assert_eq!(
            robots::pyro_fire_impact_frame_paths(0),
            vec![
                "other/fire/fire0_n00.png",
                "other/fire/fire0_n01.png",
                "other/fire/fire0_n02.png",
                "other/fire/fire0_n03.png",
            ]
        );
        assert_eq!(robots::pyro_fire_impact_frame_paths(3).len(), 6);
        assert_eq!(
            robots::pyro_fire_impact_frame_paths(4).last().unwrap(),
            "other/fire/fire4_n05.png"
        );
        assert_eq!(profile.frame_time, 0.06);
    }

    #[test]
    fn apc_driver_attack_kind_uses_original_driver_weapon_family() {
        let pyro_driver = DriverHealth::new(
            RobotType::Pyro,
            ObjectStats::from_kind(ObjectKind::Robot(RobotType::Pyro), 100).health,
        );

        assert_eq!(
            effective_attack_kind(ObjectKind::Vehicle(VehicleType::Apc), Some(&pyro_driver)),
            ObjectKind::Robot(RobotType::Pyro)
        );
        assert_eq!(
            effective_attack_kind(ObjectKind::Vehicle(VehicleType::Apc), None),
            ObjectKind::Vehicle(VehicleType::Apc)
        );
    }

    #[test]
    fn damage_missile_visuals_match_original_projectile_families() {
        let grenade = AttackDelivery {
            damage: GRENADE_DAMAGE,
            damage_chance: 0.0,
            radius: GRENADE_DAMAGE_RADIUS,
            missile_speed: GRENADE_MISSILE_SPEED,
            cooldown: GRENADE_ATTACK_SPEED,
            scatter_half_extent: GRENADE_SCATTER_HALF_EXTENT,
            consumes_grenade: true,
        };

        assert_eq!(
            damage_missile_visual_for_attack(ObjectKind::Robot(RobotType::Grunt), grenade),
            DamageMissileVisual::Grenade
        );
        assert_eq!(
            damage_missile_visual_for_attack(
                ObjectKind::Vehicle(VehicleType::Light),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Light), 100),
                    0
                )
            ),
            DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 0,
                xx_large: 0
            }
        );
        assert_eq!(
            damage_missile_visual_for_attack(
                ObjectKind::Vehicle(VehicleType::Medium),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Medium), 100),
                    0
                )
            ),
            DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 1,
                xx_large: 0
            }
        );
        assert_eq!(
            damage_missile_visual_for_attack(
                ObjectKind::Vehicle(VehicleType::Heavy),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Heavy), 100),
                    0
                )
            ),
            DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 1,
                xx_large: 1
            }
        );
        assert_eq!(
            damage_missile_visual_for_attack(
                ObjectKind::Cannon(CannonType::Gun),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gun), 100),
                    0
                )
            ),
            DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 1,
                xx_large: 0
            }
        );
        assert_eq!(
            damage_missile_visual_for_attack(
                ObjectKind::Cannon(CannonType::Howitzer),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Howitzer), 100),
                    0
                )
            ),
            DamageMissileVisual::LightRocket {
                extra_small: 1,
                extra_large: 1,
                xx_large: 0
            }
        );
        assert_eq!(
            damage_missile_visual_for_attack(
                ObjectKind::Cannon(CannonType::MissileCannon),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Cannon(CannonType::MissileCannon), 100),
                    0
                )
            ),
            DamageMissileVisual::MissileCannon
        );
        assert_eq!(
            damage_missile_visual_for_attack(
                ObjectKind::Vehicle(VehicleType::MissileLauncher),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::MissileLauncher), 100),
                    0
                )
            ),
            DamageMissileVisual::MissileLauncher
        );
        assert_eq!(
            damage_missile_visual_for_attack(
                ObjectKind::Robot(RobotType::Tough),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Robot(RobotType::Tough), 100),
                    0
                )
            ),
            DamageMissileVisual::ToughRocket
        );
    }

    #[test]
    fn missile_impact_effect_assets_match_original_frame_sets() {
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::Grenade),
            vec![
                "other/grenades/grenade_n00.png",
                "other/grenades/grenade_n01.png",
                "other/grenades/grenade_n02.png",
                "other/grenades/grenade_n03.png",
            ]
        );
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::LightRocket {
                extra_small: 1,
                extra_large: 1,
                xx_large: 0
            }),
            vec!["units/vehicles/light/bullet.png"]
        );
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::MissileCannon),
            vec!["units/cannons/missile_cannon/bullet.png"]
        );
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::MissileLauncher),
            vec!["units/vehicles/missile_launcher/bullet.png"]
        );
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::ToughRocket),
            vec![
                "units/robots/tough/bullet_n00.png",
                "units/robots/tough/bullet_n01.png"
            ]
        );
        assert_eq!(
            damage_missile_frame_paths(DamageMissileVisual::MapObjectTurrent(12)),
            vec!["other/map_items/no_shadow12.png"]
        );
        assert_eq!(robots::tough_mushroom_frame_paths().len(), 12);
        assert_eq!(
            robots::tough_mushroom_frame_paths().last().unwrap(),
            "units/robots/tough/mushroom_n11.png"
        );
        assert_eq!(robots::tough_smoke_frame_paths().len(), 8);
        assert_eq!(
            robots::tough_smoke_frame_paths().last().unwrap(),
            "units/robots/tough/smoke_n07.png"
        );
        assert_eq!(side_explosion_frame_paths().len(), 7);
        assert_eq!(
            side_explosion_frame_paths().last().unwrap(),
            "other/explosions/side_explosion_n06.png"
        );
        assert_eq!(
            light_rocket_init_fire_frame_path(0),
            "units/vehicles/light/initfire_n00.png"
        );
        assert_eq!(
            light_rocket_init_fire_frame_path(3),
            "units/vehicles/light/initfire_n03.png"
        );
        assert_eq!(LIGHT_ROCKET_INIT_FIRE_FRAME_TIME, 0.02);
    }

    #[test]
    fn rocket_impact_profiles_match_original_mushroom_and_particle_counts() {
        assert_eq!(
            rocket_impact_profile(DamageMissileVisual::LightRocket {
                extra_small: 1,
                extra_large: 1,
                xx_large: 0
            }),
            Some(RocketImpactProfile {
                xx_large_mushrooms: 0,
                large_mushrooms: 1,
                small_mushrooms: 1,
                unit_particle_radius: 40.0,
                unit_particle_amount: 12,
            })
        );
        assert_eq!(
            rocket_impact_profile(DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 1,
                xx_large: 1
            }),
            Some(RocketImpactProfile {
                xx_large_mushrooms: 1,
                large_mushrooms: 1,
                small_mushrooms: 0,
                unit_particle_radius: 40.0,
                unit_particle_amount: 14,
            })
        );
        assert_eq!(
            rocket_impact_profile(DamageMissileVisual::MissileCannon),
            Some(RocketImpactProfile {
                xx_large_mushrooms: 0,
                large_mushrooms: 3,
                small_mushrooms: 1,
                unit_particle_radius: 50.0,
                unit_particle_amount: 18,
            })
        );
        assert_eq!(
            rocket_impact_profile(DamageMissileVisual::MissileLauncher),
            Some(RocketImpactProfile {
                xx_large_mushrooms: 3,
                large_mushrooms: 0,
                small_mushrooms: 2,
                unit_particle_radius: 80.0,
                unit_particle_amount: 23,
            })
        );
    }

    #[test]
    fn missile_object_particle_targets_match_original_rect_overlap_filter() {
        let snapshots = vec![
            CombatObjectSnapshot {
                ref_id: 1,
                kind: ObjectKind::Vehicle(VehicleType::Heavy),
                position: map_point_to_world(Vec2::new(30.0, 30.0)),
                size: Vec2::splat(32.0),
                team: TeamType::Blue,
                stats: ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Heavy), 100),
                attack_target_ref: None,
            },
            CombatObjectSnapshot {
                ref_id: 2,
                kind: ObjectKind::Robot(RobotType::Grunt),
                position: map_point_to_world(Vec2::new(90.0, 0.0)),
                size: Vec2::splat(14.0),
                team: TeamType::Green,
                stats: ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100),
                attack_target_ref: None,
            },
            CombatObjectSnapshot {
                ref_id: 3,
                kind: ObjectKind::Building(BuildingType::Radar),
                position: map_point_to_world(Vec2::new(20.0, 20.0)),
                size: Vec2::splat(64.0),
                team: TeamType::Yellow,
                stats: ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 100),
                attack_target_ref: None,
            },
        ];

        assert_eq!(missile_object_particle_radius(50.0), 40.0);
        let targets = missile_object_particle_targets(&snapshots, Vec2::ZERO, 50.0);
        assert_eq!(
            targets,
            vec![MissileObjectParticleTarget {
                kind: ObjectKind::Vehicle(VehicleType::Heavy),
                top_left_map: Vec2::new(14.0, 14.0),
                size: Vec2::splat(32.0),
            }]
        );
    }

    #[test]
    fn missile_object_particle_amount_matches_original_robot_halving() {
        let mut rng = CombatRng::default();
        let vehicle_amount =
            missile_object_particle_amount(ObjectKind::Vehicle(VehicleType::Heavy), 18, &mut rng);
        assert!((14..32).contains(&vehicle_amount));

        let mut rng = CombatRng::default();
        let robot_amount =
            missile_object_particle_amount(ObjectKind::Robot(RobotType::Grunt), 18, &mut rng);
        assert_eq!(robot_amount, vehicle_amount / 2);
    }

    #[test]
    fn rocket_crater_profiles_match_original_chances() {
        assert_eq!(light_rocket_big_crater_chance(0, 0), None,);
        assert_eq!(light_rocket_big_crater_chance(1, 0), Some(0.15),);
        assert_eq!(light_rocket_big_crater_chance(1, 1), Some(0.35),);
        assert_eq!(
            damage_crater_for_attack(
                ObjectKind::Cannon(CannonType::MissileCannon),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Cannon(CannonType::MissileCannon), 100),
                    0
                )
            ),
            Some(DamageCrater {
                is_big: false,
                chance: 1.0,
                big_chance: None,
            })
        );
        assert_eq!(
            damage_crater_for_attack(
                ObjectKind::Vehicle(VehicleType::Heavy),
                attack_delivery(
                    ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Heavy), 100),
                    0
                )
            ),
            Some(DamageCrater {
                is_big: false,
                chance: 0.75,
                big_chance: Some(0.35),
            })
        );
    }

    #[test]
    fn rocket_projectile_rotation_tracks_travel_direction() {
        assert_eq!(
            damage_missile_rotation(
                DamageMissileVisual::LightRocket {
                    extra_small: 0,
                    extra_large: 0,
                    xx_large: 0,
                },
                Vec2::ZERO,
                Vec2::new(10.0, 0.0),
            ),
            Quat::IDENTITY
        );

        let north = damage_missile_rotation(
            DamageMissileVisual::MissileLauncher,
            Vec2::ZERO,
            Vec2::new(0.0, 10.0),
        );
        let expected = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        assert!((north.z - expected.z).abs() < 0.0001);
        assert!((north.w - expected.w).abs() < 0.0001);
    }

    #[test]
    fn tough_rocket_smoke_timing_matches_original_back_offset() {
        let start = Vec2::ZERO;
        let target = Vec2::new(150.0, 0.0);
        let total_time = 1.0;
        let mut cursor = 0.0;

        assert!(
            robots::tough_rocket_smoke_positions(
                start,
                target,
                total_time,
                &mut cursor,
                0.05,
                &[Vec2::ZERO]
            )
            .is_empty()
        );
        let smoke = robots::tough_rocket_smoke_positions(
            start,
            target,
            total_time,
            &mut cursor,
            0.06,
            &[Vec2::ZERO],
        );
        assert_eq!(smoke.len(), 1);
        assert_eq!(cursor, 8.0 / 150.0);
        assert!((smoke[0].x + 6.0).abs() < 0.0001);
        assert_eq!(smoke[0].y, 0.0);
    }

    #[test]
    fn multi_rocket_smoke_positions_apply_original_side_offsets() {
        let start = Vec2::ZERO;
        let target = Vec2::new(150.0, 0.0);
        let total_time = 1.0;
        let mut cursor = 0.0;

        let smoke = robots::tough_rocket_smoke_positions(
            start,
            target,
            total_time,
            &mut cursor,
            0.06,
            &[Vec2::ZERO, Vec2::new(0.0, 8.0)],
        );

        assert_eq!(smoke.len(), 2);
        assert_eq!(cursor, 8.0 / 150.0);
        assert!((smoke[0].x + 6.0).abs() < 0.0001);
        assert_eq!(smoke[0].y, 0.0);
        assert!((smoke[1].x + 6.0).abs() < 0.0001);
        assert_eq!(smoke[1].y, 8.0);
    }

    #[test]
    fn multi_rocket_visual_offsets_match_original_render_shifts() {
        let start = Vec2::ZERO;
        let target = Vec2::new(100.0, 0.0);

        let missile_cannon =
            damage_missile_visual_geometry(DamageMissileVisual::MissileCannon, start, target);
        assert_eq!(missile_cannon.primary_offset, Vec2::new(0.0, -8.0));
        assert_eq!(missile_cannon.replica_offsets, vec![Vec2::ZERO]);
        assert_eq!(
            missile_cannon.smoke_offsets,
            vec![Vec2::new(0.0, -8.0), Vec2::ZERO]
        );

        let missile_launcher =
            damage_missile_visual_geometry(DamageMissileVisual::MissileLauncher, start, target);
        assert_eq!(missile_launcher.primary_offset, Vec2::ZERO);
        assert_eq!(
            missile_launcher.replica_offsets,
            vec![Vec2::new(0.0, 8.0), Vec2::new(0.0, -8.0)]
        );
        assert_eq!(
            missile_launcher.smoke_offsets,
            vec![Vec2::ZERO, Vec2::new(0.0, 8.0), Vec2::new(0.0, -8.0)]
        );
    }

    #[test]
    fn rocket_muzzle_offsets_match_original_direction_tables() {
        assert_eq!(vehicle_rocket_muzzle_offset(0), Vec2::new(21.0, 2.0));
        assert_eq!(vehicle_rocket_muzzle_offset(2), Vec2::new(1.0, 22.0));
        assert_eq!(vehicle_rocket_muzzle_offset(4), Vec2::new(-19.0, 2.0));
        assert_eq!(vehicle_rocket_muzzle_offset(6), Vec2::new(1.0, -18.0));

        assert_eq!(tough_rocket_muzzle_offset(0), Vec2::new(8.0, 0.0));
        assert_eq!(tough_rocket_muzzle_offset(2), Vec2::new(0.0, 8.0));
        assert_eq!(tough_rocket_muzzle_offset(4), Vec2::new(-8.0, 0.0));
        assert_eq!(tough_rocket_muzzle_offset(6), Vec2::new(0.0, -8.0));
    }

    #[test]
    fn damage_missiles_start_from_original_muzzle_family_offsets() {
        assert_eq!(
            damage_missile_start_position(
                DamageMissileVisual::LightRocket {
                    extra_small: 0,
                    extra_large: 0,
                    xx_large: 0,
                },
                Vec2::ZERO,
                Vec2::new(100.0, 0.0),
            ),
            Vec2::new(21.0, 2.0)
        );
        assert_eq!(
            damage_missile_start_position(
                DamageMissileVisual::ToughRocket,
                Vec2::ZERO,
                Vec2::new(0.0, 100.0),
            ),
            Vec2::new(0.0, 8.0)
        );
        assert_eq!(
            damage_missile_start_position(
                DamageMissileVisual::Grenade,
                Vec2::ZERO,
                Vec2::new(100.0, 0.0),
            ),
            Vec2::ZERO
        );
    }

    #[test]
    fn missile_impact_effect_positions_match_original_top_left_math() {
        let center = Vec2::new(100.0, 50.0);

        assert_eq!(
            robots::tough_mushroom_base_top_left(center, 1.5),
            Vec2::new(76.0, 2.0)
        );
        assert_eq!(
            side_explosion_base_top_left_scaled(center, 1.0),
            Vec2::new(84.0, 34.0)
        );
        assert_eq!(
            robots::tough_mushroom_frame_offsets(1.5)[0],
            Vec2::new(0.0, 21.0)
        );
        assert_eq!(
            side_explosion_base_top_left_scaled(center, 1.3),
            Vec2::new(79.2, 29.2)
        );
    }

    #[test]
    fn map_object_turrent_missile_matches_original_ranges() {
        assert_eq!(
            map_object::turrent_object_index(ObjectKind::MapItem(ItemType::MapObjectStart as u8)),
            Some(0)
        );
        assert_eq!(
            map_object::turrent_object_index(ObjectKind::MapItem(
                ItemType::MapObjectStart as u8 + 21
            )),
            Some(21)
        );
        assert_eq!(
            map_object::turrent_object_index(ObjectKind::MapItem(
                ItemType::MapObjectStart as u8 + 99
            )),
            Some(21)
        );
        assert_eq!(
            map_object::turrent_object_index(ObjectKind::MapItem(ItemType::Hut as u8)),
            None
        );
        assert_eq!(map_object::turrent_image_height(0), 32.0);
        assert_eq!(map_object::turrent_image_height(5), 16.0);
        assert_eq!(map_object::turrent_image_height(10), 16.0);
        assert_eq!(map_object::turrent_image_height(99), 32.0);
        assert_eq!(map_object::turrent_visual_offset(0), Vec2::new(0.0, 16.0));
        assert_eq!(map_object::turrent_visual_offset(5), Vec2::ZERO);

        let mut rng = CombatRng::default();
        let top_left = Vec2::new(100.0, 50.0);
        for _ in 0..32 {
            let target = map_object::turrent_target(top_left, &mut rng);
            let offset = target - (top_left + Vec2::splat(16.0));
            assert!(
                offset.x > -map_object::TURRENT_MAX_DISTANCE
                    && offset.x <= map_object::TURRENT_MAX_DISTANCE
            );
            assert!(
                offset.y > -map_object::TURRENT_MAX_DISTANCE
                    && offset.y <= map_object::TURRENT_MAX_DISTANCE
            );
            assert!((3.0..=3.99).contains(&map_object::turrent_delay(&mut rng)));
            assert!((0.5..=1.49).contains(&map_object::turrent_rise(&mut rng)));
        }

        assert_eq!(map_object::turrent_arc_size(1.0, 3.0, 0.0), 0.0);
        assert!(map_object::turrent_arc_size(1.0, 3.0, 1.0) > 0.0);
    }

    #[test]
    fn building_death_profiles_match_original_do_death_effects() {
        assert_eq!(
            buildings::death_profile(
                ObjectKind::Building(BuildingType::Radar),
                PlanetType::Desert
            )
            .unwrap(),
            BuildingDeathProfile {
                effect_box: BuildingEffectBox {
                    x: 1.0,
                    y: 6.0,
                    width: 44,
                    height: 30
                },
                width_pix: 64.0,
                height_pix: 48.0,
                max_effects_base: 6,
                max_effects_random: 3,
                fireball_base: 4,
                fireball_random: 3,
                piece_base: 3,
                piece_random: 3,
                piece_variants: 2,
                piece_flight_base: 1.5
            }
        );
        assert_eq!(
            buildings::death_profile(
                ObjectKind::Building(BuildingType::Repair),
                PlanetType::Desert
            )
            .unwrap(),
            BuildingDeathProfile {
                effect_box: BuildingEffectBox {
                    x: 8.0,
                    y: 8.0,
                    width: 56,
                    height: 40
                },
                width_pix: 80.0,
                height_pix: 64.0,
                max_effects_base: 6,
                max_effects_random: 4,
                fireball_base: 6,
                fireball_random: 3,
                piece_base: 4,
                piece_random: 3,
                piece_variants: 2,
                piece_flight_base: 1.5
            }
        );
        assert_eq!(
            buildings::death_profile(
                ObjectKind::Building(BuildingType::RobotFactory),
                PlanetType::Desert
            )
            .unwrap(),
            BuildingDeathProfile {
                effect_box: BuildingEffectBox {
                    x: 8.0,
                    y: 8.0,
                    width: 40,
                    height: 56
                },
                width_pix: 64.0,
                height_pix: 80.0,
                max_effects_base: 8,
                max_effects_random: 4,
                fireball_base: 8,
                fireball_random: 3,
                piece_base: 6,
                piece_random: 3,
                piece_variants: 2,
                piece_flight_base: 1.5
            }
        );
        assert_eq!(
            buildings::death_profile(
                ObjectKind::Building(BuildingType::VehicleFactory),
                PlanetType::Desert
            )
            .unwrap(),
            buildings::death_profile(
                ObjectKind::Building(BuildingType::RobotFactory),
                PlanetType::Desert
            )
            .unwrap()
        );

        let fort_front = buildings::death_profile(
            ObjectKind::Building(BuildingType::FortFront),
            PlanetType::Desert,
        )
        .unwrap();
        assert_eq!(
            fort_front,
            BuildingDeathProfile {
                effect_box: BuildingEffectBox {
                    x: 18.0,
                    y: 18.0,
                    width: 136,
                    height: 118
                },
                width_pix: 160.0,
                height_pix: 192.0,
                max_effects_base: 20,
                max_effects_random: 8,
                fireball_base: 12,
                fireball_random: 6,
                piece_base: 16,
                piece_random: 6,
                piece_variants: 5,
                piece_flight_base: 3.0
            }
        );
        assert_eq!(
            buildings::death_profile(
                ObjectKind::Building(BuildingType::FortBack),
                PlanetType::Desert
            )
            .unwrap()
            .height_pix,
            176.0
        );
        assert_eq!(
            buildings::death_profile(
                ObjectKind::Building(BuildingType::FortFront),
                PlanetType::Jungle
            )
            .unwrap()
            .height_pix,
            176.0
        );
    }

    #[test]
    fn building_piece_assets_and_timing_match_original_turrent_missiles() {
        assert_eq!(
            buildings::building_piece_frame_paths_for_variants(0, 2).len(),
            12
        );
        assert_eq!(
            buildings::building_piece_frame_paths_for_variants(0, 2)
                .first()
                .unwrap(),
            "buildings/death_effects/piece0_n00.png"
        );
        assert_eq!(
            buildings::building_piece_frame_paths_for_variants(1, 2)
                .last()
                .unwrap(),
            "buildings/death_effects/piece1_n11.png"
        );
        assert_eq!(
            buildings::building_piece_frame_paths_for_variants(4, 5)
                .last()
                .unwrap(),
            "buildings/death_effects/fort_piece4_n11.png"
        );
        assert_eq!(
            buildings::building_top_left_from_grid(MapGridPosition { x: 3, y: 4 }),
            Vec2::new(48.0, 64.0)
        );
    }

    #[test]
    fn vehicle_death_wreck_assets_match_original_edeath_inputs() {
        assert_eq!(
            vehicles::death_wreck_asset_path(VehicleType::Jeep, TeamType::Red, 180, 0).unwrap(),
            "units/vehicles/jeep/wasted.png"
        );
        assert_eq!(
            vehicles::death_wreck_asset_path(VehicleType::Apc, TeamType::Blue, 180, 0).unwrap(),
            "units/vehicles/apc/wasted.png"
        );
        assert_eq!(
            vehicles::death_wreck_asset_path(VehicleType::MissileLauncher, TeamType::Green, 180, 0)
                .unwrap(),
            "units/vehicles/missile_launcher/wasted.png"
        );
        assert_eq!(
            vehicles::death_wreck_asset_path(VehicleType::Crane, TeamType::Yellow, 180, 0).unwrap(),
            "units/vehicles/crane/wasted_null.png"
        );
        assert_eq!(
            vehicles::death_wreck_asset_path(VehicleType::Light, TeamType::Red, 225, 1).unwrap(),
            "units/vehicles/light/base_damaged_red_r045_n01.png"
        );
        assert!(
            vehicles::death_wreck_asset_path(VehicleType::Heavy, TeamType::Null, 0, 0).is_none()
        );
    }

    #[test]
    fn vehicle_death_effect_bounds_match_original_edeath_rects() {
        assert_eq!(
            vehicles::death_effect_bounds(VehicleType::Jeep).unwrap(),
            vehicles::VehicleDeathEffectBounds {
                x: 5,
                y: 14,
                width: 22,
                height: 10
            }
        );
        assert_eq!(
            vehicles::death_effect_bounds(VehicleType::MissileLauncher).unwrap(),
            vehicles::VehicleDeathEffectBounds {
                x: 5,
                y: 10,
                width: 21,
                height: 19
            }
        );
        assert_eq!(
            vehicles::death_effect_bounds(VehicleType::Apc).unwrap(),
            vehicles::VehicleDeathEffectBounds {
                x: 5,
                y: 8,
                width: 18,
                height: 20
            }
        );
        assert_eq!(
            vehicles::death_effect_bounds(VehicleType::Crane).unwrap(),
            vehicles::VehicleDeathEffectBounds {
                x: 4,
                y: 9,
                width: 23,
                height: 19
            }
        );
        assert_eq!(
            vehicles::death_effect_bounds(VehicleType::Heavy).unwrap(),
            vehicles::VehicleDeathEffectBounds {
                x: 8,
                y: 8,
                width: 16,
                height: 16
            }
        );
    }

    #[test]
    fn vehicle_death_standard_effect_assets_and_offsets_match_original() {
        let anchor = Vec2::new(100.0, 50.0);
        let (big_pos, big_frames) = vehicles::death_standard_effect_shape(
            vehicles::VehicleDeathStandardKind::BigSmoke,
            anchor,
        )
        .unwrap();
        let (fire_pos, fire_frames) = vehicles::death_standard_effect_shape(
            vehicles::VehicleDeathStandardKind::LittleFire,
            anchor,
        )
        .unwrap();
        let (small_pos, small_frames) = vehicles::death_standard_effect_shape(
            vehicles::VehicleDeathStandardKind::SmallFireSmoke,
            anchor,
        )
        .unwrap();
        let (full_fire_pos, full_fire_frames) =
            vehicles::death_standard_effect_shape(vehicles::VehicleDeathStandardKind::Fire, anchor)
                .unwrap();

        assert_eq!(big_pos, Vec2::new(84.0, 18.0));
        assert_eq!(fire_pos, Vec2::new(96.0, 42.0));
        assert_eq!(small_pos, Vec2::new(92.0, 34.0));
        assert_eq!(full_fire_pos, Vec2::new(96.0, 42.0));
        assert_eq!(estandard_initial_elapsed(), 0.05);
        assert_eq!(
            big_frames.last().unwrap(),
            "units/vehicles/death_effects/big_smoke_n03.png"
        );
        assert_eq!(
            fire_frames.last().unwrap(),
            "units/vehicles/death_effects/little_fire_n03.png"
        );
        assert_eq!(
            small_frames.last().unwrap(),
            "units/vehicles/death_effects/small_fire_smoke_n03.png"
        );
        assert_eq!(
            full_fire_frames.last().unwrap(),
            "units/vehicles/death_effects/fire_n03.png"
        );
    }

    #[test]
    fn vehicle_death_timing_and_spark_ranges_match_original() {
        let mut rng = CombatRng::default();
        for _ in 0..32 {
            let lifetime = vehicles::death_lifetime(&mut rng);
            assert!((5.0..=7.0).contains(&lifetime));
            let sparks = vehicles::death_spark_count(&mut rng);
            assert!((40..=69).contains(&sparks));

            let flight = vehicles::turrent_flight_time(&mut rng);
            assert!((3.0..=3.99).contains(&flight));

            let target = vehicles::turrent_target(Vec2::new(16.0, 16.0), &mut rng);
            assert!(target.x > -284.0 && target.x <= 316.0);
            assert!(target.y > -284.0 && target.y <= 316.0);
        }

        assert_eq!(vehicles::death_damaged_frame(180, 0), (0, 2));
        assert_eq!(vehicles::death_damaged_frame(270, 2), (90, 0));
        assert_eq!(
            vehicles::death_top_left_world(Vec2::new(100.0, -50.0)),
            Vec2::new(84.0, -34.0)
        );
        assert_eq!(
            vehicles::turrent_frame_paths(VehicleType::Light, TeamType::Red)
                .unwrap()
                .last()
                .unwrap(),
            "units/vehicles/light/top_pop_n07.png"
        );
        assert_eq!(
            vehicles::turrent_frame_paths(VehicleType::Medium, TeamType::Red)
                .unwrap()
                .first()
                .unwrap(),
            "units/vehicles/medium/top_pop_n00.png"
        );
        assert_eq!(
            vehicles::turrent_frame_paths(VehicleType::Heavy, TeamType::Blue)
                .unwrap()
                .last()
                .unwrap(),
            "units/vehicles/heavy/top_pop_blue_n07.png"
        );
        assert!(vehicles::turrent_frame_paths(VehicleType::Heavy, TeamType::Null).is_none());
        assert_eq!(turrent_crater().chance, 0.35);
    }

    #[test]
    fn cannon_turrent_and_spark_assets_match_original_frames() {
        assert_eq!(vehicles::death_spark_frame_paths().len(), 6);
        assert_eq!(
            vehicles::death_spark_frame_paths().last().unwrap(),
            "units/vehicles/death_effects/spark_n05.png"
        );
        assert_eq!(cannons::turrent_arc_size(2.0, 4.0, 0.0), 1.0);
        assert!(cannons::turrent_arc_size(2.0, 4.0, 1.0) > 1.0);
        assert_eq!(vehicles::death_spark_arc_size(3.0, 1.5, 0.0), 0.0);
        assert!(vehicles::death_spark_arc_size(3.0, 1.5, 0.5) > 0.0);
    }

    #[test]
    fn grenade_attack_amount_only_counts_original_grenade_robots() {
        assert_eq!(
            grenade_attack_amount(ObjectKind::Robot(RobotType::Grunt), Some(4)),
            4
        );
        assert_eq!(
            grenade_attack_amount(ObjectKind::Robot(RobotType::Tough), Some(4)),
            0
        );
        assert_eq!(
            grenade_attack_amount(ObjectKind::Vehicle(VehicleType::Jeep), Some(4)),
            0
        );
    }

    #[test]
    fn grenade_box_destroy_spawns_one_delayed_missile_per_grenade() {
        let mut rng = CombatRng::default();
        let missiles = item_grenades::destroy_missile_rules(Vec2::new(10.0, 20.0), 20, &mut rng);

        assert_eq!(missiles.len(), 20);
        assert!(missiles.iter().all(|missile| {
            missile.target.x >= 10.0 - GRENADE_BOX_SCATTER_HALF_EXTENT
                && missile.target.x <= 10.0 + GRENADE_BOX_SCATTER_HALF_EXTENT
                && missile.target.y >= 20.0 - GRENADE_BOX_SCATTER_HALF_EXTENT
                && missile.target.y <= 20.0 + GRENADE_BOX_SCATTER_HALF_EXTENT
                && missile.delay >= GRENADE_BOX_EXPLOSION_DELAY
                && missile.delay < GRENADE_BOX_EXPLOSION_DELAY + 1.0
        }));
    }

    #[test]
    fn null_team_explosion_damages_neutral_objects_like_original() {
        let neutral_rock = ObjectStats::from_kind(ObjectKind::MapItem(ItemType::Rock as u8), 100);
        let snapshots = vec![CombatObjectSnapshot {
            ref_id: 42,
            kind: ObjectKind::MapItem(ItemType::Rock as u8),
            position: Vec2::ZERO,
            size: Vec2::splat(16.0),
            team: TeamType::Null,
            stats: neutral_rock,
            attack_target_ref: None,
        }];

        assert_eq!(
            explosion_damage_targets(
                &snapshots,
                PendingExplosion {
                    position: Vec2::ZERO,
                    damage: GRENADE_DAMAGE,
                    radius: item_grenades::destroy_missile_radius(),
                    team: TeamType::Null,
                    visual: DamageMissileVisual::Grenade,
                    crater: None,
                },
            ),
            vec![(42, GRENADE_DAMAGE)]
        );
    }

    #[test]
    fn passive_engage_targets_only_units_and_cannons() {
        assert!(passive_engage_target_kind(ObjectKind::Robot(
            RobotType::Grunt
        )));
        assert!(passive_engage_target_kind(ObjectKind::Vehicle(
            VehicleType::Jeep
        )));
        assert!(passive_engage_target_kind(ObjectKind::Cannon(
            CannonType::Gun
        )));
        assert!(!passive_engage_target_kind(ObjectKind::Building(
            BuildingType::Radar
        )));
        assert!(!passive_engage_target_kind(ObjectKind::MapItem(
            ItemType::Grenades as u8
        )));
    }

    #[test]
    fn passive_engage_skips_moving_robots() {
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let jeep = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let movement = MovementPath::new(vec![Vec2::new(16.0, 0.0)], grunt.move_speed);

        assert!(!can_passively_engage(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            grunt,
            true,
            Some(&movement)
        ));
        assert!(can_passively_engage(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Red,
            jeep,
            true,
            Some(&movement)
        ));
    }

    #[test]
    fn passive_attack_uses_first_in_range_but_agro_uses_closest() {
        let attacker = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let target_stats = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let snapshots = vec![
            CombatObjectSnapshot {
                ref_id: 2,
                kind: ObjectKind::Vehicle(VehicleType::Heavy),
                position: Vec2::new(100.0, 0.0),
                size: Vec2::splat(32.0),
                team: TeamType::Blue,
                stats: target_stats,
                attack_target_ref: None,
            },
            CombatObjectSnapshot {
                ref_id: 3,
                kind: ObjectKind::Vehicle(VehicleType::Jeep),
                position: Vec2::new(50.0, 0.0),
                size: Vec2::splat(32.0),
                team: TeamType::Blue,
                stats: target_stats,
                attack_target_ref: None,
            },
        ];

        assert_eq!(
            passive_attack_target_choice(1, Vec2::ZERO, TeamType::Red, attacker, &snapshots)
                .map(|target| target.ref_id),
            Some(2)
        );

        let far_snapshots = vec![
            CombatObjectSnapshot {
                ref_id: 2,
                kind: ObjectKind::Vehicle(VehicleType::Heavy),
                position: Vec2::new(150.0, 0.0),
                size: Vec2::splat(32.0),
                team: TeamType::Blue,
                stats: target_stats,
                attack_target_ref: None,
            },
            CombatObjectSnapshot {
                ref_id: 3,
                kind: ObjectKind::Vehicle(VehicleType::Jeep),
                position: Vec2::new(130.0, 0.0),
                size: Vec2::splat(32.0),
                team: TeamType::Blue,
                stats: target_stats,
                attack_target_ref: None,
            },
        ];
        assert_eq!(
            passive_agro_target_choice(1, Vec2::ZERO, TeamType::Red, attacker, &far_snapshots)
                .map(|target| target.ref_id),
            Some(3)
        );
    }

    #[test]
    fn passive_grenade_pickup_chooses_nearest_box_inside_auto_radius() {
        let robot = PassiveAutoEnterRobotSnapshot {
            ref_id: 1,
            position: Vec2::ZERO,
            has_waypoint: false,
            has_attack_target: false,
            has_task_target: false,
            is_minion: false,
            just_left_cannon: false,
        };
        let boxes = vec![
            PassiveGrenadeBoxSnapshot {
                ref_id: 10,
                position: Vec2::new(180.0, 0.0),
            },
            PassiveGrenadeBoxSnapshot {
                ref_id: 11,
                position: Vec2::new(80.0, 0.0),
            },
        ];

        assert_eq!(
            passive_grenade_pickup_target_choice(
                ObjectKind::Robot(RobotType::Grunt),
                true,
                robot,
                Some(0),
                &boxes,
            )
            .map(|target| target.ref_id),
            Some(11)
        );
    }

    #[test]
    fn passive_grenade_pickup_requires_available_robot_and_auto_radius() {
        let robot = PassiveAutoEnterRobotSnapshot {
            ref_id: 1,
            position: Vec2::ZERO,
            has_waypoint: false,
            has_attack_target: false,
            has_task_target: false,
            is_minion: false,
            just_left_cannon: false,
        };
        let far_box = [PassiveGrenadeBoxSnapshot {
            ref_id: 10,
            position: Vec2::new(AUTO_GRAB_VEHICLE_DISTANCE + 1.0, 0.0),
        }];
        let near_box = [PassiveGrenadeBoxSnapshot {
            ref_id: 11,
            position: Vec2::new(AUTO_GRAB_VEHICLE_DISTANCE, 0.0),
        }];

        assert!(
            passive_grenade_pickup_target_choice(
                ObjectKind::Robot(RobotType::Grunt),
                true,
                robot,
                Some(0),
                &far_box,
            )
            .is_none()
        );
        assert!(
            passive_grenade_pickup_target_choice(
                ObjectKind::Robot(RobotType::Grunt),
                true,
                robot,
                Some(1),
                &near_box,
            )
            .is_none()
        );
        assert!(
            passive_grenade_pickup_target_choice(
                ObjectKind::Robot(RobotType::Tough),
                true,
                robot,
                Some(0),
                &near_box,
            )
            .is_none()
        );
        assert!(
            passive_grenade_pickup_target_choice(
                ObjectKind::Robot(RobotType::Grunt),
                true,
                PassiveAutoEnterRobotSnapshot {
                    is_minion: true,
                    ..robot
                },
                Some(0),
                &near_box,
            )
            .is_none()
        );
        assert_eq!(
            passive_grenade_pickup_target_choice(
                ObjectKind::Robot(RobotType::Grunt),
                true,
                robot,
                Some(0),
                &near_box,
            )
            .map(|target| target.ref_id),
            Some(11)
        );
    }

    #[test]
    fn passive_auto_enter_skips_cannon_after_just_left_cannon() {
        let robot = PassiveAutoEnterRobotSnapshot {
            ref_id: 1,
            position: Vec2::ZERO,
            has_waypoint: false,
            has_attack_target: false,
            has_task_target: false,
            is_minion: false,
            just_left_cannon: true,
        };
        let targets = vec![
            PassiveAutoEnterTargetSnapshot {
                ref_id: 2,
                kind: ObjectKind::Cannon(CannonType::Gun),
                position: Vec2::new(10.0, 0.0),
            },
            PassiveAutoEnterTargetSnapshot {
                ref_id: 3,
                kind: ObjectKind::Vehicle(VehicleType::Jeep),
                position: Vec2::new(20.0, 0.0),
            },
        ];

        assert_eq!(
            passive_auto_enter_target_choice(
                ObjectKind::Robot(RobotType::Grunt),
                true,
                robot,
                &targets
            )
            .map(|target| target.ref_id),
            Some(3)
        );
    }

    #[test]
    fn passive_auto_enter_requires_idle_leader_robot() {
        let target = PassiveAutoEnterTargetSnapshot {
            ref_id: 2,
            kind: ObjectKind::Vehicle(VehicleType::Jeep),
            position: Vec2::new(10.0, 0.0),
        };
        let idle_robot = PassiveAutoEnterRobotSnapshot {
            ref_id: 1,
            position: Vec2::ZERO,
            has_waypoint: false,
            has_attack_target: false,
            has_task_target: false,
            is_minion: false,
            just_left_cannon: false,
        };

        assert_eq!(
            passive_auto_enter_target_choice(
                ObjectKind::Robot(RobotType::Grunt),
                true,
                idle_robot,
                &[target]
            )
            .map(|target| target.ref_id),
            Some(2)
        );

        assert!(
            passive_auto_enter_target_choice(
                ObjectKind::Vehicle(VehicleType::Jeep),
                true,
                idle_robot,
                &[target]
            )
            .is_none()
        );

        assert!(
            passive_auto_enter_target_choice(
                ObjectKind::Robot(RobotType::Grunt),
                true,
                PassiveAutoEnterRobotSnapshot {
                    has_waypoint: true,
                    ..idle_robot
                },
                &[target]
            )
            .is_none()
        );

        assert!(
            passive_auto_enter_target_choice(
                ObjectKind::Robot(RobotType::Grunt),
                true,
                PassiveAutoEnterRobotSnapshot {
                    is_minion: true,
                    ..idle_robot
                },
                &[target]
            )
            .is_none()
        );
    }

    #[test]
    fn missile_aoe_damage_uses_linear_falloff() {
        assert_eq!(aoe_damage_at_distance(100.0, 40.0, 0.0), 100.0);
        assert_eq!(aoe_damage_at_distance(100.0, 40.0, 20.0), 50.0);
        assert_eq!(aoe_damage_at_distance(100.0, 40.0, 40.0), 0.0);
        assert_eq!(aoe_damage_at_distance(100.0, 40.0, 41.0), 0.0);
    }

    #[test]
    fn crater_asset_counts_and_paths_match_original_sequences() {
        assert_eq!(crater_variant_count(PlanetType::Desert, 0, true), 3);
        assert_eq!(crater_variant_count(PlanetType::Desert, 1, true), 7);
        assert_eq!(crater_variant_count(PlanetType::Desert, 1, false), 3);
        assert_eq!(crater_variant_count(PlanetType::Volcanic, 0, false), 2);
        assert_eq!(crater_variant_count(PlanetType::Arctic, 0, false), 0);
        assert_eq!(crater_variant_count(PlanetType::City, 2, true), 4);
        assert_eq!(crater_variant_count(PlanetType::Desert, 6, true), 0);

        assert_eq!(
            crater_asset_path(PlanetType::Jungle, 2, false, 1),
            "planets/craters/crater_large_jungle_t02_n01.png"
        );
        assert_eq!(
            crater_asset_path(PlanetType::City, 0, true, 1),
            "planets/craters/crater_small_city_t00_n01.png"
        );
    }

    #[test]
    fn crater_creation_downgrades_big_at_map_edge_like_original() {
        let map = tiny_tile_map(vec![1, 1, 1, 1], PlanetType::Desert);
        let tile_info = vec![
            test_tile_info(false),
            original::tileinfo::PaletteTileInfo {
                crater_type: 1,
                ..test_tile_info(false)
            },
        ];
        let registry = CraterStampRegistry::default();
        let mut rng = CombatRng::default();

        let spec = create_crater_stamp_spec(
            &map,
            &tile_info,
            &registry,
            &mut rng,
            Vec2::new(24.0, 24.0),
            DamageCrater {
                is_big: true,
                chance: 1.0,
                big_chance: None,
            },
        )
        .unwrap();

        assert!(!spec.is_big);
        assert_eq!(spec.tile, IVec2::new(1, 1));
        assert_eq!(spec.crater_type, 1);
        assert!(
            spec.asset_path
                .starts_with("planets/craters/crater_small_desert_t01_n")
        );
    }

    #[test]
    fn crater_creation_keeps_original_type_when_nonuniform_big_downgrades() {
        let map = tiny_tile_map(vec![1, 0, 0, 0], PlanetType::Desert);
        let tile_info = vec![
            original::tileinfo::PaletteTileInfo {
                crater_type: 0,
                ..test_tile_info(false)
            },
            original::tileinfo::PaletteTileInfo {
                crater_type: 1,
                ..test_tile_info(false)
            },
        ];
        let registry = CraterStampRegistry::default();
        let mut rng = CombatRng::default();

        let spec = create_crater_stamp_spec(
            &map,
            &tile_info,
            &registry,
            &mut rng,
            Vec2::new(8.0, 8.0),
            DamageCrater {
                is_big: true,
                chance: 1.0,
                big_chance: None,
            },
        )
        .unwrap();

        assert!(!spec.is_big);
        assert_eq!(spec.crater_type, 1);
        assert!(
            spec.asset_path
                .starts_with("planets/craters/crater_small_desert_t01_n")
        );
    }

    #[test]
    fn crater_registry_blocks_small_on_big_stamped_tile() {
        let map = tiny_tile_map(vec![1, 1, 1, 1], PlanetType::Desert);
        let tile_info = vec![
            test_tile_info(false),
            original::tileinfo::PaletteTileInfo {
                crater_type: 1,
                ..test_tile_info(false)
            },
        ];
        let mut registry = CraterStampRegistry::default();
        registry.stamped_tiles.insert((0, 0));
        let mut rng = CombatRng::default();

        assert!(
            create_crater_stamp_spec(
                &map,
                &tile_info,
                &registry,
                &mut rng,
                Vec2::new(8.0, 8.0),
                DamageCrater {
                    is_big: false,
                    chance: 1.0,
                    big_chance: None,
                },
            )
            .is_none()
        );
    }

    #[test]
    fn run_stamina_drains_recharges_and_clamps_like_original() {
        let mut stamina = MovementStamina {
            max: 2.0,
            current: 1.0,
            running: true,
        };

        process_run_stamina(&mut stamina, 1.5);
        assert_eq!(stamina.current, 0.0);
        assert!(!stamina.running);

        process_run_stamina(&mut stamina, 10.0);
        assert_eq!(stamina.current, 2.0);
        assert!(!stamina.running);
    }

    #[test]
    fn attempt_start_run_matches_reach_stamina_and_random_gate() {
        let mut stamina = MovementStamina {
            max: 2.0,
            current: 1.0,
            running: false,
        };

        assert!(can_reach_target_running(
            10.0,
            stamina.current,
            Vec2::ZERO,
            Vec2::new(9.0, 0.0)
        ));
        attempt_start_run(&mut stamina, true, 0.19);
        assert!(!stamina.running);

        attempt_start_run(&mut stamina, true, 0.2);
        assert!(stamina.running);

        stamina.running = false;
        stamina.current = 0.29;
        attempt_start_run(&mut stamina, true, 0.9);
        assert!(!stamina.running);
    }

    #[test]
    fn vehicle_damage_speed_thresholds_are_strict_like_original() {
        let mut jeep = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);

        jeep.health = jeep.max_health * 0.7;
        assert_eq!(
            damaged_speed_multiplier(ObjectKind::Vehicle(VehicleType::Jeep), jeep),
            1.0
        );
        jeep.health = jeep.max_health * 0.69;
        assert_eq!(
            damaged_speed_multiplier(ObjectKind::Vehicle(VehicleType::Jeep), jeep),
            vehicles::damaged_speed_multiplier_for_ratio(0.69)
        );
        jeep.health = jeep.max_health * 0.4;
        assert_eq!(
            damaged_speed_multiplier(ObjectKind::Vehicle(VehicleType::Jeep), jeep),
            1.0
        );
        jeep.health = jeep.max_health * 0.39;
        assert_eq!(
            damaged_speed_multiplier(ObjectKind::Vehicle(VehicleType::Jeep), jeep),
            vehicles::damaged_speed_multiplier_for_ratio(0.39)
        );
    }

    #[test]
    fn run_multiplier_is_suppressed_only_for_damaged_vehicles() {
        let robot = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let mut jeep = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);

        assert_eq!(
            movement_speed_multiplier(ObjectKind::Robot(RobotType::Grunt), robot, true),
            RUN_UNIT_SPEED
        );
        jeep.health = jeep.max_health * 0.69;
        assert_eq!(
            movement_speed_multiplier(ObjectKind::Vehicle(VehicleType::Jeep), jeep, true),
            vehicles::movement_speed_multiplier_for_ratio(0.69, true)
        );
        jeep.health = jeep.max_health * 0.39;
        assert_eq!(
            movement_speed_multiplier(ObjectKind::Vehicle(VehicleType::Jeep), jeep, true),
            vehicles::movement_speed_multiplier_for_ratio(0.39, true)
        );
    }

    #[test]
    fn vehicle_track_offsets_match_original_set_track_coords() {
        assert_eq!(
            vehicles::track_offsets(0),
            [Vec2::new(-15.0, -2.0), Vec2::new(-15.0, 10.0)]
        );
        assert_eq!(
            vehicles::track_offsets(3),
            [Vec2::new(15.0, 8.0), Vec2::new(5.0, 18.0)]
        );
        assert_eq!(
            vehicles::track_offsets(6),
            [Vec2::new(-8.0, -11.0), Vec2::new(8.0, -11.0)]
        );
        assert_eq!(vehicles::track_frame_for_delta(-0.1), Some(0));
        assert_eq!(vehicles::track_frame_for_delta(3.29), Some(0));
        assert_eq!(vehicles::track_frame_for_delta(3.3), Some(1));
        assert_eq!(vehicles::track_frame_for_delta(3.6), Some(2));
        assert_eq!(vehicles::track_frame_for_delta(3.9), None);

        let mut rng = CombatRng::default();
        for _ in 0..16 {
            let delay = vehicles::track_start_delay(&mut rng);
            assert!((0.0..=0.900_001).contains(&delay));
        }
    }

    #[test]
    fn vehicle_track_road_gate_checks_point_and_cardinal_neighbors() {
        let map = tiny_tile_map(vec![0, 1, 0, 0], PlanetType::Desert);
        let tile_info = vec![test_tile_info(false), test_tile_info(true)];

        assert!(coord_is_road(&map, &tile_info, Vec2::new(17.0, 1.0)));
        assert!(track_point_is_near_road(
            &map,
            &tile_info,
            Vec2::new(17.0, 1.0)
        ));
        assert!(!track_point_is_near_road(
            &map,
            &tile_info,
            Vec2::new(1.0, 33.0)
        ));
        assert!(!coord_is_road(&map, &tile_info, Vec2::new(-1.0, 1.0)));
    }

    #[test]
    fn vehicle_damage_effect_gates_match_original_strict_thresholds() {
        assert!(!vehicles::show_partially_damaged_effects(0.7));
        assert!(vehicles::show_partially_damaged_effects(0.69));
        assert!(vehicles::show_partially_damaged_effects(0.41));
        assert!(!vehicles::show_partially_damaged_effects(0.4));

        assert!(!vehicles::show_heavily_damaged_effects(0.4));
        assert!(vehicles::show_heavily_damaged_effects(0.39));
    }

    #[test]
    fn vehicle_effect_timer_starts_ready_like_original_zero_next_drop_time() {
        let mut timer = VehicleEffectDropTimer {
            elapsed: vehicles::EFFECT_DROP_INTERVAL,
        };
        assert!(tick_vehicle_effect_drop_timer(&mut timer, 0.0));
        assert!(!tick_vehicle_effect_drop_timer(&mut timer, 0.19));
        assert!(tick_vehicle_effect_drop_timer(&mut timer, 0.01));
    }

    #[test]
    fn vehicle_effect_asset_rules_match_original_track_loading() {
        assert_eq!(vehicles::tank_dirt_shape(PlanetType::Desert), Some((2, 5)));
        assert_eq!(vehicles::tank_dirt_shape(PlanetType::Jungle), Some((1, 6)));
        assert_eq!(vehicles::tank_dirt_shape(PlanetType::City), None);
        assert_eq!(
            vehicles::track_effect_frame_paths(VehicleType::Jeep, PlanetType::Desert, 5)
                .unwrap()
                .last()
                .unwrap(),
            "units/vehicles/track_effects/jeep_track_desert_r045_n02.png"
        );
        assert!(
            vehicles::track_effect_frame_paths(VehicleType::Jeep, PlanetType::Jungle, 0).is_none()
        );
        assert_eq!(
            vehicles::tank_smoke_frame_paths(6, true).first().unwrap(),
            "units/vehicles/track_spark_r270_n00.png"
        );
        let mut oil_rng = CombatRng::default();
        assert_eq!(
            vehicles::tank_oil_frame_paths(&mut oil_rng),
            vec![
                "units/vehicles/tank_oil_2_n00.png".to_string(),
                "units/vehicles/tank_oil_2_n01.png".to_string(),
                "units/vehicles/tank_oil_2_n02.png".to_string(),
            ]
        );
        assert_eq!(
            vehicles::tank_spark_frame_paths().last().unwrap(),
            "units/vehicles/ground_spark_n05.png"
        );
        assert_eq!(rotation_for_direction(5 % 4), 45);
    }

    #[test]
    fn destroyed_assets_use_original_wasted_paths() {
        assert_eq!(
            destroyed_asset_name(
                ObjectKind::Vehicle(VehicleType::Jeep),
                TeamType::Red,
                PlanetType::Desert
            ),
            Some("units/vehicles/jeep/wasted.png".to_string())
        );
        assert_eq!(
            destroyed_asset_name(
                ObjectKind::Vehicle(VehicleType::MissileLauncher),
                TeamType::Blue,
                PlanetType::Desert
            ),
            Some("units/vehicles/missile_launcher/wasted_blue.png".to_string())
        );
        assert_eq!(
            destroyed_asset_name(
                ObjectKind::Cannon(CannonType::MissileCannon),
                TeamType::Green,
                PlanetType::Desert
            ),
            Some("units/cannons/missile_cannon/wasted_green.png".to_string())
        );
        assert_eq!(
            destroyed_asset_name(
                ObjectKind::Building(BuildingType::Radar),
                TeamType::Red,
                PlanetType::Arctic
            ),
            Some("buildings/radar/base_destroyed_arctic.png".to_string())
        );
        assert_eq!(
            destroyed_asset_name(
                ObjectKind::Robot(RobotType::Grunt),
                TeamType::Red,
                PlanetType::Desert
            ),
            None
        );
    }

    #[test]
    fn destroyable_policy_matches_original_lifecycle() {
        assert!(can_be_destroyed(ObjectKind::Vehicle(VehicleType::Jeep)));
        assert!(can_be_destroyed(ObjectKind::Cannon(CannonType::Gun)));
        assert!(can_be_destroyed(ObjectKind::Robot(RobotType::Grunt)));
        assert!(can_be_destroyed(ObjectKind::MapItem(
            ItemType::Grenades as u8
        )));
        assert!(!can_be_destroyed(ObjectKind::Building(BuildingType::Radar)));
        assert!(!can_be_destroyed(ObjectKind::Bridge(
            BuildingType::BridgeHorz
        )));
        assert!(!can_be_destroyed(ObjectKind::MapItem(ItemType::Flag as u8)));
        assert!(map_item_blocks_tile(ObjectKind::MapItem(
            ItemType::Hut as u8
        )));
        assert!(map_item_blocks_tile(ObjectKind::MapItem(
            ItemType::MapObjectStart as u8
        )));
        assert!(!map_item_blocks_tile(ObjectKind::MapItem(
            ItemType::Grenades as u8
        )));
    }

    #[test]
    fn auto_repair_policy_matches_original_non_fort_buildings() {
        assert!(buildings::auto_repairable_after_destroy(
            ObjectKind::Building(BuildingType::Radar)
        ));
        assert!(buildings::auto_repairable_after_destroy(
            ObjectKind::Building(BuildingType::Repair)
        ));
        assert!(buildings::auto_repairable_after_destroy(
            ObjectKind::Bridge(BuildingType::BridgeVert)
        ));
        assert!(!buildings::auto_repairable_after_destroy(
            ObjectKind::Building(BuildingType::FortFront)
        ));
        assert!(!buildings::auto_repairable_after_destroy(
            ObjectKind::Cannon(CannonType::Gun)
        ));
    }

    #[test]
    fn auto_repair_delay_matches_original_default_range() {
        let mut rng = CombatRng::default();
        for _ in 0..32 {
            let delay = buildings::auto_repair_delay(&mut rng);
            assert!((600.0..=660.0).contains(&delay));
        }
    }

    #[test]
    fn auto_repair_zone_blocker_matches_original_fort_front_check() {
        assert!(buildings::auto_repair_blocking_fort(ObjectKind::Building(
            BuildingType::FortFront
        )));
        assert!(!buildings::auto_repair_blocking_fort(ObjectKind::Building(
            BuildingType::FortBack
        )));
        assert!(!buildings::auto_repair_blocking_fort(ObjectKind::Building(
            BuildingType::Radar
        )));
    }

    #[test]
    fn bridge_world_rect_uses_original_tile_footprint() {
        let bridge = BridgeFootprint {
            x: 2,
            y: 3,
            building: BuildingType::BridgeHorz,
            extra_links: 2,
        };

        let rect = buildings::bridge_world_rect(bridge).expect("horizontal bridge footprint");

        assert_eq!(rect.min_x, 2.0 * TILE_SIZE);
        assert_eq!(rect.max_x, 9.0 * TILE_SIZE);
        assert_eq!(rect.min_y, -7.0 * TILE_SIZE);
        assert_eq!(rect.max_y, -3.0 * TILE_SIZE);
    }

    #[test]
    fn bridge_effect_assets_match_original_paths() {
        let bridge_frames = buildings::bridge_turrent_frame_paths(PlanetType::Desert);
        assert_eq!(bridge_frames.len(), 12);
        assert_eq!(
            bridge_frames.first().unwrap(),
            "planets/bridge_effects/debri_large_desert_n00.png"
        );
        assert_eq!(
            bridge_frames.last().unwrap(),
            "planets/bridge_effects/debri_large_desert_n11.png"
        );

        let rock_frames = buildings::bridge_rock_particle_frame_paths(PlanetType::Arctic);
        assert_eq!(rock_frames.len(), 16);
        assert_eq!(
            rock_frames.last().unwrap(),
            "planets/rock_effects/debri_small_arctic_n15.png"
        );
    }

    #[test]
    fn map_object_and_rock_death_assets_match_original_paths() {
        assert_eq!(
            item_rock::destroyed_rubble_indices(),
            [33, 34, 35, 17, 23, 29]
        );

        let unit_frames = map_object::unit_particle_frame_paths();
        assert_eq!(unit_frames.len(), 20);
        assert_eq!(unit_frames[0], "other/particles/unit_particle_n00.png");
        assert_eq!(unit_frames[19], "other/particles/unit_particle_n19.png");

        let mut rng = CombatRng::default();
        let small = item_rock::particle_frame_paths(
            PlanetType::Desert,
            item_rock::RockParticleKind::Small,
            &mut rng,
        );
        assert_eq!(small.len(), 16);
        assert_eq!(
            small.last().unwrap(),
            "planets/rock_effects/debri_small_desert_n15.png"
        );

        let mid = item_rock::particle_frame_paths(
            PlanetType::Arctic,
            item_rock::RockParticleKind::Mid,
            &mut rng,
        );
        assert_eq!(mid.len(), 8);
        assert!(mid[0].starts_with("planets/rock_effects/debri_mid"));
        assert!(mid[0].ends_with("_arctic_n00.png"));

        let large_desert = item_rock::turrent_frame_paths(PlanetType::Desert, &mut rng);
        assert_eq!(
            large_desert[0],
            "planets/rock_effects/debri_large0_desert_n00.png"
        );
        assert_eq!(large_desert.len(), 12);
    }

    #[test]
    fn map_object_and_rock_death_counts_match_original_ranges() {
        let mut rng = CombatRng::default();
        for _ in 0..32 {
            assert!((10..=17).contains(&map_object::death_unit_particle_count(&mut rng)));
            assert!((20..=39).contains(&map_object::death_spark_count(&mut rng)));
            assert!((12..=17).contains(&item_rock::small_particle_count(&mut rng)));
            assert!((4..=6).contains(&item_rock::mid_particle_count(&mut rng)));
            assert!((0..=1).contains(&item_rock::large_turrent_count(&mut rng)));
            assert!((12..=17).contains(&item_rock::turrent_end_small_particle_count(&mut rng)));
        }

        let unit = map_object::death_unit_particle_trajectory(Vec2::ZERO, 65.0, 55.0, &mut rng);
        assert!((1.4..=2.3).contains(&unit.final_time));
        assert!((1.1..=1.39).contains(&unit.rise));
        let large = item_rock::turrent_trajectory(Vec2::ZERO, 140.0, 140.0, &mut rng);
        assert!((1.5..=2.4).contains(&large.final_time));
        assert!((1.1..=3.09).contains(&large.rise));
    }

    #[test]
    fn bridge_turrent_spawn_points_follow_original_axis_stride() {
        let mut rng = CombatRng::default();
        let vertical = BridgeFootprint {
            x: 2,
            y: 3,
            building: BuildingType::BridgeVert,
            extra_links: 2,
        };
        let vertical_points = buildings::bridge_turrent_spawn_points(vertical, &mut rng);
        assert!(!vertical_points.is_empty());
        for point in vertical_points {
            assert!((48.0..80.0).contains(&point.x));
            assert!((69.0..144.0).contains(&point.y));
        }

        let horizontal = BridgeFootprint {
            x: 2,
            y: 3,
            building: BuildingType::BridgeHorz,
            extra_links: 2,
        };
        let horizontal_points = buildings::bridge_turrent_spawn_points(horizontal, &mut rng);
        assert!(!horizontal_points.is_empty());
        for point in horizontal_points {
            assert!((53.0..128.0).contains(&point.x));
            assert!((64.0..96.0).contains(&point.y));
        }
    }

    #[test]
    fn bridge_turrent_and_rock_particle_timing_matches_original_ranges() {
        let mut rng = CombatRng::default();
        for _ in 0..32 {
            let turrent =
                buildings::bridge_turrent_trajectory(Vec2::new(100.0, 50.0), false, &mut rng);
            assert!((1.5..=2.4).contains(&turrent.final_time));
            assert!([1.0, 2.0, 3.0].contains(&turrent.rise));
            assert!(turrent.start.distance(turrent.end) > 0.0);

            let rock = buildings::bridge_rock_particle_trajectory(Vec2::new(100.0, 50.0), &mut rng);
            assert!((1.1..=2.0).contains(&rock.final_time));
            assert!((1.1..=1.39).contains(&rock.rise));

            let count = buildings::bridge_end_particle_count(&mut rng);
            assert!((12..=17).contains(&count));
        }

        assert_eq!(buildings::bridge_turrent_arc_size(1.1, 1.5, 0.0), 1.0);
        assert!(buildings::bridge_turrent_arc_size(1.1, 1.5, 0.75) > 1.0);
        assert!((-239.0..=240.0).contains(&buildings::turrent_spin_degrees_per_sec(&mut rng)));
        assert_eq!(buildings::bridge_rock_particle_arc_size(1.1, 1.1, 1.1), 1.0);
    }

    #[test]
    fn destroyed_bridge_kills_intersecting_robots_and_vehicles_only() {
        let bridge = BridgeFootprint {
            x: 1,
            y: 1,
            building: BuildingType::BridgeVert,
            extra_links: 0,
        };
        let inside_bridge = Vec2::new(3.0 * TILE_SIZE, -3.0 * TILE_SIZE);
        let outside_bridge = Vec2::new(7.0 * TILE_SIZE, -3.0 * TILE_SIZE);
        let size = Vec2::splat(12.0);
        let live_robot = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let live_vehicle = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let live_cannon = ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gun), 100);
        let destroyed_robot = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 0);

        assert!(bridge_destroy_kills_unit(
            ObjectKind::Robot(RobotType::Grunt),
            live_robot,
            inside_bridge,
            size,
            bridge
        ));
        assert!(bridge_destroy_kills_unit(
            ObjectKind::Vehicle(VehicleType::Jeep),
            live_vehicle,
            inside_bridge,
            size,
            bridge
        ));
        assert!(!bridge_destroy_kills_unit(
            ObjectKind::Cannon(CannonType::Gun),
            live_cannon,
            inside_bridge,
            size,
            bridge
        ));
        assert!(!bridge_destroy_kills_unit(
            ObjectKind::Robot(RobotType::Grunt),
            live_robot,
            outside_bridge,
            size,
            bridge
        ));
        assert!(!bridge_destroy_kills_unit(
            ObjectKind::Robot(RobotType::Grunt),
            destroyed_robot,
            inside_bridge,
            size,
            bridge
        ));
    }

    #[test]
    fn destroyed_fort_eliminates_only_non_null_owner_like_original() {
        assert_eq!(
            eliminated_team_for_destroyed_fort(
                ObjectKind::Building(BuildingType::FortFront),
                TeamType::Blue,
                true
            ),
            Some(TeamType::Blue)
        );
        assert_eq!(
            eliminated_team_for_destroyed_fort(
                ObjectKind::Building(BuildingType::FortBack),
                TeamType::Null,
                true
            ),
            None
        );
        assert_eq!(
            eliminated_team_for_destroyed_fort(
                ObjectKind::Building(BuildingType::RobotFactory),
                TeamType::Blue,
                true
            ),
            None
        );
    }

    #[test]
    fn fort_elimination_destroys_team_objects_but_skips_map_items() {
        assert!(object_should_be_destroyed_by_fort_elimination(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Blue,
            false,
            TeamType::Blue
        ));
        assert!(object_should_be_destroyed_by_fort_elimination(
            ObjectKind::Building(BuildingType::RobotFactory),
            TeamType::Blue,
            false,
            TeamType::Blue
        ));
        assert!(!object_should_be_destroyed_by_fort_elimination(
            ObjectKind::MapItem(ItemType::Flag as u8),
            TeamType::Blue,
            false,
            TeamType::Blue
        ));
        assert!(!object_should_be_destroyed_by_fort_elimination(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Red,
            false,
            TeamType::Blue
        ));
    }

    #[test]
    fn no_alive_combat_units_marks_team_fort_for_elimination() {
        let snapshots = vec![
            (
                1,
                ObjectKind::Building(BuildingType::FortFront),
                TeamType::Blue,
                false,
            ),
            (2, ObjectKind::Robot(RobotType::Grunt), TeamType::Blue, true),
            (
                3,
                ObjectKind::Building(BuildingType::RobotFactory),
                TeamType::Blue,
                false,
            ),
        ];

        assert!(!team_has_alive_combat_unit(&snapshots, TeamType::Blue));
        assert!(team_has_fort(&snapshots, TeamType::Blue));
    }

    #[test]
    fn minimap_click_center_maps_to_world_center() {
        let map = ZMap::parse(include_bytes!("../maps/p02_bb_orig01.map")).unwrap();
        let layout = HudLayout::for_map(&map);
        let window_size = Vec2::new(800.0, 600.0);
        let minimap_center = layout.render_offset + layout.render_size * 0.5;
        let screen_pos = window_size + layout.bottom_right_offset_for_minimap_local(minimap_center);

        let world = layout
            .minimap_screen_to_world(screen_pos, window_size)
            .expect("center is inside minimap");

        assert!((world.x - layout.map_pixel_size.x * 0.5).abs() < 0.01);
        assert!((world.y + layout.map_pixel_size.y * 0.5).abs() < 0.01);
    }

    #[test]
    fn camera_center_clamps_to_original_view_shift_bounds() {
        let map_size = Vec2::new(1024.0, 1376.0);
        let view_size = Vec2::new(700.0, 564.0);

        let clamped = clamp_camera_center(Vec2::new(-500.0, 500.0), map_size, view_size);
        assert_eq!(clamped, Vec2::new(350.0, -282.0));

        let clamped = clamp_camera_center(Vec2::new(5000.0, -5000.0), map_size, view_size);
        assert_eq!(clamped, Vec2::new(674.0, -1094.0));
    }

    #[test]
    fn hud_button_positions_match_original_offsets() {
        let window_size = Vec2::new(800.0, 600.0);

        let menu = hud_button_spec(HudButtonKind::Menu).unwrap();
        assert_eq!(
            hud_button_screen_top_left(menu, window_size),
            Vec2::new(634.0, 574.0)
        );

        let r_button = hud_button_spec(HudButtonKind::R).unwrap();
        assert_eq!(
            hud_button_screen_top_left(r_button, window_size),
            Vec2::new(8.0, 574.0)
        );
    }

    #[test]
    fn hud_buttons_route_original_unit_type_selection_commands() {
        assert_eq!(
            hud_command_for_button(HudButtonKind::R),
            Some(HudCommand::SelectGroup(ObjectSelectionGroup::Robot))
        );
        assert_eq!(
            hud_command_for_button(HudButtonKind::V),
            Some(HudCommand::SelectGroup(ObjectSelectionGroup::Vehicle))
        );
        assert_eq!(
            hud_command_for_button(HudButtonKind::G),
            Some(HudCommand::SelectGroup(ObjectSelectionGroup::Cannon))
        );
        assert_eq!(hud_command_for_button(HudButtonKind::D), None);
    }

    #[test]
    fn orderly_selection_cycles_team_objects_and_skips_destroyed() {
        let candidates = [
            SelectionCandidate {
                ref_id: 4,
                kind: ObjectKind::Robot(RobotType::Grunt),
                team: TeamType::Red,
                destroyed: false,
                leader_ref_id: None,
            },
            SelectionCandidate {
                ref_id: 8,
                kind: ObjectKind::Robot(RobotType::Tough),
                team: TeamType::Blue,
                destroyed: false,
                leader_ref_id: None,
            },
            SelectionCandidate {
                ref_id: 12,
                kind: ObjectKind::Robot(RobotType::Sniper),
                team: TeamType::Red,
                destroyed: true,
                leader_ref_id: None,
            },
            SelectionCandidate {
                ref_id: 16,
                kind: ObjectKind::Robot(RobotType::Psycho),
                team: TeamType::Red,
                destroyed: false,
                leader_ref_id: None,
            },
            SelectionCandidate {
                ref_id: 20,
                kind: ObjectKind::Vehicle(VehicleType::Jeep),
                team: TeamType::Red,
                destroyed: false,
                leader_ref_id: None,
            },
        ];

        assert_eq!(
            next_ordered_selection_ref(
                &candidates,
                ObjectSelectionGroup::Robot,
                TeamType::Red,
                None
            ),
            Some(4)
        );
        assert_eq!(
            next_ordered_selection_ref(
                &candidates,
                ObjectSelectionGroup::Robot,
                TeamType::Red,
                Some(4)
            ),
            Some(16)
        );
        assert_eq!(
            next_ordered_selection_ref(
                &candidates,
                ObjectSelectionGroup::Robot,
                TeamType::Red,
                Some(16)
            ),
            Some(4)
        );
    }

    #[test]
    fn movement_direction_uses_original_sector_table() {
        assert_eq!(direction_index_from_delta(Vec2::new(1.0, 0.0)), Some(0));
        assert_eq!(rotation_for_direction(0), 0);

        assert_eq!(direction_index_from_delta(Vec2::new(0.0, -1.0)), Some(6));
        assert_eq!(rotation_for_direction(6), 270);

        assert_eq!(direction_index_from_delta(Vec2::new(-1.0, 0.0)), Some(4));
        assert_eq!(rotation_for_direction(4), 180);

        assert_eq!(direction_index_from_delta(Vec2::new(0.0, 1.0)), Some(2));
        assert_eq!(rotation_for_direction(2), 90);
    }

    #[test]
    fn selected_hud_assets_match_original_names() {
        assert_eq!(
            selected_hud_asset_name(
                ObjectKind::Robot(RobotType::Grunt),
                TeamType::Blue,
                HudSelectedObjectSlot::Icon
            ),
            Some("icon_grunt_blue.png".to_string())
        );
        assert_eq!(
            selected_hud_asset_name(
                ObjectKind::Vehicle(VehicleType::MissileLauncher),
                TeamType::Yellow,
                HudSelectedObjectSlot::Label
            ),
            Some("label_missile_launcher.png".to_string())
        );
        assert_eq!(
            selected_hud_asset_name(
                ObjectKind::Cannon(CannonType::MissileCannon),
                TeamType::Green,
                HudSelectedObjectSlot::Icon
            ),
            Some("icon_missile_cannon_green.png".to_string())
        );
    }

    #[test]
    fn grenade_hud_asset_names_match_original_names() {
        assert_eq!(
            grenade_icon_asset_name(TeamType::Null),
            "icon_grenade_null.png"
        );
        assert_eq!(
            grenade_icon_asset_name(TeamType::Red),
            "icon_grenade_red.png"
        );
        assert_eq!(
            grenade_icon_asset_name(TeamType::Blue),
            "icon_grenade_blue.png"
        );
        assert_eq!(
            grenade_icon_asset_name(TeamType::Green),
            "icon_grenade_green.png"
        );
        assert_eq!(
            grenade_icon_asset_name(TeamType::Yellow),
            "icon_grenade_yellow.png"
        );
    }

    #[test]
    fn default_factory_production_matches_original_build_list_heads() {
        assert_eq!(
            default_production_unit(ObjectKind::Building(BuildingType::RobotFactory), 0),
            Some(ObjectKind::Robot(RobotType::Grunt))
        );
        assert_eq!(
            default_production_unit(ObjectKind::Building(BuildingType::VehicleFactory), 0),
            Some(ObjectKind::Vehicle(VehicleType::Jeep))
        );
        assert_eq!(
            default_production_unit(ObjectKind::Building(BuildingType::FortBack), 0),
            Some(ObjectKind::Robot(RobotType::Grunt))
        );

        assert!(unit_in_default_build_list(
            BuildingType::RobotFactory,
            5,
            ObjectKind::Cannon(CannonType::MissileCannon)
        ));
        assert!(!unit_in_default_build_list(
            BuildingType::RobotFactory,
            0,
            ObjectKind::Robot(RobotType::Laser)
        ));
    }

    #[test]
    fn production_duration_uses_original_zone_and_health_modifiers() {
        assert_eq!(
            production_duration(ObjectKind::Robot(RobotType::Grunt), 100.0, 100.0, 0.0),
            Some(72.0)
        );
        assert_eq!(
            production_duration(ObjectKind::Robot(RobotType::Grunt), 100.0, 100.0, 1.0),
            Some(36.0)
        );
        assert_eq!(
            production_duration(ObjectKind::Robot(RobotType::Grunt), 50.0, 100.0, 0.0),
            Some(117.0)
        );
    }

    #[test]
    fn reset_build_time_applies_zone_ownage_without_restarting_progress() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = initial_production_for_building(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            TeamType::Red,
            stats,
        )
        .expect("robot factory starts default production");
        production.elapsed = 20.0;

        assert_eq!(production.duration, 72.0);
        assert!(reset_build_time(&mut production, stats, 1.0));
        assert_eq!(production.zone_ownage, 1.0);
        assert_eq!(production.duration, 36.0);
        assert_eq!(production.elapsed, 20.0);
        assert!(!reset_build_time(&mut production, stats, 1.0));
    }

    #[test]
    fn zone_ownage_counts_owned_flags_like_original_server() {
        let zones = ZoneOwnership {
            owners: vec![TeamType::Red, TeamType::Blue, TeamType::Red],
            links: vec![
                ZoneLink {
                    zone_index: 0,
                    flag_ref_id: 10,
                    building_refs: Vec::new(),
                },
                ZoneLink {
                    zone_index: 1,
                    flag_ref_id: 20,
                    building_refs: Vec::new(),
                },
                ZoneLink {
                    zone_index: 2,
                    flag_ref_id: 30,
                    building_refs: Vec::new(),
                },
            ],
        };

        assert!((zones.team_zone_ownage(TeamType::Red) - 2.0 / 3.0).abs() < f32::EPSILON);
        assert!((zones.team_zone_ownage(TeamType::Blue) - 1.0 / 3.0).abs() < f32::EPSILON);
        assert_eq!(zones.team_zone_ownage(TeamType::Green), 0.0);
        assert_eq!(
            (ZoneOwnership {
                owners: Vec::new(),
                links: Vec::new(),
            })
            .team_zone_ownage(TeamType::Red),
            0.0
        );
    }

    #[test]
    fn production_world_points_match_original_building_offsets() {
        assert_eq!(
            production_world_points(BuildingType::RobotFactory, 10, 20),
            Some((Vec2::new(203.0, -373.0), Vec2::new(203.0, -416.0)))
        );
        assert_eq!(
            production_world_points(BuildingType::VehicleFactory, 10, 20),
            Some((Vec2::new(192.0, -368.0), Vec2::new(192.0, -416.0)))
        );
        assert_eq!(
            production_world_points(BuildingType::FortFront, 10, 20),
            Some((Vec2::new(240.0, -448.0), Vec2::new(240.0, -528.0)))
        );
        assert_eq!(
            production_world_points(BuildingType::FortBack, 10, 20),
            Some((Vec2::new(240.0, -352.0), Vec2::new(240.0, -304.0)))
        );
    }

    #[test]
    fn robot_group_metadata_marks_leader_and_minions() {
        let kind = ObjectKind::Robot(RobotType::Grunt);

        assert_eq!(
            world_objects::robot_group_for(kind, 10, Some(10)),
            Some(RobotGroup { leader_ref_id: 10 })
        );
        assert_eq!(
            world_objects::robot_group_for(kind, 11, Some(10)),
            Some(RobotGroup { leader_ref_id: 10 })
        );
        assert_eq!(
            world_objects::robot_group_for(ObjectKind::Vehicle(VehicleType::Jeep), 20, Some(10)),
            None
        );
    }

    #[test]
    fn orderly_selection_skips_robot_group_minions() {
        let candidates = [
            SelectionCandidate {
                ref_id: 10,
                kind: ObjectKind::Robot(RobotType::Grunt),
                team: TeamType::Red,
                destroyed: false,
                leader_ref_id: Some(10),
            },
            SelectionCandidate {
                ref_id: 11,
                kind: ObjectKind::Robot(RobotType::Grunt),
                team: TeamType::Red,
                destroyed: false,
                leader_ref_id: Some(10),
            },
            SelectionCandidate {
                ref_id: 12,
                kind: ObjectKind::Robot(RobotType::Grunt),
                team: TeamType::Red,
                destroyed: false,
                leader_ref_id: Some(10),
            },
            SelectionCandidate {
                ref_id: 20,
                kind: ObjectKind::Robot(RobotType::Psycho),
                team: TeamType::Red,
                destroyed: false,
                leader_ref_id: Some(20),
            },
        ];

        assert_eq!(
            next_ordered_selection_ref(
                &candidates,
                ObjectSelectionGroup::Robot,
                TeamType::Red,
                None
            ),
            Some(10)
        );
        assert_eq!(
            next_ordered_selection_ref(
                &candidates,
                ObjectSelectionGroup::Robot,
                TeamType::Red,
                Some(10)
            ),
            Some(20)
        );
        assert_eq!(
            next_ordered_selection_ref(
                &candidates,
                ObjectSelectionGroup::Robot,
                TeamType::Red,
                Some(20)
            ),
            Some(10)
        );
    }

    #[test]
    fn produced_object_count_uses_original_robot_group_amounts() {
        assert_eq!(
            produced_object_count(ObjectKind::Robot(RobotType::Grunt)),
            3
        );
        assert_eq!(
            produced_object_count(ObjectKind::Robot(RobotType::Tough)),
            2
        );
        assert_eq!(
            produced_object_count(ObjectKind::Robot(RobotType::Laser)),
            4
        );
        assert_eq!(
            produced_object_count(ObjectKind::Vehicle(VehicleType::Jeep)),
            1
        );
        assert_eq!(
            produced_object_count(ObjectKind::Cannon(CannonType::Gatling)),
            0
        );
    }

    #[test]
    fn building_production_advances_queue_like_original_reset() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = initial_production_for_building(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            TeamType::Red,
            stats,
        )
        .expect("robot factory starts default production");

        assert_eq!(
            production.current,
            Some(ObjectKind::Robot(RobotType::Grunt))
        );
        assert_eq!(production.queue.len(), 1);

        assert!(add_production_queue(
            &mut production,
            BuildingType::RobotFactory,
            0,
            ObjectKind::Cannon(CannonType::Gatling),
            true
        ));

        let duration = production.duration;
        let completed = advance_production(&mut production, duration + 0.1, stats);

        assert_eq!(completed, vec![ObjectKind::Robot(RobotType::Grunt)]);
        assert_eq!(
            production.ready_units,
            vec![ObjectKind::Robot(RobotType::Grunt)]
        );
        assert_eq!(
            production.current,
            Some(ObjectKind::Cannon(CannonType::Gatling))
        );
        assert_eq!(
            production.queue.front().copied(),
            Some(ObjectKind::Robot(RobotType::Grunt))
        );
    }

    #[test]
    fn production_unit_limit_pauses_completion_until_slot_opens_like_original() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = initial_production_for_building(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            TeamType::Red,
            stats,
        )
        .expect("robot factory starts default production");
        let duration = production.duration;

        assert!(team_unit_limit_reached(DEFAULT_MAX_UNITS_PER_TEAM));
        assert!(!team_unit_limit_reached(DEFAULT_MAX_UNITS_PER_TEAM - 1));

        let completed =
            advance_production_with_unit_limit(&mut production, duration + 10.0, stats, true);
        assert!(completed.is_empty());
        assert_eq!(production.status, BuildingProductionStatus::Building);
        assert!(production.unit_limit_reached);
        assert_eq!(
            building_production_window_state(&production, stats),
            BuildingProductionStatus::Paused
        );
        assert_eq!(production.elapsed, duration);
        assert_eq!(
            production.current,
            Some(ObjectKind::Robot(RobotType::Grunt))
        );

        let completed = advance_production_with_unit_limit(&mut production, 0.0, stats, false);
        assert_eq!(completed, vec![ObjectKind::Robot(RobotType::Grunt)]);
        assert!(!production.unit_limit_reached);
        assert_eq!(
            production.ready_units,
            vec![ObjectKind::Robot(RobotType::Grunt)]
        );
        assert_eq!(production.status, BuildingProductionStatus::Building);
    }

    #[test]
    fn cannon_production_stores_for_placement_instead_of_spawning() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = initial_production_for_building(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            TeamType::Red,
            stats,
        )
        .expect("robot factory starts default production");

        assert!(start_production(
            &mut production,
            ObjectKind::Cannon(CannonType::Gatling),
            stats
        ));

        let duration = production.duration;
        let completed = advance_production(&mut production, duration, stats);

        assert_eq!(completed, vec![ObjectKind::Cannon(CannonType::Gatling)]);
        assert!(production.ready_units.is_empty());
        assert!(production.stored_cannons.is_empty());
        assert!(can_store_cannon_in_zone(0));
        assert!(store_built_cannon(
            &mut production,
            ObjectKind::Cannon(CannonType::Gatling)
        ));
        assert_eq!(
            production.stored_cannons,
            vec![ObjectKind::Cannon(CannonType::Gatling)]
        );
        assert_eq!(
            produced_object_count(ObjectKind::Cannon(CannonType::Gatling)),
            0
        );
    }

    #[test]
    fn stored_cannons_respect_original_limit() {
        let stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let mut production = initial_production_for_building(
            ObjectKind::Building(BuildingType::RobotFactory),
            0,
            TeamType::Red,
            stats,
        )
        .expect("robot factory starts default production");

        for _ in 0..MAX_STORED_CANNONS {
            assert!(store_built_cannon(
                &mut production,
                ObjectKind::Cannon(CannonType::Gatling)
            ));
        }
        assert!(!store_built_cannon(
            &mut production,
            ObjectKind::Cannon(CannonType::Gun)
        ));
        assert_eq!(production.stored_cannons.len(), MAX_STORED_CANNONS);

        assert!(remove_stored_cannon(
            &mut production,
            ObjectKind::Cannon(CannonType::Gatling)
        ));
        assert_eq!(production.stored_cannons.len(), MAX_STORED_CANNONS - 1);
    }

    #[test]
    fn stored_cannon_placement_uses_original_zone_inset() {
        let map = placement_test_map();
        let production = placement_test_production(vec![ObjectKind::Cannon(CannonType::Gatling)]);

        assert!(can_place_stored_cannon(
            &production,
            ObjectKind::Cannon(CannonType::Gatling),
            7,
            TeamType::Red,
            Some(0),
            (2, 2),
            &map,
            &[]
        ));
        assert!(!can_place_stored_cannon(
            &production,
            ObjectKind::Cannon(CannonType::Gatling),
            7,
            TeamType::Red,
            Some(0),
            (0, 0),
            &map,
            &[]
        ));
        assert!(!can_place_stored_cannon(
            &production,
            ObjectKind::Cannon(CannonType::Gatling),
            7,
            TeamType::Red,
            None,
            (2, 2),
            &map,
            &[]
        ));
    }

    #[test]
    fn stored_cannon_placement_removes_storage_only_after_validation() {
        let map = placement_test_map();
        let mut production = placement_test_production(vec![ObjectKind::Cannon(CannonType::Gun)]);

        assert!(!place_stored_cannon(
            &mut production,
            ObjectKind::Cannon(CannonType::Gun),
            7,
            TeamType::Red,
            Some(0),
            (0, 0),
            &map,
            &[]
        ));
        assert_eq!(
            production.stored_cannons,
            vec![ObjectKind::Cannon(CannonType::Gun)]
        );

        assert!(place_stored_cannon(
            &mut production,
            ObjectKind::Cannon(CannonType::Gun),
            7,
            TeamType::Red,
            Some(0),
            (2, 2),
            &map,
            &[]
        ));
        assert!(production.stored_cannons.is_empty());
        assert_eq!(production.status, BuildingProductionStatus::Building);
    }

    #[test]
    fn stored_cannon_placement_blocks_existing_cannons_but_ignores_vehicles() {
        let map = placement_test_map();
        let production = placement_test_production(vec![ObjectKind::Cannon(CannonType::Howitzer)]);
        let blockers = vec![
            PlacementObstacle {
                ref_id: 11,
                kind: ObjectKind::Cannon(CannonType::Gatling),
                center: cannon_spawn_center(2, 2),
                size: Vec2::splat(TILE_SIZE * 2.0),
            },
            PlacementObstacle {
                ref_id: 12,
                kind: ObjectKind::Vehicle(VehicleType::Jeep),
                center: cannon_spawn_center(4, 4),
                size: Vec2::splat(TILE_SIZE * 2.0),
            },
        ];

        assert!(!can_place_stored_cannon(
            &production,
            ObjectKind::Cannon(CannonType::Howitzer),
            7,
            TeamType::Red,
            Some(0),
            (2, 2),
            &map,
            &blockers
        ));
        assert!(can_place_stored_cannon(
            &production,
            ObjectKind::Cannon(CannonType::Howitzer),
            7,
            TeamType::Red,
            Some(0),
            (4, 4),
            &map,
            &blockers[1..]
        ));
    }

    #[test]
    fn fort_turret_tiles_match_original_non_ejectable_cannon_slots() {
        let mut map = placement_test_map();
        map.objects.push(original::map::MapObject {
            x: 2,
            y: 3,
            owner: TeamType::Red,
            object_type: original::map::MapObjectType::Building,
            object_id: BuildingType::FortFront as u8,
            building_level: 0,
            extra_links: 0,
            health_percent: 100,
        });

        assert!(area_is_fort_turret_tile(&map, 3, 3));
        assert!(area_is_fort_turret_tile(&map, 9, 6));
        assert!(!area_is_fort_turret_tile(&map, 4, 3));
    }

    #[test]
    fn zone_wide_cannon_capacity_blocks_fifth_cannon() {
        assert!(can_store_cannon_in_zone(0));
        assert!(can_store_cannon_in_zone(MAX_STORED_CANNONS - 1));
        assert!(!can_store_cannon_in_zone(MAX_STORED_CANNONS));
        assert!(!can_store_cannon_in_zone(MAX_STORED_CANNONS + 1));
    }

    fn placement_test_map() -> ZMap {
        ZMap {
            basics: original::map::MapBasics {
                width: 10,
                height: 10,
                map_name: "placement".to_string(),
                player_count: 2,
                object_count: 0,
                terrain_type: PlanetType::Desert,
                zone_count: 1,
            },
            zones: vec![original::map::MapZone {
                x: 0,
                y: 0,
                w: 10,
                h: 10,
            }],
            objects: Vec::new(),
            tiles: Vec::new(),
        }
    }

    fn placement_test_production(stored_cannons: Vec<ObjectKind>) -> BuildingProduction {
        BuildingProduction {
            status: BuildingProductionStatus::Place,
            current: Some(ObjectKind::Robot(RobotType::Grunt)),
            queue: std::collections::VecDeque::new(),
            elapsed: 0.0,
            duration: 1.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            ready_units: Vec::new(),
            stored_cannons,
        }
    }
}
