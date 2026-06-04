use bevy::prelude::*;
use std::collections::VecDeque;

use crate::original::map::{MapObjectType, ZMap};
use crate::original::objects::{BuildingType, ObjectKind, RobotType, VehicleType};
use crate::original::settings::{
    object_attack_damage, object_attack_radius, object_attack_speed, object_damage_chance,
    object_damage_radius, object_max_health, object_missile_speed, object_move_speed,
    object_snipe_chance,
};
use crate::original::tileinfo::PaletteTileInfo;
use crate::original::types::TeamType;
use crate::render::atlas::{
    FactoryOverlayKind, MobileSpriteRole, RadarOverlayKind, RepairOverlayKind, SpriteFrame,
};

#[derive(Resource)]
pub(crate) struct CurrentMap(pub(crate) ZMap);

#[derive(Resource)]
pub(crate) struct CurrentTileInfo(pub(crate) Vec<PaletteTileInfo>);

#[derive(Resource)]
pub(crate) struct PlanetAtlas {
    pub(crate) desert: Handle<Image>,
    pub(crate) volcanic: Handle<Image>,
    pub(crate) arctic: Handle<Image>,
    pub(crate) jungle: Handle<Image>,
    pub(crate) city: Handle<Image>,
    pub(crate) layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource)]
pub(crate) struct RockAtlas {
    pub(crate) desert: Handle<Image>,
    pub(crate) volcanic: Handle<Image>,
    pub(crate) arctic: Handle<Image>,
    pub(crate) jungle: Handle<Image>,
    pub(crate) city: Handle<Image>,
    pub(crate) layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource)]
pub(crate) struct HudAssets {
    pub(crate) side_panel: Handle<Image>,
    pub(crate) side_filler: Handle<Image>,
    pub(crate) bottom_left: Handle<Image>,
    pub(crate) bottom_center: Handle<Image>,
    pub(crate) bottom_right: Handle<Image>,
    pub(crate) health_full: Handle<Image>,
    pub(crate) health_lost: Handle<Image>,
    pub(crate) health_empty: Handle<Image>,
    pub(crate) grenade_icons: Vec<(TeamType, Handle<Image>)>,
    pub(crate) fort_under_attack_message: Handle<Image>,
    pub(crate) font: Handle<Font>,
    pub(crate) buttons: Vec<HudButtonImages>,
}

#[derive(Resource)]
pub(crate) struct ProductionUiAssets {
    pub(crate) base: Handle<Image>,
    pub(crate) base_expanded: Handle<Image>,
    pub(crate) labels: Vec<Handle<Image>>,
    pub(crate) state_labels: Vec<[Handle<Image>; 2]>,
    pub(crate) buttons: Vec<ProductionButtonImages>,
    pub(crate) queue_item_button: ProductionButtonImages,
    pub(crate) object_button: ProductionButtonImages,
    pub(crate) full_selector_frame: ProductionFullSelectorFrameAssets,
    pub(crate) progress_bar: Handle<Image>,
    pub(crate) progress_yellow: Handle<Image>,
    pub(crate) font: Handle<Font>,
}

#[derive(Clone)]
pub(crate) struct ProductionFullSelectorFrameAssets {
    pub(crate) top_left: Handle<Image>,
    pub(crate) top_right: Handle<Image>,
    pub(crate) bottom_left: Handle<Image>,
    pub(crate) bottom_right: Handle<Image>,
    pub(crate) top: Handle<Image>,
    pub(crate) bottom: Handle<Image>,
    pub(crate) left: Handle<Image>,
    pub(crate) right: Handle<Image>,
}

#[derive(Clone)]
pub(crate) struct ProductionButtonImages {
    pub(crate) normal: Handle<Image>,
    pub(crate) pressed: Handle<Image>,
}

#[derive(Clone)]
pub(crate) struct HudButtonImages {
    pub(crate) active: Handle<Image>,
    pub(crate) inactive: Handle<Image>,
    pub(crate) pressed: Handle<Image>,
}

#[derive(Component)]
pub(crate) struct MainCamera;

#[derive(Component)]
pub(crate) struct HudCamera;

#[derive(Clone, Copy, Component)]
#[allow(dead_code)]
pub(crate) struct GameObjectEntity {
    pub(crate) ref_id: u32,
    pub(crate) kind: ObjectKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(crate) struct RobotGroup {
    pub(crate) leader_ref_id: u32,
}

#[derive(Clone, Copy, Component)]
#[allow(dead_code)]
pub(crate) struct ObjectTeam(pub(crate) TeamType);

#[derive(Clone, Copy, Component)]
#[allow(dead_code)]
pub(crate) struct HealthPercent(pub(crate) i32);

#[derive(Clone, Copy, Component)]
#[allow(dead_code)]
pub(crate) struct ObjectStats {
    pub(crate) max_health: f32,
    pub(crate) health: f32,
    pub(crate) move_speed: f32,
    pub(crate) attack_radius: f32,
    pub(crate) attack_damage: f32,
    pub(crate) damage_chance: f32,
    pub(crate) damage_radius: f32,
    pub(crate) missile_speed: f32,
    pub(crate) attack_speed: f32,
    pub(crate) snipe_chance: f32,
    pub(crate) attacked_only_by_explosives: bool,
    pub(crate) cannon_ejectable: bool,
}

impl ObjectStats {
    pub(crate) fn from_kind(kind: ObjectKind, health_percent: i32) -> Self {
        let max_health = object_max_health(kind);
        let health = (health_percent.clamp(0, 100) as f32 * max_health / 100.0) as i32 as f32;

        Self {
            max_health,
            health,
            move_speed: object_move_speed(kind),
            attack_radius: object_attack_radius(kind),
            attack_damage: object_attack_damage(kind),
            damage_chance: object_damage_chance(kind),
            damage_radius: object_damage_radius(kind),
            missile_speed: object_missile_speed(kind),
            attack_speed: object_attack_speed(kind),
            snipe_chance: object_snipe_chance(kind),
            attacked_only_by_explosives: matches!(
                kind,
                ObjectKind::Rock
                    | ObjectKind::Bridge(_)
                    | ObjectKind::Building(_)
                    | ObjectKind::MapItem(_)
            ),
            cannon_ejectable: true,
        }
    }

    pub(crate) fn can_attack(self) -> bool {
        self.attack_damage > 0.0 && self.health > 0.0
    }

    pub(crate) fn destroyed(self) -> bool {
        self.health <= 0.0
    }

    pub(crate) fn has_explosive_damage(self) -> bool {
        self.damage_radius > 0.0 || self.missile_speed > 0.0
    }
}

pub(crate) fn can_eject_drivers(kind: ObjectKind, stats: ObjectStats) -> bool {
    !stats.destroyed()
        && match kind {
            ObjectKind::Vehicle(VehicleType::Apc) => true,
            ObjectKind::Cannon(_) => stats.cannon_ejectable,
            _ => false,
        }
}

pub(crate) fn area_is_fort_turret_tile(map: &ZMap, tx: i32, ty: i32) -> bool {
    map.objects.iter().any(|object| {
        object.object_type == MapObjectType::Building
            && matches!(
                ObjectKind::from_map_parts(object.object_type, object.object_id),
                Ok(ObjectKind::Building(
                    BuildingType::FortFront | BuildingType::FortBack
                ))
            )
            && (tx == object.x as i32 + 1 || tx == object.x as i32 + 7)
            && (ty == object.y as i32 || ty == object.y as i32 + 3)
    })
}

#[derive(Clone, Component)]
pub(crate) struct DriverHealth {
    pub(crate) driver_kind: RobotType,
    pub(crate) driver_healths: Vec<f32>,
    pub(crate) next_attack_cooldowns: Vec<f32>,
}

impl DriverHealth {
    pub(crate) fn new(driver_kind: RobotType, health: f32) -> Self {
        Self::with_driver_healths(driver_kind, vec![health])
    }

    pub(crate) fn with_driver_healths(driver_kind: RobotType, driver_healths: Vec<f32>) -> Self {
        let next_attack_cooldowns = vec![0.0; driver_healths.len()];
        Self::with_driver_states(driver_kind, driver_healths, next_attack_cooldowns)
    }

    pub(crate) fn with_driver_states(
        driver_kind: RobotType,
        driver_healths: Vec<f32>,
        mut next_attack_cooldowns: Vec<f32>,
    ) -> Self {
        next_attack_cooldowns.resize(driver_healths.len(), 0.0);
        Self {
            driver_kind,
            driver_healths,
            next_attack_cooldowns,
        }
    }

    pub(crate) fn lead_health(&self) -> f32 {
        self.driver_healths.first().copied().unwrap_or(0.0)
    }

    pub(crate) fn driver_count(&self) -> usize {
        self.driver_healths
            .len()
            .max(self.next_attack_cooldowns.len())
    }
}

#[derive(Component)]
pub(crate) struct EjectDriversCommand;

#[derive(Component)]
pub(crate) struct JustLeftCannon;

#[derive(Clone, Copy, Component)]
#[allow(dead_code)]
pub(crate) struct MapGridPosition {
    pub(crate) x: u16,
    pub(crate) y: u16,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct RockRenderPiece {
    pub(crate) x: u16,
    pub(crate) y: u16,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct TerrainEffectTile {
    pub(crate) current_tile: u16,
    pub(crate) next_effect_time: f32,
    pub(crate) effect_active: bool,
    pub(crate) water_listed: bool,
}

#[derive(Resource, Default)]
pub(crate) struct TerrainEffectPools {
    pub(crate) water_tiles: Vec<u16>,
    pub(crate) water_effect_tiles: Vec<u16>,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct AmbientBird {
    pub(crate) planet: crate::original::types::PlanetType,
    pub(crate) map_size: Vec2,
    pub(crate) position_map: Vec2,
    pub(crate) fractional_shift: Vec2,
    pub(crate) angle_degrees: f32,
    pub(crate) dangle: f32,
    pub(crate) rise: f32,
    pub(crate) render_frame: usize,
    pub(crate) speed: f32,
    pub(crate) next_render_time: f32,
    pub(crate) last_process_time: f32,
    pub(crate) next_dangle_time: f32,
    pub(crate) next_caw_sound_time: f32,
    pub(crate) next_height_change_time: f32,
    pub(crate) rise_change_end: f32,
    pub(crate) rise_change_start_time: f32,
    pub(crate) rise_change_target: f32,
    pub(crate) rise_change_start: f32,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct HutAnimalSpawner {
    pub(crate) max_animals: usize,
    pub(crate) animal_timer: f32,
    pub(crate) max_timer: f32,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct BridgeFootprint {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) building: BuildingType,
    pub(crate) extra_links: u16,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct BridgeRevivePending {
    pub(crate) bridge: BridgeFootprint,
    pub(crate) timer: f32,
    pub(crate) spawned_effect: bool,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct AutoRepair {
    pub(crate) timer: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductionLevel {
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

impl ProductionLevel {
    pub(crate) fn from_original(value: i8) -> Self {
        match value.clamp(0, 5) {
            0 => Self::Level0,
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            4 => Self::Level4,
            _ => Self::Level5,
        }
    }

    pub(crate) fn original(self) -> i8 {
        match self {
            Self::Level0 => 0,
            Self::Level1 => 1,
            Self::Level2 => 2,
            Self::Level3 => 3,
            Self::Level4 => 4,
            Self::Level5 => 5,
        }
    }
}

impl From<i8> for ProductionLevel {
    fn from(value: i8) -> Self {
        Self::from_original(value)
    }
}

#[derive(Clone, Copy, Component)]
pub(crate) struct BuildingLevel(pub(crate) ProductionLevel);

impl BuildingLevel {
    pub(crate) fn from_original(value: i8) -> Self {
        Self(ProductionLevel::from_original(value))
    }

    pub(crate) fn original(self) -> i8 {
        self.0.original()
    }
}

#[derive(Clone, Copy, Component)]
pub(crate) struct ObjectLayerRef(pub(crate) u32);

#[derive(Clone, Copy, Component)]
pub(crate) struct Selectable {
    pub(crate) radius: f32,
    pub(crate) selection_size: Vec2,
    pub(crate) mobile: bool,
}

#[derive(Component)]
pub(crate) struct MovementPath {
    pub(crate) waypoints: Vec<Vec2>,
    pub(crate) speed: f32,
    pub(crate) attempt_run: bool,
}

impl MovementPath {
    pub(crate) fn new(waypoints: Vec<Vec2>, speed: f32) -> Self {
        Self {
            waypoints,
            speed,
            attempt_run: false,
        }
    }

    pub(crate) fn with_run_attempt(mut self) -> Self {
        self.attempt_run = true;
        self
    }
}

#[derive(Clone, Copy, Component)]
pub(crate) struct MovementStamina {
    pub(crate) max: f32,
    pub(crate) current: f32,
    pub(crate) running: bool,
}

#[derive(Component)]
pub(crate) struct VehicleEffectDropTimer {
    pub(crate) elapsed: f32,
}

#[derive(Component)]
pub(crate) struct AttackTarget {
    pub(crate) ref_id: u32,
    pub(crate) cooldown: f32,
    pub(crate) player_given: bool,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct GrenadeInventory {
    pub(crate) amount: u8,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct GrenadeBox {
    pub(crate) amount: u8,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct PickupGrenadesTarget {
    pub(crate) ref_id: u32,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct EnterTarget {
    pub(crate) ref_id: u32,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct EnterFortTarget {
    pub(crate) ref_id: u32,
    pub(crate) stage: EnterFortStage,
    pub(crate) inside_point: Vec2,
    pub(crate) exit_point: Vec2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnterFortStage {
    GotoEntrance,
    EnterBuilding,
    ExitBuilding,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct CraneRepairTarget {
    pub(crate) ref_id: u32,
    pub(crate) stage: CraneRepairStage,
    pub(crate) center_point: Vec2,
    pub(crate) exit_point: Vec2,
    pub(crate) target_top_left_map: Vec2,
    pub(crate) target_size: Vec2,
    pub(crate) target_is_bridge: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CraneRepairStage {
    GotoEntrance,
    EnterBuilding,
    ExitBuilding,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct UnitRepairTarget {
    pub(crate) ref_id: u32,
    pub(crate) stage: UnitRepairStage,
    pub(crate) center_point: Vec2,
    pub(crate) entrance_point: Vec2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnitRepairStage {
    GotoEntrance,
    Wait,
    EnterBuilding,
    ExitBuilding,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct RepairingUnit {
    pub(crate) building_ref_id: u32,
    pub(crate) exit_point: Vec2,
    pub(crate) remaining: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum BuildingProductionStatus {
    Place,
    Select,
    Building,
    Paused,
}

#[derive(Component)]
pub(crate) struct BuildingProduction {
    pub(crate) status: BuildingProductionStatus,
    pub(crate) current: Option<ObjectKind>,
    pub(crate) queue: VecDeque<ObjectKind>,
    pub(crate) elapsed: f32,
    pub(crate) duration: f32,
    pub(crate) zone_ownage: f32,
    pub(crate) unit_limit_reached: bool,
    pub(crate) ready_units: Vec<ObjectKind>,
    pub(crate) stored_cannons: Vec<ObjectKind>,
}

#[allow(dead_code)]
impl BuildingProduction {
    pub(crate) fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    pub(crate) fn time_left(&self) -> f32 {
        (self.duration - self.elapsed).max(0.0)
    }
}

#[derive(Component)]
pub(crate) struct DamageMissile {
    pub(crate) start: Vec2,
    pub(crate) target: Vec2,
    pub(crate) time_remaining: f32,
    pub(crate) total_time: f32,
    pub(crate) damage: f32,
    pub(crate) radius: f32,
    pub(crate) team: TeamType,
    pub(crate) visual: DamageMissileVisual,
    pub(crate) frames: Vec<Handle<Image>>,
    pub(crate) frame_time: f32,
    pub(crate) frame_elapsed: f32,
    pub(crate) frame: usize,
    pub(crate) visual_rise: f32,
    pub(crate) angle_degrees_per_sec: f32,
    pub(crate) crater: Option<DamageCrater>,
    pub(crate) visual_offset: Vec2,
    pub(crate) smoke_offsets: Vec<Vec2>,
    pub(crate) smoke_time_cursor: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DamageMissileVisual {
    Generic,
    Grenade,
    ToughRocket,
    LightRocket {
        extra_small: u8,
        extra_large: u8,
        xx_large: u8,
    },
    MissileCannon,
    MissileLauncher,
    MapObjectTurrent(u8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DamageCrater {
    pub(crate) is_big: bool,
    pub(crate) chance: f32,
    pub(crate) big_chance: Option<f32>,
}

#[derive(Component)]
pub(crate) struct DirectFireBullet {
    pub(crate) start: Vec2,
    pub(crate) target: Vec2,
    pub(crate) time_remaining: f32,
    pub(crate) total_time: f32,
}

#[derive(Component)]
pub(crate) struct ImageEffectAnimation {
    pub(crate) frames: Vec<Handle<Image>>,
    pub(crate) frame_time: f32,
    pub(crate) elapsed: f32,
    pub(crate) current: usize,
    pub(crate) remaining_advances: Option<usize>,
}

#[derive(Clone, Copy, Component, Default)]
pub(crate) struct DamageCauseTimers {
    pub(crate) fire: f32,
    pub(crate) missile: f32,
}

#[derive(Resource)]
pub(crate) struct CombatRng(pub(crate) u32);

#[derive(Resource)]
pub(crate) struct PassiveEngageTimer(pub(crate) Timer);

#[derive(Resource)]
pub(crate) struct FlagCaptureTimer(pub(crate) Timer);

#[derive(Resource)]
pub(crate) struct NextObjectRefId(pub(crate) u32);

#[derive(Clone)]
pub(crate) struct ZoneLink {
    pub(crate) zone_index: usize,
    pub(crate) flag_ref_id: u32,
    pub(crate) building_refs: Vec<u32>,
}

#[derive(Resource)]
pub(crate) struct ZoneOwnership {
    pub(crate) owners: Vec<TeamType>,
    pub(crate) links: Vec<ZoneLink>,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct MobileSpriteLayer {
    pub(crate) kind: ObjectKind,
    pub(crate) team: TeamType,
    pub(crate) role: MobileSpriteRole,
    pub(crate) rotation: u16,
    pub(crate) frame: usize,
    pub(crate) elapsed: f32,
}

#[derive(Resource)]
pub(crate) struct PassabilityGrid {
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) walkable: Vec<bool>,
    pub(crate) vehicle_walkable: Vec<bool>,
    pub(crate) walk_speed: Vec<f32>,
}

#[derive(Component)]
pub(crate) struct SelectionMarker {
    pub(crate) ref_id: u32,
    pub(crate) offset: Vec2,
}

#[derive(Component)]
pub(crate) struct SelectionHealthBar {
    pub(crate) ref_id: u32,
    pub(crate) offset: Vec2,
}

#[derive(Component)]
pub(crate) struct DestroyedObject;

#[derive(Component)]
pub(crate) struct TargetMarker;

#[derive(Component)]
pub(crate) struct DragSelectionBox;

#[derive(Component)]
pub(crate) struct AtlasAnimation {
    pub(crate) frames: Vec<usize>,
    pub(crate) frame_time: f32,
    pub(crate) elapsed: f32,
    pub(crate) current: usize,
}

#[derive(Component)]
pub(crate) struct RadarOverlayLayer {
    pub(crate) ref_id: u32,
    pub(crate) kind: RadarOverlayKind,
    pub(crate) top_left_map: Vec2,
    pub(crate) frames: Vec<SpriteFrame>,
    pub(crate) frame_time: f32,
    pub(crate) elapsed: f32,
    pub(crate) current: usize,
}

#[derive(Component)]
pub(crate) struct RepairOverlayLayer {
    pub(crate) ref_id: u32,
    pub(crate) kind: RepairOverlayKind,
    pub(crate) top_left_map: Vec2,
    pub(crate) frames: Vec<SpriteFrame>,
    pub(crate) frame_time: f32,
    pub(crate) elapsed: f32,
    pub(crate) current: usize,
}

#[derive(Component)]
pub(crate) struct FactoryOverlayLayer {
    pub(crate) ref_id: u32,
    pub(crate) kind: FactoryOverlayKind,
    pub(crate) top_left_map: Vec2,
    pub(crate) frames: Vec<SpriteFrame>,
    pub(crate) frame_time: f32,
    pub(crate) elapsed: f32,
    pub(crate) current: usize,
}

#[derive(Default, Resource)]
pub(crate) struct SelectionState {
    pub(crate) selected_refs: Vec<u32>,
}

#[derive(Default, Resource)]
pub(crate) struct MouseCommandState {
    pub(crate) left_start: Option<Vec2>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObjectSelectionGroup {
    Robot,
    Vehicle,
    Cannon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HudCommand {
    SelectGroup(ObjectSelectionGroup),
    BeginBuildingAction,
    JumpToObject(u32),
}

#[derive(Default, Resource)]
pub(crate) struct HudCommandQueue {
    pub(crate) pending: Vec<HudCommand>,
}

#[derive(Default, Resource)]
pub(crate) struct HudCommandState {
    pub(crate) last_robot_ref: Option<u32>,
    pub(crate) last_vehicle_ref: Option<u32>,
    pub(crate) last_cannon_ref: Option<u32>,
}

#[derive(Resource)]
pub(crate) struct HudAttackAlert {
    pub(crate) target_ref_id: Option<u32>,
    pub(crate) visible: bool,
    pub(crate) not_under_attack_checks: u8,
    pub(crate) check_elapsed: f32,
    pub(crate) flash_elapsed: f32,
}

impl Default for HudAttackAlert {
    fn default() -> Self {
        Self {
            target_ref_id: None,
            visible: false,
            not_under_attack_checks: 0,
            check_elapsed: 0.0,
            flash_elapsed: 0.0,
        }
    }
}

#[derive(Clone, Copy, Resource)]
pub(crate) struct FortUnderAttackWarning {
    pub(crate) danger_check_elapsed: f32,
    pub(crate) danger_fort_ref_id: Option<u32>,
    pub(crate) verbal_cooldown_remaining: f32,
    pub(crate) message: ComputerMessageState,
}

impl Default for FortUnderAttackWarning {
    fn default() -> Self {
        Self {
            danger_check_elapsed: 0.25,
            danger_fort_ref_id: None,
            verbal_cooldown_remaining: 0.0,
            message: ComputerMessageState::default(),
        }
    }
}

#[derive(Clone, Copy, Default, Resource)]
pub(crate) struct LosingVerbalWarning {
    pub(crate) cooldown_remaining: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct ComputerMessageState {
    pub(crate) target_ref_id: Option<u32>,
    pub(crate) visible: bool,
    pub(crate) blink_elapsed: f32,
    pub(crate) flips_remaining: u8,
    pub(crate) hold_remaining: f32,
}

impl Default for ComputerMessageState {
    fn default() -> Self {
        Self {
            target_ref_id: None,
            visible: false,
            blink_elapsed: 0.0,
            flips_remaining: 0,
            hold_remaining: 0.0,
        }
    }
}

impl ComputerMessageState {
    pub(crate) fn active(self) -> bool {
        self.target_ref_id.is_some()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CannonPlacementRequest {
    pub(crate) source_ref_id: u32,
    pub(crate) cannon: ObjectKind,
}

#[derive(Default, Resource)]
pub(crate) struct CannonPlacementState {
    pub(crate) pending: Option<CannonPlacementRequest>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductionWindowKind {
    Robot,
    Vehicle,
    Fort,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductionButtonKind {
    Place,
    Ok,
    Cancel,
    Up,
    Down,
    Plus,
    Minus,
    Queue,
    QueueUp,
    QueueDown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductionFullSelectorTarget {
    Main,
    Queue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProductionWindow {
    pub(crate) building_ref_id: u32,
    pub(crate) kind: ProductionWindowKind,
    pub(crate) selected_index: usize,
    pub(crate) queue_selected_index: usize,
    pub(crate) expanded: bool,
    pub(crate) full_selector: Option<ProductionFullSelectorTarget>,
    pub(crate) pressed_button: Option<ProductionButtonKind>,
}

#[derive(Default, Resource)]
pub(crate) struct ProductionWindowState {
    pub(crate) open: Option<ProductionWindow>,
    pub(crate) input_captured: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub(crate) struct ProductionDebugOpen {
    pub(crate) enabled: bool,
    pub(crate) ref_id: Option<u32>,
    pub(crate) expanded: bool,
    pub(crate) full_selector: Option<ProductionFullSelectorTarget>,
}

impl ProductionDebugOpen {
    pub(crate) fn from_env() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let enabled = std::env::var("ZOD_DEBUG_PRODUCTION").ok();
            let ref_id = std::env::var("ZOD_DEBUG_PRODUCTION_REF").ok();
            let expanded = std::env::var("ZOD_DEBUG_PRODUCTION_EXPANDED").ok();
            let full_selector = std::env::var("ZOD_DEBUG_PRODUCTION_FULL").ok();
            return Self::from_values(
                enabled.as_deref(),
                ref_id.as_deref(),
                expanded.as_deref(),
                full_selector.as_deref(),
            );
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self::default()
        }
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    fn from_values(
        enabled: Option<&str>,
        ref_id: Option<&str>,
        expanded: Option<&str>,
        full_selector: Option<&str>,
    ) -> Self {
        let ref_id = ref_id.and_then(|value| value.parse::<u32>().ok());
        let full_selector = full_selector.and_then(parse_debug_full_selector);

        Self {
            enabled: enabled.is_some_and(parse_truthy_env) || ref_id.is_some(),
            ref_id,
            expanded: expanded.is_some_and(parse_truthy_env),
            full_selector,
        }
    }
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn parse_truthy_env(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn parse_debug_full_selector(value: &str) -> Option<ProductionFullSelectorTarget> {
    match value.trim().to_ascii_lowercase().as_str() {
        "main" | "unit" | "units" => Some(ProductionFullSelectorTarget::Main),
        "queue" => Some(ProductionFullSelectorTarget::Queue),
        _ => None,
    }
}

#[derive(Component)]
pub(crate) struct ProductionWindowEntity;

#[derive(Component)]
pub(crate) struct ProductionWindowLabel;

#[derive(Component)]
pub(crate) struct ProductionWindowStateLabel;

#[derive(Component)]
pub(crate) struct ProductionWindowButton;

#[derive(Default, Resource)]
pub(crate) struct StartupScreenshot {
    pub(crate) path: Option<String>,
    pub(crate) frames_remaining: u32,
    pub(crate) requested: bool,
}

#[derive(Clone, Copy, Resource)]
pub(crate) struct HudLayout {
    pub(crate) map_pixel_size: Vec2,
    pub(crate) render_offset: Vec2,
    pub(crate) render_size: Vec2,
    pub(crate) render_ratio: f32,
}

#[derive(Clone, Copy, Component)]
pub(crate) enum HudAnchor {
    SidePanel,
    SideFiller,
    BottomLeft,
    BottomCenter,
    BottomRightCap,
    BaseTopLeft { top_left: Vec2, size: Vec2 },
    FixedXBaseY { top_left: Vec2, size: Vec2 },
    ScreenTopCenter { top_y: f32, size: Vec2 },
    BottomRight { offset: Vec2 },
}

#[derive(Clone, Copy, Component)]
pub(crate) struct HudHealthSegment {
    pub(crate) segment: HudHealthSegmentKind,
}

#[derive(Component)]
pub(crate) struct HudGrenadeIcon;

#[derive(Component)]
pub(crate) struct HudGrenadeText;

#[derive(Component)]
pub(crate) struct HudComputerMessage;

#[derive(Clone, Copy, Component)]
pub(crate) struct HudSelectedObjectSprite {
    pub(crate) slot: HudSelectedObjectSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HudSelectedObjectSlot {
    Icon,
    Label,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HudHealthSegmentKind {
    Full,
    Lost,
    Empty,
}

#[derive(Clone, Copy, Component)]
#[allow(dead_code)]
pub(crate) struct HudButton {
    pub(crate) kind: HudButtonKind,
    pub(crate) state: HudButtonState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub(crate) enum HudButtonKind {
    A = 0,
    B = 1,
    D = 2,
    G = 3,
    Menu = 4,
    R = 5,
    T = 6,
    V = 7,
    Z = 8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum HudButtonState {
    Active,
    Inactive,
    Pressed,
}

#[derive(Clone, Copy)]
pub(crate) struct HudButtonSpec {
    pub(crate) kind: HudButtonKind,
    pub(crate) asset_name: &'static str,
    pub(crate) top_left: Vec2,
    pub(crate) size: Vec2,
    pub(crate) initial_state: HudButtonState,
    pub(crate) fixed_x: bool,
}

#[derive(Component)]
pub(crate) struct MinimapDot {
    pub(crate) ref_id: u32,
}

#[derive(Component)]
pub(crate) struct MinimapZone {
    pub(crate) zone_index: usize,
}

#[derive(Component)]
pub(crate) struct MinimapViewBox;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_level_clamps_original_map_values() {
        assert_eq!(ProductionLevel::from_original(-3), ProductionLevel::Level0);
        assert_eq!(ProductionLevel::from_original(0), ProductionLevel::Level0);
        assert_eq!(ProductionLevel::from_original(3), ProductionLevel::Level3);
        assert_eq!(ProductionLevel::from_original(5), ProductionLevel::Level5);
        assert_eq!(ProductionLevel::from_original(9), ProductionLevel::Level5);
        assert_eq!(BuildingLevel::from_original(9).original(), 5);
    }

    #[test]
    fn debug_production_env_ref_enables_open_without_flag() {
        let debug = ProductionDebugOpen::from_values(None, Some("1"), None, None);

        assert!(debug.enabled);
        assert_eq!(debug.ref_id, Some(1));
        assert!(!debug.expanded);
        assert_eq!(debug.full_selector, None);
    }

    #[test]
    fn debug_production_env_parses_selector_and_expanded_flags() {
        let debug =
            ProductionDebugOpen::from_values(Some("yes"), Some("42"), Some("on"), Some("queue"));

        assert_eq!(
            debug,
            ProductionDebugOpen {
                enabled: true,
                ref_id: Some(42),
                expanded: true,
                full_selector: Some(ProductionFullSelectorTarget::Queue),
            }
        );
    }

    #[test]
    fn debug_production_env_ignores_unknown_selector() {
        let debug =
            ProductionDebugOpen::from_values(Some("true"), Some("bad"), Some("false"), Some("x"));

        assert!(debug.enabled);
        assert_eq!(debug.ref_id, None);
        assert!(!debug.expanded);
        assert_eq!(debug.full_selector, None);
    }
}
