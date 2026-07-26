use bevy::{camera::visibility::RenderLayers, prelude::*};

use crate::{
    components::{
        CombatRng, CurrentMap, DriverHealth, GameObjectEntity, HudAnchor, ObjectTeam,
        PortraitAnimationEvent, PortraitAnimationKind, PortraitAnimationSoundQueue,
        PortraitAnimationState, SelectedPortraitAnimationState, SelectionState,
    },
    constants::HUD_LAYER,
    local_player::LocalPlayerState,
    original::{
        objects::{ObjectKind, RobotType},
        types::{PlanetType, TeamType},
    },
    units,
};

const PORTRAIT_TOP_LEFT: Vec2 = Vec2::new(556.0, 44.0);
const PORTRAIT_SIZE: Vec2 = Vec2::new(86.0, 74.0);
const END_UNIT_INTERVAL_SECONDS: f32 = 1.2;
const PORTRAIT_FRAME_TIME_UNIT: f32 = 0.015;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HudEndUnit {
    pub(crate) ref_id: u32,
    pub(crate) robot: RobotType,
    pub(crate) in_vehicle: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamEndedClientOutcome {
    pub(crate) team: TeamType,
    pub(crate) won: bool,
    pub(crate) units: Vec<HudEndUnit>,
}

#[derive(Default, Resource)]
pub(crate) struct TeamEndedClientQueue {
    pub(crate) pending: Vec<TeamEndedClientOutcome>,
}

#[derive(Default, Resource)]
pub(crate) struct EndAnimationDebug {
    requested_win: Option<bool>,
    end_emitted: bool,
    requested_portrait: bool,
    portrait_emitted: bool,
}

impl EndAnimationDebug {
    pub(crate) fn from_env() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(value) = std::env::var("ZOD_DEBUG_END_ANIMATION") {
            let requested_win = match value.trim().to_ascii_lowercase().as_str() {
                "win" | "won" | "1" | "true" => Some(true),
                "lose" | "lost" | "0" | "false" => Some(false),
                _ => None,
            };
            return Self {
                requested_win,
                end_emitted: false,
                requested_portrait: std::env::var_os("ZOD_DEBUG_PORTRAIT").is_some(),
                portrait_emitted: false,
            };
        }
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            requested_portrait: std::env::var_os("ZOD_DEBUG_PORTRAIT").is_some(),
            ..Self::default()
        }
    }
}

#[derive(Default, Resource)]
pub(crate) struct HudEndAnimationState {
    active: bool,
    won: bool,
    remaining: Vec<HudEndUnit>,
    current_unit: Option<HudEndUnit>,
    next_unit_delay: f32,
    last_sound_variant: Option<u8>,
}

#[derive(Resource)]
pub(crate) struct PortraitIdleState {
    subject_ref_id: Option<u32>,
    delay_remaining: f32,
}

impl Default for PortraitIdleState {
    fn default() -> Self {
        Self {
            subject_ref_id: None,
            delay_remaining: 0.5,
        }
    }
}

impl HudEndAnimationState {
    fn start(&mut self, units: Vec<HudEndUnit>, won: bool) {
        self.active = true;
        self.won = won;
        self.remaining = units;
        self.current_unit = None;
        self.next_unit_delay = 0.0;
    }

    pub(crate) fn clear_for_reset(&mut self) {
        self.active = false;
        self.won = false;
        self.remaining.clear();
        self.current_unit = None;
        self.next_unit_delay = 0.0;
    }
}

pub(crate) fn source_hud_end_units(
    objects: impl Iterator<Item = (u32, ObjectKind, TeamType, Option<RobotType>)>,
    team: TeamType,
) -> Vec<HudEndUnit> {
    let mut units = objects
        .filter_map(|(ref_id, kind, object_team, driver)| {
            if object_team != team {
                return None;
            }
            let (robot, in_vehicle) = match kind {
                ObjectKind::Robot(robot) => (robot, false),
                ObjectKind::Vehicle(_) | ObjectKind::Cannon(_) => {
                    (driver.unwrap_or(RobotType::Grunt), true)
                }
                _ => return None,
            };
            Some(HudEndUnit {
                ref_id,
                robot,
                in_vehicle,
            })
        })
        .collect::<Vec<_>>();
    units.sort_by_key(|unit| unit.ref_id);
    units
}

pub(crate) fn queue_debug_end_animation(
    mut debug: ResMut<EndAnimationDebug>,
    local_player: Res<LocalPlayerState>,
    objects: Query<(&GameObjectEntity, &ObjectTeam, Option<&DriverHealth>)>,
    mut packets: ResMut<TeamEndedClientQueue>,
    mut selected_portrait_state: ResMut<SelectedPortraitAnimationState>,
    mut portrait_sounds: ResMut<PortraitAnimationSoundQueue>,
    mut selection: ResMut<SelectionState>,
) {
    let team = local_player.team();
    let units = source_hud_end_units(
        objects.iter().map(|(object, object_team, driver)| {
            (
                object.ref_id,
                object.kind,
                object_team.0,
                driver.map(|driver| driver.driver_kind),
            )
        }),
        team,
    );
    if units.is_empty() {
        return;
    }
    if let Some(won) = debug.requested_win {
        if !debug.end_emitted {
            packets.pending.push(TeamEndedClientOutcome {
                team,
                won,
                units: units.clone(),
            });
            debug.end_emitted = true;
        }
    }
    if debug.requested_portrait && !debug.portrait_emitted {
        let unit = units[0];
        let kind = PortraitAnimationKind::SelectedCommon(0);
        selected_portrait_state.start(PortraitAnimationEvent {
            ref_id: unit.ref_id,
            kind,
        });
        portrait_sounds.pending.push(kind);
        selection.selected_refs = vec![unit.ref_id];
        debug.portrait_emitted = true;
    }
}

#[derive(Resource)]
pub(crate) struct PortraitAssets {
    desert_backdrop: Handle<Image>,
    volcanic_backdrop: Handle<Image>,
    arctic_backdrop: Handle<Image>,
    jungle_backdrop: Handle<Image>,
    city_backdrop: Handle<Image>,
    vehicle_backdrop: Handle<Image>,
    frames: Vec<PortraitFrameSet>,
}

struct PortraitFrameSet {
    robot: RobotType,
    team: TeamType,
    frames: Vec<Handle<Image>>,
}

impl PortraitAssets {
    pub(crate) fn load(asset_server: &AssetServer) -> Self {
        let mut frames = Vec::new();
        for robot in [
            RobotType::Grunt,
            RobotType::Psycho,
            RobotType::Sniper,
            RobotType::Tough,
            RobotType::Pyro,
            RobotType::Laser,
        ] {
            for team in [
                TeamType::Red,
                TeamType::Blue,
                TeamType::Green,
                TeamType::Yellow,
            ] {
                frames.push(PortraitFrameSet {
                    robot,
                    team,
                    frames: (0..40)
                        .filter_map(|frame| units::robots::portrait_frame_path(robot, team, frame))
                        .map(|path| asset_server.load(path))
                        .collect(),
                });
            }
        }

        Self {
            desert_backdrop: asset_server.load("other/hud/backdrop_desert.bmp"),
            volcanic_backdrop: asset_server.load("other/hud/backdrop_volcanic.bmp"),
            arctic_backdrop: asset_server.load("other/hud/backdrop_arctic.bmp"),
            jungle_backdrop: asset_server.load("other/hud/backdrop_jungle.bmp"),
            city_backdrop: asset_server.load("other/hud/backdrop_city.bmp"),
            vehicle_backdrop: asset_server.load("other/hud/backdrop_vehicle.bmp"),
            frames,
        }
    }

    fn backdrop(&self, planet: PlanetType, in_vehicle: bool) -> Handle<Image> {
        if in_vehicle {
            return self.vehicle_backdrop.clone();
        }
        match planet {
            PlanetType::Desert => self.desert_backdrop.clone(),
            PlanetType::Volcanic => self.volcanic_backdrop.clone(),
            PlanetType::Arctic => self.arctic_backdrop.clone(),
            PlanetType::Jungle => self.jungle_backdrop.clone(),
            PlanetType::City => self.city_backdrop.clone(),
        }
    }

    fn frame(&self, robot: RobotType, team: TeamType, index: usize) -> Option<Handle<Image>> {
        let team = team.atlas_team();
        self.frames
            .iter()
            .find(|set| set.robot == robot && set.team == team)
            .and_then(|set| set.frames.get(index.min(39)))
            .cloned()
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(crate) enum HudPortraitLayer {
    Backdrop,
    Head,
    Eyes,
    Mouth,
    Shoulders,
    Hand,
}

pub(crate) fn spawn_hud_portrait(commands: &mut Commands, assets: &PortraitAssets) {
    let default_face = assets
        .frame(RobotType::Grunt, TeamType::Red, 1)
        .unwrap_or_default();
    for (layer, z) in [
        (HudPortraitLayer::Backdrop, 720.0),
        (HudPortraitLayer::Head, 721.0),
        (HudPortraitLayer::Eyes, 722.0),
        (HudPortraitLayer::Mouth, 723.0),
        (HudPortraitLayer::Shoulders, 724.0),
        (HudPortraitLayer::Hand, 725.0),
    ] {
        let image = if layer == HudPortraitLayer::Backdrop {
            assets.desert_backdrop.clone()
        } else {
            default_face.clone()
        };
        commands.spawn((
            Sprite::from_image(image),
            bevy::sprite::Anchor::TOP_LEFT,
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, z),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::BasePoint {
                point: PORTRAIT_TOP_LEFT,
            },
            layer,
            Name::new("hud_portrait_layer"),
        ));
    }
}

pub(crate) fn process_portrait_animation_state(
    time: Res<Time<Real>>,
    selection: Res<SelectionState>,
    mut portrait_state: ResMut<PortraitAnimationState>,
    mut selected_portrait_state: ResMut<SelectedPortraitAnimationState>,
    mut idle_state: ResMut<PortraitIdleState>,
    mut rng: ResMut<CombatRng>,
    objects: Query<&GameObjectEntity>,
) {
    let delta_secs = time.delta_secs().max(0.0);
    portrait_state.process(delta_secs);

    if let Some(event) = selected_portrait_state.active_event()
        && idle_state.subject_ref_id != Some(event.ref_id)
    {
        idle_state.subject_ref_id = Some(event.ref_id);
        idle_state.delay_remaining = 0.5;
    }
    if selected_portrait_state.process(delta_secs) {
        idle_state.delay_remaining = source_random_portrait_delay(&mut rng);
    }
    if selected_portrait_state.active_event().is_some() {
        return;
    }

    let subject_ref_id = selection.selected_refs.first().copied().filter(|ref_id| {
        objects.iter().any(|object| {
            object.ref_id == *ref_id
                && matches!(
                    object.kind,
                    ObjectKind::Robot(_) | ObjectKind::Vehicle(_) | ObjectKind::Cannon(_)
                )
        })
    });
    if let Some(event) =
        advance_portrait_idle(&mut idle_state, subject_ref_id, delta_secs, &mut rng)
    {
        selected_portrait_state.start(event);
    }
}

fn source_random_portrait_delay(rng: &mut CombatRng) -> f32 {
    0.5 + rng.index(50) as f32 * 0.1
}

fn advance_portrait_idle(
    state: &mut PortraitIdleState,
    subject_ref_id: Option<u32>,
    delta_secs: f32,
    rng: &mut CombatRng,
) -> Option<PortraitAnimationEvent> {
    let Some(subject_ref_id) = subject_ref_id else {
        state.subject_ref_id = None;
        state.delay_remaining = 0.5;
        return None;
    };
    if state.subject_ref_id != Some(subject_ref_id) {
        state.subject_ref_id = Some(subject_ref_id);
        state.delay_remaining = 0.5;
    }
    state.delay_remaining -= delta_secs.max(0.0);
    if state.delay_remaining > 0.0 {
        return None;
    }
    Some(PortraitAnimationEvent {
        ref_id: subject_ref_id,
        kind: PortraitAnimationKind::Idle(rng.index(13) as u8),
    })
}

pub(crate) fn process_hud_end_animations(
    time: Res<Time>,
    local_player: Res<LocalPlayerState>,
    mut packets: ResMut<TeamEndedClientQueue>,
    mut state: ResMut<HudEndAnimationState>,
    mut portrait_state: ResMut<PortraitAnimationState>,
    mut portrait_sounds: ResMut<PortraitAnimationSoundQueue>,
    mut rng: ResMut<CombatRng>,
) {
    apply_team_ended_outcomes(
        &mut state,
        local_player.team(),
        std::mem::take(&mut packets.pending),
    );
    let Some((unit, kind)) = advance_end_animation(&mut state, time.delta_secs(), &mut rng) else {
        return;
    };
    portrait_state.start(PortraitAnimationEvent {
        ref_id: unit.ref_id,
        kind,
    });
    portrait_sounds.pending.push(kind);
}

fn apply_team_ended_outcomes(
    state: &mut HudEndAnimationState,
    local_team: TeamType,
    outcomes: Vec<TeamEndedClientOutcome>,
) {
    for outcome in outcomes {
        if outcome.team == local_team {
            state.start(outcome.units, outcome.won);
        }
    }
}

fn advance_end_animation(
    state: &mut HudEndAnimationState,
    virtual_delta: f32,
    rng: &mut CombatRng,
) -> Option<(HudEndUnit, PortraitAnimationKind)> {
    if !state.active {
        return None;
    }
    if state.remaining.is_empty() {
        state.active = false;
        return None;
    }
    state.next_unit_delay = (state.next_unit_delay - virtual_delta.max(0.0)).max(0.0);
    if state.next_unit_delay > 0.0 {
        return None;
    }
    state.next_unit_delay = END_UNIT_INTERVAL_SECONDS;

    let unit = state.remaining.pop()?;
    state.current_unit = Some(unit);
    let animation = rng.index(3) as u8;
    let sound = next_end_sound_variant(state.won, &mut state.last_sound_variant, rng);
    let kind = if state.won {
        PortraitAnimationKind::EndWin { animation, sound }
    } else {
        PortraitAnimationKind::EndLose { animation, sound }
    };
    Some((unit, kind))
}

fn next_end_sound_variant(
    won: bool,
    last_sound_variant: &mut Option<u8>,
    rng: &mut CombatRng,
) -> u8 {
    let count = if won { 6 } else { 7 };
    let mut next = rng.index(count) as u8;
    while *last_sound_variant == Some(next) {
        next = rng.index(count) as u8;
    }
    *last_sound_variant = Some(next);
    next
}

pub(crate) fn sync_hud_portrait_visual(
    map: Res<CurrentMap>,
    local_player: Res<LocalPlayerState>,
    assets: Res<PortraitAssets>,
    images: Res<Assets<Image>>,
    selection: Res<SelectionState>,
    state: Res<HudEndAnimationState>,
    portrait_state: Res<PortraitAnimationState>,
    selected_portrait_state: Res<SelectedPortraitAnimationState>,
    objects: Query<(&GameObjectEntity, &ObjectTeam, Option<&DriverHealth>)>,
    mut layers: Query<(
        &HudPortraitLayer,
        &mut Sprite,
        &mut Visibility,
        &mut HudAnchor,
    )>,
) {
    let overlay_event = portrait_state.active_event();
    let active_is_end = overlay_event.is_some_and(|event| {
        matches!(
            event.kind,
            PortraitAnimationKind::EndWin { .. } | PortraitAnimationKind::EndLose { .. }
        )
    });
    let selected_event = selected_portrait_state.active_event();
    let subject = if active_is_end {
        state.current_unit.map(|unit| PortraitSubject {
            unit,
            team: local_player.team().atlas_team(),
        })
    } else if let Some(event) = overlay_event {
        portrait_subject_for_ref(&objects, event.ref_id)
    } else if state.active {
        state.current_unit.map(|unit| PortraitSubject {
            unit,
            team: local_player.team().atlas_team(),
        })
    } else {
        selected_event
            .and_then(|event| portrait_subject_for_ref(&objects, event.ref_id))
            .or_else(|| {
                selection
                    .selected_refs
                    .first()
                    .and_then(|ref_id| portrait_subject_for_ref(&objects, *ref_id))
            })
    };
    let Some(subject) = subject else {
        for (_, _, mut visibility, _) in &mut layers {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let frame = if let Some(event) = overlay_event {
        source_portrait_frame(event.kind, portrait_state.elapsed_secs())
    } else if state.active {
        STILL_FRAME
    } else if let Some(event) = selected_event {
        source_portrait_frame(event.kind, selected_portrait_state.elapsed_secs())
    } else {
        STILL_FRAME
    };
    let face_offset_y = match subject.unit.robot {
        RobotType::Grunt => 0.0,
        RobotType::Sniper => 4.0,
        _ => 2.0,
    };
    let team = subject.team;

    for (layer, mut sprite, mut visibility, mut anchor) in &mut layers {
        sprite.rect = None;
        let (image, point, visible, rect) = match *layer {
            HudPortraitLayer::Backdrop => (
                Some(assets.backdrop(map.0.basics.terrain_type, subject.unit.in_vehicle)),
                PORTRAIT_TOP_LEFT,
                true,
                None,
            ),
            HudPortraitLayer::Head => (
                assets.frame(subject.unit.robot, team, 1 + frame.look_direction),
                PORTRAIT_TOP_LEFT + Vec2::new(frame.head_x, frame.head_y),
                true,
                None,
            ),
            HudPortraitLayer::Eyes => (
                assets.frame(subject.unit.robot, team, 20 + frame.eyes),
                PORTRAIT_TOP_LEFT
                    + Vec2::new(14.0 + frame.head_x, 8.0 + frame.head_y + face_offset_y),
                frame.look_direction == 0,
                None,
            ),
            HudPortraitLayer::Mouth => (
                assets.frame(subject.unit.robot, team, 4 + frame.mouth),
                PORTRAIT_TOP_LEFT
                    + Vec2::new(22.0 + frame.head_x, 24.0 + frame.head_y + face_offset_y),
                frame.look_direction == 0,
                None,
            ),
            HudPortraitLayer::Shoulders => (
                assets.frame(subject.unit.robot, team, 0),
                PORTRAIT_TOP_LEFT
                    + Vec2::new(
                        0.0,
                        PORTRAIT_SIZE.y
                            - units::robots::portrait_shoulders_height(subject.unit.robot),
                    ),
                true,
                None,
            ),
            HudPortraitLayer::Hand => {
                let image = assets.frame(subject.unit.robot, team, 31 + frame.hand);
                let clipped = image.as_ref().and_then(|image| {
                    clipped_hand_blit(image, frame.hand_x, frame.hand_y, &images)
                });
                (
                    image,
                    clipped
                        .map(|(point, _)| PORTRAIT_TOP_LEFT + point)
                        .unwrap_or(PORTRAIT_TOP_LEFT),
                    frame.hand_do_render && clipped.is_some(),
                    clipped.map(|(_, rect)| rect),
                )
            }
        };
        if let Some(image) = image {
            sprite.image = image;
        }
        sprite.rect = rect;
        *anchor = HudAnchor::BasePoint { point };
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[derive(Clone, Copy)]
struct PortraitSubject {
    unit: HudEndUnit,
    team: TeamType,
}

fn portrait_subject_for_ref(
    objects: &Query<(&GameObjectEntity, &ObjectTeam, Option<&DriverHealth>)>,
    ref_id: u32,
) -> Option<PortraitSubject> {
    objects.iter().find_map(|(object, team, driver)| {
        (object.ref_id == ref_id).then(|| {
            let (robot, in_vehicle) = match object.kind {
                ObjectKind::Robot(robot) => (robot, false),
                ObjectKind::Vehicle(_) | ObjectKind::Cannon(_) => (
                    driver.map_or(RobotType::Grunt, |driver| driver.driver_kind),
                    true,
                ),
                _ => return None,
            };
            Some(PortraitSubject {
                unit: HudEndUnit {
                    ref_id,
                    robot,
                    in_vehicle,
                },
                team: team.0.atlas_team(),
            })
        })?
    })
}

fn clipped_hand_blit(
    image: &Handle<Image>,
    x: f32,
    y: f32,
    images: &Assets<Image>,
) -> Option<(Vec2, Rect)> {
    let size = images.get(image)?.size_f32();
    clipped_hand_rect(size, x, y)
}

fn clipped_hand_rect(size: Vec2, x: f32, y: f32) -> Option<(Vec2, Rect)> {
    let source = Vec2::new((-x).max(0.0), (-y).max(0.0));
    let destination = Vec2::new(x.max(0.0), y.max(0.0));
    let visible = (size - source).min(PORTRAIT_SIZE - destination);
    if visible.x <= 0.0 || visible.y <= 0.0 {
        return None;
    }
    Some((destination, Rect::from_corners(source, source + visible)))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PortraitFrame {
    duration: f32,
    look_direction: usize,
    head_x: f32,
    head_y: f32,
    mouth: usize,
    eyes: usize,
    hand_do_render: bool,
    hand: usize,
    hand_x: f32,
    hand_y: f32,
}

const fn frame(duration: u8, head_y: u8, mouth: u8, eyes: u8) -> PortraitFrame {
    full_frame(duration, 0, head_y as i8, mouth, eyes, false, 0, 0, 0)
}

const fn full_frame(
    duration: u8,
    look_direction: u8,
    head_y: i8,
    mouth: u8,
    eyes: u8,
    hand_do_render: bool,
    hand: u8,
    hand_x: i8,
    hand_y: i8,
) -> PortraitFrame {
    PortraitFrame {
        duration: duration as f32 * PORTRAIT_FRAME_TIME_UNIT,
        look_direction: look_direction as usize,
        head_x: 4.0,
        head_y: head_y as f32,
        mouth: mouth as usize,
        eyes: eyes as usize,
        hand_do_render,
        hand: hand as usize,
        hand_x: hand_x as f32,
        hand_y: hand_y as f32,
    }
}

const STILL_FRAME: PortraitFrame = frame(0, 2, 0, 0);

include!("portrait_frames_generated.rs");

fn source_portrait_frame(kind: PortraitAnimationKind, elapsed: f32) -> PortraitFrame {
    let frames = source_portrait_frames(kind);
    let mut selected = STILL_FRAME;
    let mut current_duration = 0.0;
    for frame in frames {
        if current_duration <= elapsed.max(0.0) {
            selected = *frame;
        } else {
            break;
        }
        current_duration += frame.duration;
    }
    selected
}

fn source_portrait_frames(kind: PortraitAnimationKind) -> &'static [PortraitFrame] {
    let source_id = match kind.wire_id() {
        63 | 66 => 21,
        64 | 67 => 60,
        65 | 68 => 19,
        id @ 0..=62 => id as usize,
        _ => return &[],
    };
    SOURCE_PORTRAIT_ANIMATIONS[source_id]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_end_sequences_keep_copied_animation_durations() {
        let duration =
            |frames: &[PortraitFrame]| frames.iter().map(|frame| frame.duration).sum::<f32>();
        assert!((duration(SOURCE_PORTRAIT_ANIMATIONS[21]) - 0.525).abs() < 0.000_001);
        assert!((duration(SOURCE_PORTRAIT_ANIMATIONS[60]) - 0.855).abs() < 0.000_001);
        assert!((duration(SOURCE_PORTRAIT_ANIMATIONS[19]) - 0.675).abs() < 0.000_001);
    }

    #[test]
    fn every_runtime_portrait_kind_uses_the_source_frame_total() {
        let mut kinds = vec![
            PortraitAnimationKind::WereUnderAttack,
            PortraitAnimationKind::TargetDestroyed,
            PortraitAnimationKind::TerritoryTaken,
            PortraitAnimationKind::GunCaptured,
            PortraitAnimationKind::VehicleCaptured,
            PortraitAnimationKind::GrenadesCollected,
        ];
        kinds.extend((0..4).map(PortraitAnimationKind::SelectedCommon));
        kinds.extend(
            [
                RobotType::Grunt,
                RobotType::Psycho,
                RobotType::Sniper,
                RobotType::Tough,
                RobotType::Laser,
                RobotType::Pyro,
            ]
            .map(PortraitAnimationKind::SelectedRobotReporting),
        );
        kinds.extend((0..12).map(PortraitAnimationKind::Acknowledge));
        kinds.extend((0..3).map(PortraitAnimationKind::AcknowledgeNoWay));
        kinds.extend((0..6).map(PortraitAnimationKind::UnderAttackRepeat));
        kinds.extend((0..7).map(PortraitAnimationKind::GoodHit));
        kinds.extend((0..13).map(PortraitAnimationKind::Idle));
        kinds.extend((0..3).map(|animation| PortraitAnimationKind::EndWin {
            animation,
            sound: 0,
        }));
        kinds.extend((0..3).map(|animation| PortraitAnimationKind::EndLose {
            animation,
            sound: 0,
        }));

        for kind in kinds {
            let duration = source_portrait_frames(kind)
                .iter()
                .map(|frame| frame.duration)
                .sum::<f32>();
            assert!(
                (duration - kind.source_total_duration_secs()).abs() < 0.000_002,
                "{kind:?}: generated={duration} typed={}",
                kind.source_total_duration_secs()
            );
        }
    }

    #[test]
    fn source_frame_selection_uses_cumulative_frame_starts() {
        assert_eq!(
            source_portrait_frame(
                PortraitAnimationKind::EndWin {
                    animation: 0,
                    sound: 0,
                },
                0.0,
            ),
            SOURCE_PORTRAIT_ANIMATIONS[21][0]
        );
        assert_eq!(
            source_portrait_frame(
                PortraitAnimationKind::EndWin {
                    animation: 0,
                    sound: 0,
                },
                0.061,
            ),
            SOURCE_PORTRAIT_ANIMATIONS[21][1]
        );
    }

    #[test]
    fn random_idle_wait_and_animation_ranges_match_source() {
        let mut rng = CombatRng(1);
        for _ in 0..1_000 {
            let delay = source_random_portrait_delay(&mut rng);
            assert!((0.5..=5.4).contains(&delay));
        }

        let mut idle = PortraitIdleState::default();
        assert_eq!(
            advance_portrait_idle(&mut idle, Some(77), 0.49, &mut rng),
            None
        );
        let event = advance_portrait_idle(&mut idle, Some(77), 0.01, &mut rng).unwrap();
        assert_eq!(event.ref_id, 77);
        assert!(matches!(event.kind, PortraitAnimationKind::Idle(0..=12)));
        assert_eq!(advance_portrait_idle(&mut idle, None, 1.0, &mut rng), None);
        assert_eq!(idle.subject_ref_id, None);
        assert_eq!(idle.delay_remaining, 0.5);
    }

    #[test]
    fn reset_clears_parade_but_preserves_static_nonrepeat_sound_memory() {
        let mut state = HudEndAnimationState {
            active: true,
            won: true,
            remaining: vec![HudEndUnit {
                ref_id: 1,
                robot: RobotType::Grunt,
                in_vehicle: false,
            }],
            current_unit: None,
            next_unit_delay: 1.0,
            last_sound_variant: Some(4),
        };
        state.clear_for_reset();
        assert!(!state.active);
        assert!(state.remaining.is_empty());
        assert_eq!(state.last_sound_variant, Some(4));
    }

    #[test]
    fn local_team_packet_starts_reverse_order_parade_at_source_interval() {
        let first = HudEndUnit {
            ref_id: 1,
            robot: RobotType::Grunt,
            in_vehicle: false,
        };
        let last = HudEndUnit {
            ref_id: 9,
            robot: RobotType::Laser,
            in_vehicle: true,
        };
        let mut state = HudEndAnimationState::default();
        apply_team_ended_outcomes(
            &mut state,
            TeamType::Red,
            vec![
                TeamEndedClientOutcome {
                    team: TeamType::Blue,
                    won: false,
                    units: vec![first],
                },
                TeamEndedClientOutcome {
                    team: TeamType::Red,
                    won: true,
                    units: vec![first, last],
                },
            ],
        );

        let mut rng = CombatRng::default();
        assert_eq!(
            advance_end_animation(&mut state, 0.0, &mut rng).unwrap().0,
            last
        );
        assert!(advance_end_animation(&mut state, 1.199, &mut rng).is_none());
        assert_eq!(
            advance_end_animation(&mut state, 0.002, &mut rng)
                .unwrap()
                .0,
            first
        );
        assert!(advance_end_animation(&mut state, 0.0, &mut rng).is_none());
        assert!(!state.active);
    }

    #[test]
    fn end_voice_variants_do_not_repeat_consecutively() {
        let mut rng = CombatRng::default();
        let mut last = None;
        let first = next_end_sound_variant(true, &mut last, &mut rng);
        let second = next_end_sound_variant(true, &mut last, &mut rng);
        assert_ne!(first, second);
        assert!(first < 6 && second < 6);

        let lose = next_end_sound_variant(false, &mut last, &mut rng);
        assert!(lose < 7);
        assert_ne!(Some(lose), Some(second));
    }

    #[test]
    fn hand_blit_clips_to_source_portrait_view() {
        assert_eq!(
            clipped_hand_rect(Vec2::new(48.0, 32.0), -4.0, -4.0),
            Some((
                Vec2::ZERO,
                Rect::from_corners(Vec2::splat(4.0), Vec2::new(48.0, 32.0)),
            ))
        );
        assert_eq!(
            clipped_hand_rect(Vec2::new(48.0, 32.0), 62.0, 60.0),
            Some((
                Vec2::new(62.0, 60.0),
                Rect::from_corners(Vec2::ZERO, Vec2::new(24.0, 14.0)),
            ))
        );
        assert!(clipped_hand_rect(Vec2::new(48.0, 32.0), 90.0, 0.0).is_none());
    }
}
