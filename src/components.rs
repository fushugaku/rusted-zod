use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use crate::original::map::ZMap;
use crate::original::objects::{BuildingType, ObjectKind, RobotType, VehicleType};
use crate::original::tileinfo::PaletteTileInfo;
use crate::original::types::TeamType;
use crate::render::atlas::{
    FactoryOverlayKind, MobileSpriteRole, RadarOverlayKind, RepairOverlayKind, SpriteFrame,
};
use crate::units::{
    object_attack_damage, object_attack_radius, object_attack_speed, object_damage_chance,
    object_damage_radius, object_max_health, object_missile_speed, object_move_speed,
    object_snipe_chance, robots::RobotIdleActionKind,
};

#[derive(Resource)]
pub(crate) struct CurrentMap(pub(crate) ZMap);

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub(crate) struct CurrentMapSource {
    pub(crate) file_name: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) generation: u64,
}

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
    pub(crate) robot_manufactured_message: Handle<Image>,
    pub(crate) vehicle_manufactured_message: Handle<Image>,
    pub(crate) gun_manufactured_message: Handle<Image>,
    pub(crate) stored_gun_indicator: Handle<Image>,
    pub(crate) click_to_resume_message: Handle<Image>,
    pub(crate) vote_in_progress_panel: Handle<Image>,
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
    pub(crate) member_index: u16,
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
            attacked_only_by_explosives: crate::units::attacked_only_by_explosives(kind),
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
    crate::units::can_eject_drivers(kind, stats)
}

pub(crate) fn area_is_fort_turret_tile(map: &ZMap, tx: i32, ty: i32) -> bool {
    map.objects.iter().any(|object| {
        crate::units::buildings::map_object_has_fort_turret_tile(
            object.object_type,
            object.object_id,
            object.x as i32,
            object.y as i32,
            tx,
            ty,
        )
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MovementWaypointMode {
    Move,
    Attack,
    ForceMove,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MovementWaypoint {
    pub(crate) position: Vec2,
    pub(crate) mode: MovementWaypointMode,
    pub(crate) ref_id: Option<u32>,
    pub(crate) attack_to: bool,
    pub(crate) player_given: bool,
}

impl MovementWaypoint {
    pub(crate) fn move_to(position: Vec2) -> Self {
        Self {
            position,
            mode: MovementWaypointMode::Move,
            ref_id: None,
            attack_to: false,
            player_given: false,
        }
    }

    pub(crate) fn player_move_to(position: Vec2, attack_to: bool) -> Self {
        Self {
            position,
            mode: MovementWaypointMode::Move,
            ref_id: None,
            attack_to,
            player_given: true,
        }
    }

    pub(crate) fn force_move(position: Vec2) -> Self {
        Self {
            position,
            mode: MovementWaypointMode::ForceMove,
            ref_id: None,
            attack_to: false,
            player_given: false,
        }
    }

    pub(crate) fn attack_target(ref_id: u32, position: Vec2, player_given: bool) -> Self {
        Self::attack_target_with_flags(ref_id, position, false, player_given)
    }

    pub(crate) fn player_attack_target(ref_id: u32, position: Vec2) -> Self {
        Self::attack_target_with_flags(ref_id, position, true, true)
    }

    fn attack_target_with_flags(
        ref_id: u32,
        position: Vec2,
        attack_to: bool,
        player_given: bool,
    ) -> Self {
        Self {
            position,
            mode: MovementWaypointMode::Attack,
            ref_id: Some(ref_id),
            attack_to,
            player_given,
        }
    }

    pub(crate) fn rally_move(position: Vec2) -> Self {
        Self {
            position,
            mode: MovementWaypointMode::Move,
            ref_id: None,
            attack_to: true,
            player_given: true,
        }
    }

    pub(crate) fn with_position(self, position: Vec2) -> Self {
        Self { position, ..self }
    }

    pub(crate) fn stoppable(self) -> bool {
        self.mode == MovementWaypointMode::Move
    }
}

#[derive(Clone, Component)]
pub(crate) struct MovementPath {
    pub(crate) waypoints: Vec<Vec2>,
    pub(crate) typed_waypoints: Vec<MovementWaypoint>,
    pub(crate) speed: f32,
    pub(crate) attempt_run: bool,
}

impl MovementPath {
    pub(crate) fn new(waypoints: Vec<Vec2>, speed: f32) -> Self {
        let typed_waypoints = waypoints
            .iter()
            .copied()
            .map(MovementWaypoint::move_to)
            .collect();
        Self {
            waypoints,
            typed_waypoints,
            speed,
            attempt_run: false,
        }
    }

    pub(crate) fn from_typed(typed_waypoints: Vec<MovementWaypoint>, speed: f32) -> Self {
        Self {
            waypoints: typed_waypoints
                .iter()
                .map(|waypoint| waypoint.position)
                .collect(),
            typed_waypoints,
            speed,
            attempt_run: false,
        }
    }

    pub(crate) fn with_run_attempt(mut self) -> Self {
        self.attempt_run = true;
        self
    }

    pub(crate) fn insert_front_waypoint(&mut self, waypoint: MovementWaypoint) {
        self.waypoints.insert(0, waypoint.position);
        self.typed_waypoints.insert(0, waypoint);
    }

    pub(crate) fn pop_front_waypoint(&mut self) {
        if !self.waypoints.is_empty() {
            self.waypoints.remove(0);
        }
        if !self.typed_waypoints.is_empty() {
            self.typed_waypoints.remove(0);
        }
    }

    pub(crate) fn replace_front_waypoint_position(&mut self, position: Vec2) {
        if let Some(waypoint) = self.waypoints.first_mut() {
            *waypoint = position;
        }
        if let Some(waypoint) = self.typed_waypoints.first_mut() {
            waypoint.position = position;
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.waypoints.is_empty() && self.typed_waypoints.is_empty()
    }
}

#[derive(Clone, Copy, Component, Debug, Default, PartialEq)]
pub(crate) struct MovementVelocity(pub(crate) Vec2);

impl MovementVelocity {
    const SOURCE_MOVING_EPSILON: f32 = 0.00001;

    pub(crate) fn is_moving(self) -> bool {
        let epsilon = Self::SOURCE_MOVING_EPSILON;
        !((self.0.x > -epsilon && self.0.x < epsilon)
            && (self.0.y > -epsilon && self.0.y < epsilon))
    }
}

#[derive(Clone, Copy, Component, Debug, Default, PartialEq)]
pub(crate) struct SourceObjectLocation {
    pub(crate) map_position: Vec2,
    pub(crate) map_velocity: Vec2,
    pub(crate) map_remainder: Vec2,
    pub(crate) world_anchor: Vec2,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct SourceLocationInterpolation {
    pub(crate) last_map_position: Vec2,
    pub(crate) layer_map_offset: Vec2,
    pub(crate) map_velocity: Vec2,
    pub(crate) elapsed: f32,
    pub(crate) just_set: bool,
}

#[derive(Component)]
pub(crate) struct AcceptedEmptyWaypointCommand;

#[derive(Default, Component)]
pub(crate) struct BuildingRallyPoints {
    pub(crate) points: Vec<Vec2>,
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

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct VehicleLidState {
    pub(crate) open: bool,
    pub(crate) closing: bool,
    pub(crate) close_delay: f32,
    pub(crate) frame: usize,
    pub(crate) elapsed: f32,
    pub(crate) show_driver: bool,
    pub(crate) attack_target_ref: Option<u32>,
}

impl VehicleLidState {
    pub(crate) fn closed() -> Self {
        Self {
            open: false,
            closing: false,
            close_delay: 0.0,
            frame: 0,
            elapsed: 0.0,
            show_driver: false,
            attack_target_ref: None,
        }
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(crate) enum VehicleLidVisualRole {
    Lid,
    Driver,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(crate) struct VehicleLidVisualLayer {
    pub(crate) vehicle: VehicleType,
    pub(crate) role: VehicleLidVisualRole,
}

#[derive(Component)]
pub(crate) struct VehicleLidVisualFrames {
    pub(crate) frames: Vec<Handle<Image>>,
    pub(crate) frames_per_direction: usize,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct AttackTarget {
    pub(crate) ref_id: u32,
    pub(crate) cooldown: f32,
    pub(crate) player_given: bool,
}

pub(crate) type AttackTargetLifecycleComponents = (
    AttackTarget,
    RobotFireAnimation,
    RobotFireVisualCue,
    RobotGrenadeReadyAttackPose,
    RobotGrenadePickupAnimation,
    RobotIdleActionAnimation,
    RobotIdleProcessTimer,
);

pub(crate) fn attack_target_for_assignment(
    target_ref_id: u32,
    player_given: bool,
    previous: Option<&AttackTarget>,
) -> AttackTarget {
    AttackTarget {
        ref_id: target_ref_id,
        cooldown: previous.map_or(0.0, |target| target.cooldown.max(0.0)),
        player_given,
    }
}

pub(crate) fn set_attack_target_components(
    commands: &mut Commands,
    entity: Entity,
    target_ref_id: u32,
    player_given: bool,
    previous: Option<&AttackTarget>,
) {
    commands
        .entity(entity)
        .insert(attack_target_for_assignment(
            target_ref_id,
            player_given,
            previous,
        ))
        .remove::<(
            RobotFireAnimation,
            RobotFireVisualCue,
            RobotGrenadeReadyAttackPose,
            RobotGrenadePickupAnimation,
            RobotIdleActionAnimation,
            RobotIdleProcessTimer,
        )>();
}

pub(crate) fn clear_attack_target_components(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<AttackTargetLifecycleComponents>();
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CraneRepairStage {
    GotoEntrance,
    EnterBuilding,
    ExitBuilding,
}

#[derive(Clone, Component)]
pub(crate) struct UnitRepairTarget {
    pub(crate) ref_id: u32,
    pub(crate) stage: UnitRepairStage,
    pub(crate) center_point: Vec2,
    pub(crate) entrance_point: Vec2,
    pub(crate) resume_waypoints: Vec<MovementWaypoint>,
}

#[derive(Clone, Component)]
pub(crate) struct RepairResumeWaypoints(pub(crate) Vec<MovementWaypoint>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnitRepairStage {
    GotoEntrance,
    Wait,
    EnterBuilding,
    ExitBuilding,
}

#[derive(Clone, Component)]
pub(crate) struct RepairBuildingOccupancy {
    pub(crate) unit: ObjectKind,
    pub(crate) driver: Option<DriverHealth>,
    pub(crate) center_point: Vec2,
    pub(crate) entrance_point: Vec2,
    pub(crate) resume_waypoints: Vec<MovementWaypoint>,
    pub(crate) remaining: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub(crate) struct RepairBuildingAnimState {
    pub(crate) remaining_time: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum BuildingProductionStatus {
    Place,
    Select,
    Building,
    Paused,
}

#[derive(Clone, Component)]
pub(crate) struct BuildingProduction {
    pub(crate) status: BuildingProductionStatus,
    pub(crate) current: Option<ObjectKind>,
    pub(crate) queue: VecDeque<ObjectKind>,
    pub(crate) elapsed: f32,
    pub(crate) duration: f32,
    pub(crate) zone_ownage: f32,
    pub(crate) unit_limit_reached: bool,
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
    pub(crate) attacker_ref_id: Option<u32>,
    pub(crate) attack_player_given: bool,
    pub(crate) target_ref_id: Option<u32>,
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
    pub(crate) killer: f32,
    pub(crate) killer_ref_id: Option<u32>,
}

#[derive(Resource)]
pub(crate) struct CombatRng(pub(crate) u32);

#[derive(Resource)]
pub(crate) struct PassiveEngageTimer(pub(crate) Timer);

#[derive(Resource)]
pub(crate) struct FlagCaptureTimer(pub(crate) Timer);

#[derive(Resource)]
pub(crate) struct NextObjectRefId(pub(crate) u32);

#[derive(Default, Resource)]
pub(crate) struct DynamicObjectRefReservations {
    pub(crate) first_ref_by_building: HashMap<u32, u32>,
}

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

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct RobotGrenadeThrowAnimation {
    pub(crate) frame: usize,
    pub(crate) elapsed: f32,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct RobotGrenadePickupAnimation {
    pub(crate) upward: bool,
    pub(crate) frame: usize,
    pub(crate) elapsed: f32,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct RobotIdleProcessTimer {
    pub(crate) elapsed: f32,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct RobotIdleActionAnimation {
    pub(crate) kind: RobotIdleActionKind,
    pub(crate) frame: usize,
    pub(crate) elapsed: f32,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct RobotFireAnimation {
    pub(crate) frame: usize,
    pub(crate) elapsed: f32,
    pub(crate) delay: f32,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct RobotFireVisualCue {
    pub(crate) target_ref_id: u32,
    pub(crate) target: Vec2,
    pub(crate) team: TeamType,
    pub(crate) effective_kind: ObjectKind,
    pub(crate) sound_top_left_map: Vec2,
    pub(crate) sound_size: Vec2,
}

pub(crate) fn robot_fire_visual_cue_matches_attack_target(
    cue: RobotFireVisualCue,
    attack_target: Option<&AttackTarget>,
) -> bool {
    attack_target.is_some_and(|target| target.ref_id == cue.target_ref_id)
}

#[derive(Clone, Copy, Component, Debug, Default, Eq, PartialEq)]
pub(crate) struct RobotGrenadeReadyAttackPose;

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
    ResumeGame,
    FocusObject {
        ref_id: u32,
        select_obj: bool,
        open_gui: bool,
    },
}

#[derive(Default, Resource)]
pub(crate) struct HudCommandQueue {
    pub(crate) pending: Vec<HudCommand>,
}

#[derive(Default, Resource)]
pub(crate) struct StoredGunHudClickState {
    pub(crate) pressed_ref_id: Option<u32>,
}

#[derive(Default, Resource)]
pub(crate) struct ResumePromptClickState {
    pub(crate) pressed: bool,
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
    pub(crate) anim_elapsed: f32,
    pub(crate) anim_delay: f32,
    pub(crate) last_anim: Option<u8>,
}

impl Default for HudAttackAlert {
    fn default() -> Self {
        Self {
            target_ref_id: None,
            visible: false,
            not_under_attack_checks: 0,
            check_elapsed: 0.0,
            flash_elapsed: 0.0,
            anim_elapsed: 0.0,
            anim_delay: 0.0,
            last_anim: None,
        }
    }
}

impl HudAttackAlert {
    pub(crate) fn source_set_ref_id(&mut self, ref_id: u32, next_anim_delay: f32) {
        self.target_ref_id = Some(ref_id);
        self.visible = true;
        self.not_under_attack_checks = 0;
        self.check_elapsed = 0.0;
        self.flash_elapsed = 0.0;
        self.source_schedule_next_anim(next_anim_delay);
    }

    pub(crate) fn source_clear(&mut self) {
        self.target_ref_id = None;
        self.visible = false;
        self.not_under_attack_checks = 0;
        self.check_elapsed = 0.0;
        self.flash_elapsed = 0.0;
    }

    pub(crate) fn source_schedule_next_anim(&mut self, delay: f32) {
        self.anim_elapsed = 0.0;
        self.anim_delay = delay.max(0.0);
    }
}

#[derive(Default, Resource)]
pub(crate) struct AttackAlertPacketQueue {
    pub(crate) pending_target_ref_ids: Vec<u32>,
}

#[derive(Clone, Copy, Resource)]
pub(crate) struct FortUnderAttackWarning {
    pub(crate) danger_check_elapsed: f32,
    pub(crate) danger_fort_ref_id: Option<u32>,
    pub(crate) verbal_cooldown_remaining: f32,
}

impl Default for FortUnderAttackWarning {
    fn default() -> Self {
        Self {
            danger_check_elapsed: 0.25,
            danger_fort_ref_id: None,
            verbal_cooldown_remaining: 0.0,
        }
    }
}

#[derive(Clone, Copy, Default, Resource)]
pub(crate) struct LosingVerbalWarning {
    pub(crate) cooldown_remaining: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComputerMessageKind {
    RobotManufactured,
    VehicleManufactured,
    GunManufactured,
    FortUnderAttack,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PortraitAnimationKind {
    SelectedCommon(u8),
    SelectedRobotReporting(RobotType),
    Acknowledge(u8),
    AcknowledgeNoWay(u8),
    WereUnderAttack,
    UnderAttackRepeat(u8),
    TargetDestroyed,
    GoodHit(u8),
    TerritoryTaken,
    GunCaptured,
    VehicleCaptured,
    GrenadesCollected,
    Idle(u8),
    EndWin { animation: u8, sound: u8 },
    EndLose { animation: u8, sound: u8 },
}

impl PortraitAnimationKind {
    pub(crate) const fn source_total_duration_secs(self) -> f32 {
        match self {
            Self::SelectedCommon(0) => 0.660,
            Self::SelectedCommon(1) => 0.720,
            Self::SelectedCommon(2) => 0.975,
            Self::SelectedCommon(_) => 0.825,
            Self::SelectedRobotReporting(RobotType::Grunt) => 0.870,
            Self::SelectedRobotReporting(RobotType::Psycho) => 0.945,
            Self::SelectedRobotReporting(RobotType::Sniper) => 0.945,
            Self::SelectedRobotReporting(RobotType::Tough) => 0.900,
            Self::SelectedRobotReporting(RobotType::Pyro) => 0.900,
            Self::SelectedRobotReporting(RobotType::Laser) => 0.900,
            Self::Acknowledge(0) => 0.765,
            Self::Acknowledge(1) => 0.705,
            Self::Acknowledge(2) => 0.465,
            Self::Acknowledge(3) => 0.660,
            Self::Acknowledge(4) => 0.510,
            Self::Acknowledge(5) => 0.465,
            Self::Acknowledge(6) => 0.630,
            Self::Acknowledge(7) => 0.570,
            Self::Acknowledge(8) => 0.630,
            Self::Acknowledge(9) => 0.675,
            Self::Acknowledge(10) => 0.900,
            Self::Acknowledge(_) => 0.525,
            Self::AcknowledgeNoWay(0) => 0.720,
            Self::AcknowledgeNoWay(1) => 0.885,
            Self::AcknowledgeNoWay(_) => 0.720,
            Self::WereUnderAttack => 1.005,
            Self::UnderAttackRepeat(0) => 1.380,
            Self::UnderAttackRepeat(1) => 1.050,
            Self::UnderAttackRepeat(2) => 1.035,
            Self::UnderAttackRepeat(3) => 1.005,
            Self::UnderAttackRepeat(4) => 1.095,
            Self::UnderAttackRepeat(_) => 1.965,
            Self::TargetDestroyed => 0.975,
            Self::GoodHit(0) => 0.780,
            Self::GoodHit(1) => 0.780,
            Self::GoodHit(2) => 0.795,
            Self::GoodHit(3) => 0.690,
            Self::GoodHit(4) => 1.140,
            Self::GoodHit(5) => 0.660,
            Self::GoodHit(_) => 1.020,
            Self::TerritoryTaken => 0.900,
            Self::GunCaptured => 0.855,
            Self::VehicleCaptured => 0.975,
            Self::GrenadesCollected => 0.945,
            Self::Idle(0) => 0.900,
            Self::Idle(1) => 1.200,
            Self::Idle(2..=5) => 1.350,
            Self::Idle(6..=9) => 1.650,
            Self::Idle(10) => 2.700,
            Self::Idle(_) => 1.500,
            Self::EndWin { animation: 0, .. } | Self::EndLose { animation: 0, .. } => 0.525,
            Self::EndWin { animation: 1, .. } | Self::EndLose { animation: 1, .. } => 0.855,
            Self::EndWin { .. } | Self::EndLose { .. } => 0.675,
        }
    }

    pub(crate) const fn wire_id(self) -> i32 {
        match self {
            Self::SelectedCommon(index) => (if index > 3 { 3 } else { index }) as i32,
            Self::SelectedRobotReporting(RobotType::Grunt) => 4,
            Self::SelectedRobotReporting(RobotType::Psycho) => 5,
            Self::SelectedRobotReporting(RobotType::Sniper) => 6,
            Self::SelectedRobotReporting(RobotType::Tough) => 7,
            Self::SelectedRobotReporting(RobotType::Laser) => 8,
            Self::SelectedRobotReporting(RobotType::Pyro) => 9,
            Self::Acknowledge(index) => 10 + ((if index > 11 { 11 } else { index }) as i32),
            Self::AcknowledgeNoWay(0) => 48,
            Self::AcknowledgeNoWay(1) => 49,
            Self::AcknowledgeNoWay(_) => 50,
            Self::WereUnderAttack => 22,
            Self::UnderAttackRepeat(index) => 23 + ((if index > 5 { 5 } else { index }) as i32),
            Self::TargetDestroyed => 30,
            Self::GoodHit(index) => 51 + ((if index > 6 { 6 } else { index }) as i32),
            Self::TerritoryTaken => 58,
            Self::GunCaptured => 60,
            Self::VehicleCaptured => 61,
            Self::GrenadesCollected => 62,
            Self::Idle(index) => 31 + (if index > 12 { 12 } else { index }) as i32,
            Self::EndWin { animation, .. } => {
                63 + (if animation > 2 { 2 } else { animation }) as i32
            }
            Self::EndLose { animation, .. } => {
                66 + (if animation > 2 { 2 } else { animation }) as i32
            }
        }
    }

    pub(crate) const fn from_wire_id(anim_id: i32) -> Option<Self> {
        match anim_id {
            22 => Some(Self::WereUnderAttack),
            23..=28 => Some(Self::UnderAttackRepeat((anim_id - 23) as u8)),
            30 => Some(Self::TargetDestroyed),
            51..=57 => Some(Self::GoodHit((anim_id - 51) as u8)),
            58 => Some(Self::TerritoryTaken),
            60 => Some(Self::GunCaptured),
            61 => Some(Self::VehicleCaptured),
            62 => Some(Self::GrenadesCollected),
            63..=65 => Some(Self::EndWin {
                animation: (anim_id - 63) as u8,
                sound: 0,
            }),
            66..=68 => Some(Self::EndLose {
                animation: (anim_id - 66) as u8,
                sound: 0,
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PortraitAnimationEvent {
    pub(crate) ref_id: u32,
    pub(crate) kind: PortraitAnimationKind,
}

#[derive(Default, Resource)]
pub(crate) struct PortraitAnimationState {
    active: Option<PortraitAnimationEvent>,
    elapsed: f32,
}

#[derive(Default, Resource)]
pub(crate) struct PortraitAnimationSoundQueue {
    pub(crate) pending: Vec<PortraitAnimationKind>,
}

impl PortraitAnimationState {
    pub(crate) fn active_event(&self) -> Option<PortraitAnimationEvent> {
        self.active
    }

    pub(crate) fn doing_anim(&self) -> bool {
        self.active_ref_id().zip(self.active_kind()).is_some()
    }

    pub(crate) fn start(&mut self, event: PortraitAnimationEvent) {
        self.active = Some(event);
        self.elapsed = 0.0;
    }

    pub(crate) fn clear(&mut self) {
        self.active = None;
        self.elapsed = 0.0;
    }

    pub(crate) fn process(&mut self, delta_secs: f32) -> bool {
        let Some(event) = self.active else {
            return false;
        };
        self.elapsed += delta_secs.max(0.0);
        if self.elapsed <= event.kind.source_total_duration_secs() {
            return false;
        }

        self.active = None;
        self.elapsed = 0.0;
        true
    }

    fn active_ref_id(&self) -> Option<u32> {
        self.active.map(|event| event.ref_id)
    }

    pub(crate) fn active_kind(&self) -> Option<PortraitAnimationKind> {
        self.active.map(|event| event.kind)
    }

    pub(crate) fn elapsed_secs(&self) -> f32 {
        self.elapsed
    }
}

#[derive(Default, Resource)]
pub(crate) struct SelectedPortraitAnimationState(PortraitAnimationState);

impl SelectedPortraitAnimationState {
    pub(crate) fn active_event(&self) -> Option<PortraitAnimationEvent> {
        self.0.active_event()
    }

    pub(crate) fn start(&mut self, event: PortraitAnimationEvent) {
        self.0.start(event);
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    pub(crate) fn process(&mut self, delta_secs: f32) -> bool {
        self.0.process(delta_secs)
    }

    pub(crate) fn elapsed_secs(&self) -> f32 {
        self.0.elapsed_secs()
    }
}

#[derive(Clone, Copy, Default, Resource)]
pub(crate) struct ComputerMessageDisplay {
    pub(crate) message: ComputerMessageState,
}

#[derive(Clone, Copy, Resource)]
pub(crate) struct GamePauseState {
    pub(crate) paused: bool,
}

impl Default for GamePauseState {
    fn default() -> Self {
        Self { paused: true }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GamePauseRequest {
    pub(crate) game_paused: bool,
}

#[derive(Default, Resource)]
pub(crate) struct GamePauseRequestQueue {
    pub(crate) pending: Vec<GamePauseRequest>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GamePauseUpdate {
    pub(crate) game_paused: bool,
}

#[derive(Default, Resource)]
pub(crate) struct GamePauseUpdateQueue {
    pub(crate) pending: Vec<GamePauseUpdate>,
}

#[derive(Default, Resource)]
pub(crate) struct GamePauseInitialQueryState {
    pub(crate) requested: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ComputerMessageState {
    pub(crate) kind: Option<ComputerMessageKind>,
    pub(crate) target_ref_id: Option<u32>,
    pub(crate) visible: bool,
    pub(crate) blink_elapsed: f32,
    pub(crate) flips_remaining: u8,
    pub(crate) hold_remaining: f32,
}

impl Default for ComputerMessageState {
    fn default() -> Self {
        Self {
            kind: None,
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
        self.kind.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SpaceBarEvent {
    pub(crate) ref_id: u32,
    pub(crate) select_obj: bool,
    pub(crate) open_gui: bool,
    pub(crate) lifetime_remaining: f32,
}

impl SpaceBarEvent {
    pub(crate) const LIFETIME_SECS: f32 = 10.0;

    pub(crate) fn new(ref_id: u32, select_obj: bool, open_gui: bool) -> Self {
        Self {
            ref_id,
            select_obj,
            open_gui,
            lifetime_remaining: Self::LIFETIME_SECS,
        }
    }

    pub(crate) fn expired(self) -> bool {
        self.lifetime_remaining <= 0.0
    }
}

#[derive(Default, Resource)]
pub(crate) struct SpaceBarEventQueue {
    pub(crate) events: VecDeque<SpaceBarEvent>,
}

impl SpaceBarEventQueue {
    pub(crate) const MAX_EVENTS: usize = 5;

    pub(crate) fn add(&mut self, event: SpaceBarEvent) {
        self.events
            .retain(|existing| existing.ref_id != event.ref_id);
        self.events.push_front(event);
        self.events.truncate(Self::MAX_EVENTS);
    }

    pub(crate) fn advance(&mut self, delta_secs: f32) {
        let delta_secs = delta_secs.max(0.0);
        for event in &mut self.events {
            event.lifetime_remaining -= delta_secs;
        }
        self.events.retain(|event| !event.expired());
    }

    pub(crate) fn source_clear(&mut self) {
        self.events.clear();
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
    BasePoint { point: Vec2 },
    FixedXBaseY { top_left: Vec2, size: Vec2 },
    ScreenTopLeft { top_left: Vec2, size: Vec2 },
    ScreenTopRight { top_right: Vec2, size: Vec2 },
    ScreenTopCenter { top_y: f32, size: Vec2 },
    ScreenCenter { size: Vec2 },
    ScreenBottomLeft { bottom_left: Vec2, size: Vec2 },
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

#[derive(Component)]
pub(crate) struct HudResumePrompt;

#[derive(Component)]
pub(crate) struct HudVotePanel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HudVoteTextField {
    Description,
    Have,
    Needed,
    ForVotes,
    AgainstVotes,
}

#[derive(Component)]
pub(crate) struct HudVoteText {
    pub(crate) field: HudVoteTextField,
}

#[derive(Component)]
pub(crate) struct HudNewsText {
    pub(crate) slot: usize,
}

#[derive(Component)]
pub(crate) struct HudChatText;

#[derive(Clone, Copy, Component)]
pub(crate) struct HudStoredGunIcon {
    pub(crate) slot: usize,
}

#[derive(Clone, Copy, Component)]
pub(crate) struct HudStoredGunMultiplier {
    pub(crate) slot: usize,
}

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
    fn movement_velocity_uses_source_is_moving_epsilon() {
        assert!(!MovementVelocity::default().is_moving());
        assert!(!MovementVelocity(Vec2::new(0.000009, -0.000009)).is_moving());
        assert!(MovementVelocity(Vec2::new(0.00001, 0.0)).is_moving());
        assert!(MovementVelocity(Vec2::new(0.0, -0.00001)).is_moving());
    }

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

    #[test]
    fn space_bar_event_queue_matches_original_dedupe_limit_and_lifetime() {
        let mut queue = SpaceBarEventQueue::default();

        for ref_id in 1..=6 {
            queue.add(SpaceBarEvent::new(ref_id, ref_id % 2 == 0, false));
        }
        assert_eq!(queue.events.len(), SpaceBarEventQueue::MAX_EVENTS);
        assert_eq!(
            queue
                .events
                .iter()
                .map(|event| event.ref_id)
                .collect::<Vec<_>>(),
            vec![6, 5, 4, 3, 2]
        );

        queue.add(SpaceBarEvent::new(4, false, true));
        assert_eq!(
            queue
                .events
                .iter()
                .map(|event| (event.ref_id, event.select_obj, event.open_gui))
                .collect::<Vec<_>>(),
            vec![
                (4, false, true),
                (6, true, false),
                (5, false, false),
                (3, false, false),
                (2, true, false)
            ]
        );

        queue.advance(SpaceBarEvent::LIFETIME_SECS + 0.1);
        assert!(queue.events.is_empty());
    }

    #[test]
    fn portrait_animation_wire_ids_match_source_under_attack() {
        assert_eq!(PortraitAnimationKind::SelectedCommon(0).wire_id(), 0);
        assert_eq!(
            PortraitAnimationKind::SelectedRobotReporting(RobotType::Pyro).wire_id(),
            9
        );
        assert_eq!(PortraitAnimationKind::Acknowledge(0).wire_id(), 10);
        assert_eq!(PortraitAnimationKind::Acknowledge(11).wire_id(), 21);
        assert_eq!(PortraitAnimationKind::AcknowledgeNoWay(0).wire_id(), 48);
        assert_eq!(PortraitAnimationKind::AcknowledgeNoWay(2).wire_id(), 50);
        assert_eq!(PortraitAnimationKind::Idle(0).wire_id(), 31);
        assert_eq!(PortraitAnimationKind::Idle(12).wire_id(), 43);
        assert_eq!(PortraitAnimationKind::Idle(99).wire_id(), 43);
        assert_eq!(PortraitAnimationKind::from_wire_id(0), None);
        assert_eq!(PortraitAnimationKind::from_wire_id(10), None);
        assert_eq!(PortraitAnimationKind::from_wire_id(47), None);
        assert_eq!(PortraitAnimationKind::WereUnderAttack.wire_id(), 22);
        assert_eq!(
            PortraitAnimationKind::from_wire_id(22),
            Some(PortraitAnimationKind::WereUnderAttack)
        );
        assert_eq!(PortraitAnimationKind::UnderAttackRepeat(0).wire_id(), 23);
        assert_eq!(PortraitAnimationKind::UnderAttackRepeat(5).wire_id(), 28);
        assert_eq!(PortraitAnimationKind::UnderAttackRepeat(9).wire_id(), 28);
        assert_eq!(
            PortraitAnimationKind::from_wire_id(28),
            Some(PortraitAnimationKind::UnderAttackRepeat(5))
        );
        assert_eq!(
            PortraitAnimationKind::EndWin {
                animation: 2,
                sound: 5,
            }
            .wire_id(),
            65
        );
        assert_eq!(
            PortraitAnimationKind::from_wire_id(68),
            Some(PortraitAnimationKind::EndLose {
                animation: 2,
                sound: 0,
            })
        );
    }

    #[test]
    fn portrait_animation_durations_match_source_totals_for_wired_anims() {
        assert_eq!(
            PortraitAnimationKind::SelectedCommon(0).source_total_duration_secs(),
            0.660
        );
        assert_eq!(
            PortraitAnimationKind::SelectedCommon(2).source_total_duration_secs(),
            0.975
        );
        assert_eq!(
            PortraitAnimationKind::SelectedRobotReporting(RobotType::Grunt)
                .source_total_duration_secs(),
            0.870
        );
        assert_eq!(
            PortraitAnimationKind::SelectedRobotReporting(RobotType::Psycho)
                .source_total_duration_secs(),
            0.945
        );
        assert_eq!(
            PortraitAnimationKind::SelectedRobotReporting(RobotType::Laser)
                .source_total_duration_secs(),
            0.900
        );
        assert_eq!(
            PortraitAnimationKind::Acknowledge(0).source_total_duration_secs(),
            0.765
        );
        assert_eq!(
            PortraitAnimationKind::Acknowledge(10).source_total_duration_secs(),
            0.900
        );
        assert_eq!(
            PortraitAnimationKind::AcknowledgeNoWay(0).source_total_duration_secs(),
            0.720
        );
        assert_eq!(
            PortraitAnimationKind::AcknowledgeNoWay(1).source_total_duration_secs(),
            0.885
        );
        assert_eq!(
            PortraitAnimationKind::WereUnderAttack.source_total_duration_secs(),
            1.005
        );
        assert_eq!(
            PortraitAnimationKind::UnderAttackRepeat(0).source_total_duration_secs(),
            1.380
        );
        assert_eq!(
            PortraitAnimationKind::UnderAttackRepeat(5).source_total_duration_secs(),
            1.965
        );
        assert_eq!(
            PortraitAnimationKind::TargetDestroyed.source_total_duration_secs(),
            0.975
        );
        assert_eq!(
            PortraitAnimationKind::GoodHit(0).source_total_duration_secs(),
            0.780
        );
        assert_eq!(
            PortraitAnimationKind::GoodHit(6).source_total_duration_secs(),
            1.020
        );
        assert_eq!(
            PortraitAnimationKind::TerritoryTaken.source_total_duration_secs(),
            0.900
        );
        assert_eq!(
            PortraitAnimationKind::GunCaptured.source_total_duration_secs(),
            0.855
        );
        assert_eq!(
            PortraitAnimationKind::VehicleCaptured.source_total_duration_secs(),
            0.975
        );
        assert_eq!(
            PortraitAnimationKind::GrenadesCollected.source_total_duration_secs(),
            0.945
        );
        assert_eq!(
            PortraitAnimationKind::Idle(10).source_total_duration_secs(),
            2.700
        );
        assert_eq!(
            PortraitAnimationKind::Idle(12).source_total_duration_secs(),
            1.500
        );
        assert_eq!(
            PortraitAnimationKind::EndWin {
                animation: 0,
                sound: 5,
            }
            .source_total_duration_secs(),
            0.525
        );
        assert_eq!(
            PortraitAnimationKind::EndLose {
                animation: 1,
                sound: 6,
            }
            .source_total_duration_secs(),
            0.855
        );
    }

    #[test]
    fn portrait_animation_state_clears_after_source_total_duration() {
        let mut state = PortraitAnimationState::default();
        state.start(PortraitAnimationEvent {
            ref_id: 7,
            kind: PortraitAnimationKind::TargetDestroyed,
        });

        assert!(state.doing_anim());
        assert!(!state.process(0.975));
        assert!(state.doing_anim());
        assert!(state.process(0.001));
        assert!(!state.doing_anim());
    }

    #[test]
    fn selected_and_a_portrait_states_advance_independently() {
        let selected_event = PortraitAnimationEvent {
            ref_id: 4,
            kind: PortraitAnimationKind::Idle(10),
        };
        let overlay_event = PortraitAnimationEvent {
            ref_id: 9,
            kind: PortraitAnimationKind::TargetDestroyed,
        };
        let mut selected = SelectedPortraitAnimationState::default();
        let mut overlay = PortraitAnimationState::default();
        selected.start(selected_event);
        overlay.start(overlay_event);

        assert!(overlay.process(0.976));
        assert_eq!(overlay.active_event(), None);
        assert_eq!(selected.active_event(), Some(selected_event));
        assert!(!selected.process(0.976));
        assert_eq!(selected.active_event(), Some(selected_event));
    }

    #[test]
    fn portrait_animation_start_resets_elapsed() {
        let mut state = PortraitAnimationState::default();
        state.start(PortraitAnimationEvent {
            ref_id: 7,
            kind: PortraitAnimationKind::GoodHit(3),
        });
        assert!(!state.process(0.5));

        state.start(PortraitAnimationEvent {
            ref_id: 8,
            kind: PortraitAnimationKind::GrenadesCollected,
        });
        assert!(!state.process(0.5));
        assert!(state.doing_anim());
    }

    #[test]
    fn attack_assignment_preserves_damage_cooldown() {
        let previous = AttackTarget {
            ref_id: 4,
            cooldown: 0.75,
            player_given: false,
        };

        assert_eq!(
            attack_target_for_assignment(9, true, Some(&previous)),
            AttackTarget {
                ref_id: 9,
                cooldown: 0.75,
                player_given: true,
            }
        );
        assert_eq!(
            attack_target_for_assignment(9, true, None),
            AttackTarget {
                ref_id: 9,
                cooldown: 0.0,
                player_given: true,
            }
        );
    }

    #[test]
    fn robot_fire_visual_cue_is_bound_to_current_target() {
        let cue = RobotFireVisualCue {
            target_ref_id: 7,
            target: Vec2::ZERO,
            team: TeamType::Red,
            effective_kind: ObjectKind::Robot(RobotType::Grunt),
            sound_top_left_map: Vec2::ZERO,
            sound_size: Vec2::ONE,
        };
        let current = AttackTarget {
            ref_id: 7,
            cooldown: 0.0,
            player_given: false,
        };
        let stale = AttackTarget {
            ref_id: 8,
            cooldown: 0.0,
            player_given: false,
        };

        assert!(robot_fire_visual_cue_matches_attack_target(
            cue,
            Some(&current)
        ));
        assert!(!robot_fire_visual_cue_matches_attack_target(
            cue,
            Some(&stale)
        ));
        assert!(!robot_fire_visual_cue_matches_attack_target(cue, None));
    }
}
