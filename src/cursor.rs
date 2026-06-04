use std::collections::HashMap;

use bevy::{camera::visibility::RenderLayers, prelude::*, window::CursorOptions};

use crate::{
    components::*,
    constants::{HUD_HEIGHT, HUD_LAYER, HUD_WIDTH},
    enter::{can_enter_fort, point_in_fort_entrance_rect},
    grenades::can_pickup_grenades,
    original::{
        objects::{ItemType, ObjectKind},
        types::TeamType,
    },
};

const CURSOR_FRAMES: usize = 4;
const CURSOR_FRAME_TIME: f32 = 0.2;
const CURSOR_SIZE: Vec2 = Vec2::splat(16.0);
const PLAYER_TEAM: TeamType = TeamType::Red;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ZCursorKind {
    Cursor,
    Place,
    Placed,
    Attack,
    Attacked,
    Grab,
    Grabbed,
    Grenade,
    Grenaded,
    Repair,
    Repaired,
    Nono,
    Cannon,
    Cannoned,
    Enter,
    Entered,
    Exit,
    Exited,
}

#[derive(Resource)]
pub(crate) struct ZCursorAssets {
    frames: HashMap<(ZCursorKind, TeamType, usize), Handle<Image>>,
}

#[derive(Resource)]
pub(crate) struct ZCursorState {
    pub(crate) team: TeamType,
    pub(crate) kind: ZCursorKind,
    pub(crate) frame: usize,
    elapsed: f32,
}

impl Default for ZCursorState {
    fn default() -> Self {
        Self {
            team: PLAYER_TEAM,
            kind: ZCursorKind::Cursor,
            frame: 0,
            elapsed: 0.0,
        }
    }
}

#[derive(Component)]
pub(crate) struct ZCursorSprite;

#[derive(Clone, Copy, Debug)]
struct CursorSelectionFacts {
    selected_any: bool,
    single_selected_ref: Option<u32>,
    can_move: bool,
    can_attack: bool,
    have_explosives: bool,
    can_pickup_grenades: bool,
}

#[derive(Clone, Copy, Debug)]
struct CursorHoverFacts {
    ref_id: u32,
    kind: ObjectKind,
    team: TeamType,
    attacked_only_by_explosives: bool,
    can_eject_drivers: bool,
    can_enter_fort: bool,
}

pub(crate) fn load_cursor_assets(asset_server: &AssetServer) -> ZCursorAssets {
    let mut frames = HashMap::new();
    for kind in all_cursor_kinds() {
        for team in cursor_teams_for_kind(kind) {
            for frame in 0..CURSOR_FRAMES {
                frames.insert(
                    (kind, team, frame),
                    asset_server.load(cursor_asset_path(kind, team, frame)),
                );
            }
        }
    }

    ZCursorAssets { frames }
}

pub(crate) fn spawn_zcursor(mut commands: Commands, assets: Res<ZCursorAssets>) {
    commands.spawn((
        Sprite {
            image: assets.image(ZCursorKind::Cursor, PLAYER_TEAM, 0),
            custom_size: Some(CURSOR_SIZE),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1000.0),
        RenderLayers::layer(HUD_LAYER),
        ZCursorSprite,
        Name::new("z_cursor"),
    ));
}

pub(crate) fn update_zcursor(
    time: Res<Time>,
    assets: Res<ZCursorAssets>,
    mut state: ResMut<ZCursorState>,
    mut window_queries: ParamSet<(Query<&Window>, Query<&mut CursorOptions>)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    selection: Res<SelectionState>,
    cannon_placement: Res<CannonPlacementState>,
    production_window: Res<ProductionWindowState>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut queries: ParamSet<(
        Query<(
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&GrenadeInventory>,
        )>,
        Query<(&mut Sprite, &mut Transform), With<ZCursorSprite>>,
    )>,
) {
    for mut options in &mut window_queries.p1() {
        options.visible = false;
    }

    let windows = window_queries.p0();
    let Ok(window) = windows.single() else {
        return;
    };

    let Some(screen_pos) = window.cursor_position() else {
        return;
    };

    state.elapsed += time.delta_secs();
    if state.elapsed >= CURSOR_FRAME_TIME {
        state.elapsed %= CURSOR_FRAME_TIME;
        state.frame = (state.frame + 1) % CURSOR_FRAMES;
    }

    if let Some(kind) = cursor_kind_for_current_input(
        &window,
        &camera_query,
        &selection,
        &cannon_placement,
        &production_window,
        &mouse,
        &queries.p0(),
    ) {
        if state.kind != kind {
            state.kind = kind;
            state.frame = 0;
            state.elapsed = 0.0;
        }
    }

    let mut cursor_query = queries.p1();
    let Ok((mut sprite, mut transform)) = cursor_query.single_mut() else {
        return;
    };
    sprite.image = assets.image(state.kind, state.team, state.frame);
    sprite.custom_size = Some(CURSOR_SIZE);
    let center = cursor_sprite_center(
        screen_pos,
        Vec2::new(window.width(), window.height()),
        state.kind,
    );
    transform.translation.x = center.x;
    transform.translation.y = center.y;
}

impl ZCursorAssets {
    fn image(&self, kind: ZCursorKind, team: TeamType, frame: usize) -> Handle<Image> {
        self.frames
            .get(&(kind, cursor_asset_team(kind, team), frame % CURSOR_FRAMES))
            .cloned()
            .unwrap_or_else(|| {
                self.frames
                    .get(&(ZCursorKind::Cursor, TeamType::Null, 0))
                    .expect("null cursor frame should be loaded")
                    .clone()
            })
    }
}

fn cursor_kind_for_current_input(
    window: &Window,
    camera_query: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    selection: &SelectionState,
    cannon_placement: &CannonPlacementState,
    production_window: &ProductionWindowState,
    mouse: &ButtonInput<MouseButton>,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&GrenadeInventory>,
    )>,
) -> Option<ZCursorKind> {
    if production_window.input_captured || production_window.open.is_some() {
        return Some(ZCursorKind::Cursor);
    }
    if cannon_placement.pending.is_some() {
        return Some(ZCursorKind::Cannon);
    }
    if mouse.pressed(MouseButton::Left) {
        return Some(ZCursorKind::Cursor);
    }

    let screen_pos = window.cursor_position()?;
    let window_size = Vec2::new(window.width(), window.height());
    if screen_pos.x >= window_size.x - HUD_WIDTH || screen_pos.y >= window_size.y - HUD_HEIGHT {
        return Some(ZCursorKind::Cursor);
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return None;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) else {
        return None;
    };

    let selection_facts = cursor_selection_facts(selection, object_query);
    let hover = cursor_hover_facts(world_pos, object_query);
    Some(determine_cursor_kind(selection_facts, hover, PLAYER_TEAM))
}

fn cursor_selection_facts(
    selection: &SelectionState,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&GrenadeInventory>,
    )>,
) -> CursorSelectionFacts {
    let mut facts = CursorSelectionFacts {
        selected_any: false,
        single_selected_ref: (selection.selected_refs.len() == 1)
            .then(|| selection.selected_refs[0]),
        can_move: false,
        can_attack: false,
        have_explosives: false,
        can_pickup_grenades: false,
    };

    for (object, _, selectable, team, stats, inventory) in object_query {
        if team.0 != PLAYER_TEAM || !selection.selected_refs.contains(&object.ref_id) {
            continue;
        }

        facts.selected_any = true;
        facts.can_move |= selectable.mobile;
        facts.can_attack |= stats.can_attack();
        facts.have_explosives |=
            stats.has_explosive_damage() || inventory.is_some_and(|inventory| inventory.amount > 0);
        facts.can_pickup_grenades |=
            inventory.is_some_and(|inventory| can_pickup_grenades(object.kind, inventory.amount));
    }

    facts
}

fn cursor_hover_facts(
    world_pos: Vec2,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&GrenadeInventory>,
    )>,
) -> Option<CursorHoverFacts> {
    object_query
        .iter()
        .filter(|(_, transform, selectable, _, stats, _)| {
            !stats.destroyed()
                && point_in_object_rect(
                    world_pos,
                    transform.translation.truncate(),
                    selectable.selection_size.max(Vec2::splat(16.0)),
                )
        })
        .min_by(
            |(_, a_transform, _, _, _, _), (_, b_transform, _, _, _, _)| {
                let a = world_pos.distance_squared(a_transform.translation.truncate());
                let b = world_pos.distance_squared(b_transform.translation.truncate());
                a.total_cmp(&b)
            },
        )
        .map(|(object, transform, selectable, team, stats, _)| CursorHoverFacts {
            ref_id: object.ref_id,
            kind: object.kind,
            team: team.0,
            attacked_only_by_explosives: stats.attacked_only_by_explosives,
            can_eject_drivers: can_eject_drivers(object.kind, *stats),
            can_enter_fort: matches!(object.kind, ObjectKind::Building(building) if can_enter_fort(object.kind, team.0, PLAYER_TEAM, *stats)
                && point_in_fort_entrance_rect(world_pos, transform.translation.truncate(), selectable.selection_size, building)),
        })
}

fn determine_cursor_kind(
    selection: CursorSelectionFacts,
    hover: Option<CursorHoverFacts>,
    player_team: TeamType,
) -> ZCursorKind {
    if !selection.selected_any {
        return ZCursorKind::Cursor;
    }

    let Some(hover) = hover else {
        return if selection.can_move {
            ZCursorKind::Place
        } else {
            ZCursorKind::Cannon
        };
    };

    if hover.team == player_team {
        if selection.single_selected_ref == Some(hover.ref_id) && hover.can_eject_drivers {
            return ZCursorKind::Exit;
        }
        return ZCursorKind::Place;
    }

    if selection.can_move {
        if matches!(hover.kind, ObjectKind::MapItem(id) if id == ItemType::Grenades as u8)
            && selection.can_pickup_grenades
        {
            return ZCursorKind::Grab;
        }
        if matches!(hover.kind, ObjectKind::MapItem(id) if id == ItemType::Flag as u8) {
            return ZCursorKind::Grab;
        }
        if hover.team == TeamType::Null
            && matches!(hover.kind, ObjectKind::Cannon(_) | ObjectKind::Vehicle(_))
        {
            return ZCursorKind::Enter;
        }
        if hover.can_enter_fort {
            return ZCursorKind::Place;
        }
    } else if matches!(hover.kind, ObjectKind::MapItem(id) if id == ItemType::Flag as u8) {
        return ZCursorKind::Cannon;
    }

    if !selection.can_attack || (hover.attacked_only_by_explosives && !selection.have_explosives) {
        ZCursorKind::Nono
    } else {
        ZCursorKind::Attack
    }
}

fn point_in_object_rect(point: Vec2, center: Vec2, size: Vec2) -> bool {
    let half = size * 0.5;
    point.x >= center.x - half.x
        && point.x <= center.x + half.x
        && point.y >= center.y - half.y
        && point.y <= center.y + half.y
}

fn cursor_sprite_center(screen_pos: Vec2, window_size: Vec2, kind: ZCursorKind) -> Vec2 {
    let top_left_shift = if kind == ZCursorKind::Cursor {
        Vec2::ZERO
    } else {
        Vec2::splat(-8.0)
    };
    let screen_center = screen_pos + top_left_shift + CURSOR_SIZE * 0.5;
    Vec2::new(
        screen_center.x - window_size.x * 0.5,
        window_size.y * 0.5 - screen_center.y,
    )
}

fn all_cursor_kinds() -> [ZCursorKind; 18] {
    [
        ZCursorKind::Cursor,
        ZCursorKind::Place,
        ZCursorKind::Placed,
        ZCursorKind::Attack,
        ZCursorKind::Attacked,
        ZCursorKind::Grab,
        ZCursorKind::Grabbed,
        ZCursorKind::Grenade,
        ZCursorKind::Grenaded,
        ZCursorKind::Repair,
        ZCursorKind::Repaired,
        ZCursorKind::Nono,
        ZCursorKind::Cannon,
        ZCursorKind::Cannoned,
        ZCursorKind::Enter,
        ZCursorKind::Entered,
        ZCursorKind::Exit,
        ZCursorKind::Exited,
    ]
}

fn cursor_teams_for_kind(kind: ZCursorKind) -> Vec<TeamType> {
    if kind == ZCursorKind::Cursor {
        return vec![
            TeamType::Null,
            TeamType::Red,
            TeamType::Blue,
            TeamType::Green,
            TeamType::Yellow,
        ];
    }
    if cursor_uses_null_team_asset(kind) {
        vec![TeamType::Null]
    } else {
        vec![
            TeamType::Red,
            TeamType::Blue,
            TeamType::Green,
            TeamType::Yellow,
        ]
    }
}

fn cursor_asset_team(kind: ZCursorKind, team: TeamType) -> TeamType {
    if cursor_uses_null_team_asset(kind) {
        TeamType::Null
    } else {
        team.atlas_team()
    }
}

fn cursor_uses_null_team_asset(kind: ZCursorKind) -> bool {
    matches!(
        kind,
        ZCursorKind::Placed
            | ZCursorKind::Attacked
            | ZCursorKind::Grabbed
            | ZCursorKind::Grenaded
            | ZCursorKind::Repaired
            | ZCursorKind::Entered
            | ZCursorKind::Exited
            | ZCursorKind::Cannoned
    )
}

fn cursor_asset_path(kind: ZCursorKind, team: TeamType, frame: usize) -> String {
    let name = match kind {
        ZCursorKind::Cursor if team == TeamType::Null => "cursor_null".to_string(),
        ZCursorKind::Cursor => format!("cursor_{}", team.asset_name()),
        ZCursorKind::Place => format!("place_{}", team.asset_name()),
        ZCursorKind::Placed => "placed".to_string(),
        ZCursorKind::Attack => format!("attack_{}", team.asset_name()),
        ZCursorKind::Attacked => "attacked".to_string(),
        ZCursorKind::Grab => format!("grab_{}", team.asset_name()),
        ZCursorKind::Grabbed => "grabbed".to_string(),
        ZCursorKind::Grenade => format!("grenade_{}", team.asset_name()),
        ZCursorKind::Grenaded => "grenaded".to_string(),
        ZCursorKind::Repair => format!("repair_{}", team.asset_name()),
        ZCursorKind::Repaired => "repaired".to_string(),
        ZCursorKind::Nono => format!("nono_{}", team.asset_name()),
        ZCursorKind::Cannon => format!("cannon_{}", team.asset_name()),
        ZCursorKind::Cannoned => "cannoned".to_string(),
        ZCursorKind::Enter => format!("enter_{}", team.asset_name()),
        ZCursorKind::Entered => "entered".to_string(),
        ZCursorKind::Exit => format!("exit_{}", team.asset_name()),
        ZCursorKind::Exited => "exited".to_string(),
    };

    format!("cursors/{name}_n{frame:02}.png")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{BuildingType, CannonType, VehicleType};

    fn selected_mobile_attacker() -> CursorSelectionFacts {
        CursorSelectionFacts {
            selected_any: true,
            single_selected_ref: Some(1),
            can_move: true,
            can_attack: true,
            have_explosives: false,
            can_pickup_grenades: true,
        }
    }

    #[test]
    fn cursor_asset_paths_match_original_filenames() {
        assert_eq!(
            cursor_asset_path(ZCursorKind::Cursor, TeamType::Red, 2),
            "cursors/cursor_red_n02.png"
        );
        assert_eq!(
            cursor_asset_path(ZCursorKind::Placed, TeamType::Null, 3),
            "cursors/placed_n03.png"
        );
        assert_eq!(
            cursor_asset_path(ZCursorKind::Nono, TeamType::Blue, 0),
            "cursors/nono_blue_n00.png"
        );
    }

    #[test]
    fn command_cursor_hotspot_matches_original_shift() {
        assert_eq!(
            cursor_sprite_center(
                Vec2::new(100.0, 50.0),
                Vec2::new(800.0, 600.0),
                ZCursorKind::Place
            ),
            Vec2::new(-300.0, 250.0)
        );
        assert_eq!(
            cursor_sprite_center(
                Vec2::new(100.0, 50.0),
                Vec2::new(800.0, 600.0),
                ZCursorKind::Cursor
            ),
            Vec2::new(-292.0, 242.0)
        );
    }

    #[test]
    fn cursor_decision_matches_original_empty_and_cannon_selection() {
        assert_eq!(
            determine_cursor_kind(
                CursorSelectionFacts {
                    selected_any: false,
                    single_selected_ref: None,
                    can_move: false,
                    can_attack: false,
                    have_explosives: false,
                    can_pickup_grenades: false,
                },
                None,
                TeamType::Red,
            ),
            ZCursorKind::Cursor
        );
        assert_eq!(
            determine_cursor_kind(
                CursorSelectionFacts {
                    selected_any: true,
                    single_selected_ref: Some(1),
                    can_move: false,
                    can_attack: true,
                    have_explosives: false,
                    can_pickup_grenades: false,
                },
                None,
                TeamType::Red,
            ),
            ZCursorKind::Cannon
        );
    }

    #[test]
    fn cursor_decision_prefers_grab_and_enter_for_mobile_selection() {
        let selected = selected_mobile_attacker();
        assert_eq!(
            determine_cursor_kind(
                selected,
                Some(CursorHoverFacts {
                    ref_id: 2,
                    kind: ObjectKind::MapItem(ItemType::Flag as u8),
                    team: TeamType::Null,
                    attacked_only_by_explosives: false,
                    can_eject_drivers: false,
                    can_enter_fort: false,
                }),
                TeamType::Red,
            ),
            ZCursorKind::Grab
        );
        assert_eq!(
            determine_cursor_kind(
                selected,
                Some(CursorHoverFacts {
                    ref_id: 2,
                    kind: ObjectKind::Vehicle(VehicleType::Jeep),
                    team: TeamType::Null,
                    attacked_only_by_explosives: false,
                    can_eject_drivers: false,
                    can_enter_fort: false,
                }),
                TeamType::Red,
            ),
            ZCursorKind::Enter
        );
    }

    #[test]
    fn cursor_decision_uses_exit_for_selected_ejectable_object_under_cursor() {
        assert_eq!(
            determine_cursor_kind(
                CursorSelectionFacts {
                    selected_any: true,
                    single_selected_ref: Some(9),
                    can_move: false,
                    can_attack: true,
                    have_explosives: false,
                    can_pickup_grenades: false,
                },
                Some(CursorHoverFacts {
                    ref_id: 9,
                    kind: ObjectKind::Cannon(CannonType::Gatling),
                    team: TeamType::Red,
                    attacked_only_by_explosives: false,
                    can_eject_drivers: true,
                    can_enter_fort: false,
                }),
                TeamType::Red,
            ),
            ZCursorKind::Exit
        );
    }

    #[test]
    fn cursor_decision_blocks_non_explosive_attack_on_buildings() {
        assert_eq!(
            determine_cursor_kind(
                selected_mobile_attacker(),
                Some(CursorHoverFacts {
                    ref_id: 2,
                    kind: ObjectKind::Building(crate::original::objects::BuildingType::FortBack),
                    team: TeamType::Blue,
                    attacked_only_by_explosives: true,
                    can_eject_drivers: false,
                    can_enter_fort: false,
                }),
                TeamType::Red,
            ),
            ZCursorKind::Nono
        );
        assert_eq!(
            determine_cursor_kind(
                CursorSelectionFacts {
                    have_explosives: true,
                    ..selected_mobile_attacker()
                },
                Some(CursorHoverFacts {
                    ref_id: 2,
                    kind: ObjectKind::Building(crate::original::objects::BuildingType::FortBack),
                    team: TeamType::Blue,
                    attacked_only_by_explosives: true,
                    can_eject_drivers: false,
                    can_enter_fort: false,
                }),
                TeamType::Red,
            ),
            ZCursorKind::Attack
        );
    }

    #[test]
    fn cursor_decision_prefers_place_on_enemy_fort_entrance() {
        assert_eq!(
            determine_cursor_kind(
                selected_mobile_attacker(),
                Some(CursorHoverFacts {
                    ref_id: 2,
                    kind: ObjectKind::Building(BuildingType::FortFront),
                    team: TeamType::Blue,
                    attacked_only_by_explosives: true,
                    can_eject_drivers: false,
                    can_enter_fort: true,
                }),
                TeamType::Red,
            ),
            ZCursorKind::Place
        );
    }
}
