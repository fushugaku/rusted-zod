use bevy::{camera::visibility::RenderLayers, prelude::*};

use crate::{
    camera::game_view_size,
    components::*,
    constants::*,
    grenades::can_have_grenades,
    original::{
        map::{MapObjectType, ZMap},
        objects::{CannonType, ItemType, ObjectKind, RobotType},
        settings::MAX_UNIT_HEALTH,
        types::TeamType,
    },
};

const GRENADE_ICON_TOP_LEFT: Vec2 = Vec2::new(575.0, 185.0);
const GRENADE_ICON_SIZE: Vec2 = Vec2::new(24.0, 20.0);
const GRENADE_TEXT_TOP_LEFT: Vec2 = Vec2::new(600.0, 187.0);
const COMPUTER_MESSAGE_TOP_Y: f32 = 20.0;
const COMPUTER_MESSAGE_SIZE: Vec2 = Vec2::new(128.0, 14.0);
const COMPUTER_MESSAGE_BLINK_INTERVAL: f32 = 0.3;
const COMPUTER_MESSAGE_BLINK_FLIPS: u8 = 10;
const COMPUTER_MESSAGE_HOLD_TIME: f32 = 5.0;

pub(crate) fn spawn_hud(
    commands: &mut Commands,
    map: &ZMap,
    hud_assets: &HudAssets,
    zone_ownership: &ZoneOwnership,
) {
    let layout = HudLayout::for_map(map);
    commands.insert_resource(layout);

    commands.spawn((
        Sprite {
            image: hud_assets.side_filler.clone(),
            custom_size: Some(Vec2::new(HUD_WIDTH, 44.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 700.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::SideFiller,
        Name::new("hud_side_filler"),
    ));
    commands.spawn((
        Sprite {
            image: hud_assets.side_panel.clone(),
            custom_size: Some(Vec2::new(HUD_WIDTH, 484.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 700.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::SidePanel,
        Name::new("hud_side_panel"),
    ));
    commands.spawn((
        Sprite {
            image: hud_assets.bottom_left.clone(),
            custom_size: Some(Vec2::new(206.0, HUD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 700.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::BottomLeft,
        Name::new("hud_bottom_left"),
    ));
    commands.spawn((
        Sprite {
            image: hud_assets.bottom_center.clone(),
            custom_size: Some(Vec2::new(212.0, HUD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 700.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::BottomCenter,
        Name::new("hud_bottom_center"),
    ));
    commands.spawn((
        Sprite {
            image: hud_assets.bottom_right.clone(),
            custom_size: Some(Vec2::new(130.0, HUD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 700.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::BottomRightCap,
        Name::new("hud_bottom_right"),
    ));

    for spec in hud_button_specs() {
        spawn_hud_button(commands, hud_assets, spec);
    }
    spawn_hud_selected_object_sprites(commands, hud_assets);
    spawn_hud_grenade_indicator(commands, hud_assets);
    spawn_hud_health_bar(commands, hud_assets);
    spawn_hud_computer_message(commands, hud_assets);

    let minimap_center = layout.render_offset + layout.render_size * 0.5;
    commands.spawn((
        Sprite::from_color(Color::srgb(0.04, 0.04, 0.04), layout.render_size),
        Transform::from_xyz(0.0, 0.0, 710.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::BottomRight {
            offset: layout.bottom_right_offset_for_minimap_local(minimap_center),
        },
        Name::new("minimap_background"),
    ));

    for (zone_index, zone) in map.zones.iter().enumerate() {
        let size = Vec2::new(
            (zone.w as f32 * TILE_SIZE * layout.render_ratio).max(1.0),
            (zone.h as f32 * TILE_SIZE * layout.render_ratio).max(1.0),
        );
        let local = layout.map_pixel_to_minimap_local(Vec2::new(
            zone.x as f32 * TILE_SIZE + zone.w as f32 * TILE_SIZE * 0.5,
            zone.y as f32 * TILE_SIZE + zone.h as f32 * TILE_SIZE * 0.5,
        ));
        commands.spawn((
            Sprite::from_color(
                minimap_zone_color(
                    zone_ownership
                        .owners
                        .get(zone_index)
                        .copied()
                        .unwrap_or(TeamType::Null),
                ),
                size,
            ),
            Transform::from_xyz(0.0, 0.0, 711.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::BottomRight {
                offset: layout.bottom_right_offset_for_minimap_local(local),
            },
            MinimapZone { zone_index },
            Name::new("minimap_zone"),
        ));
    }

    for (ref_id, object) in map.objects.iter().enumerate() {
        if object.object_type == MapObjectType::MapItem && object.object_id == ItemType::Rock as u8
        {
            continue;
        }

        let local = layout.map_pixel_to_minimap_local(Vec2::new(
            object.x as f32 * TILE_SIZE,
            object.y as f32 * TILE_SIZE,
        ));
        commands.spawn((
            Sprite::from_color(object.owner.color(), Vec2::splat(2.0)),
            Transform::from_xyz(0.0, 0.0, 713.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::BottomRight {
                offset: layout.bottom_right_offset_for_minimap_local(local),
            },
            MinimapDot {
                ref_id: ref_id as u32 + 1,
            },
            Name::new("minimap_object"),
        ));
    }

    commands.spawn((
        Sprite::from_color(Color::srgba(0.78, 0.78, 0.0, 0.35), Vec2::splat(1.0)),
        Transform::from_xyz(0.0, 0.0, 714.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::BottomRight {
            offset: layout.bottom_right_offset_for_minimap_local(minimap_center),
        },
        MinimapViewBox,
        Name::new("minimap_view_box"),
    ));
}

fn spawn_hud_computer_message(commands: &mut Commands, hud_assets: &HudAssets) {
    commands.spawn((
        Sprite {
            image: hud_assets.fort_under_attack_message.clone(),
            custom_size: Some(COMPUTER_MESSAGE_SIZE),
            ..default()
        },
        Visibility::Hidden,
        Transform::from_xyz(0.0, 0.0, 730.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::ScreenTopCenter {
            top_y: COMPUTER_MESSAGE_TOP_Y,
            size: COMPUTER_MESSAGE_SIZE,
        },
        HudComputerMessage,
        Name::new("hud_computer_message_fort_under_attack"),
    ));
}

fn spawn_hud_selected_object_sprites(commands: &mut Commands, hud_assets: &HudAssets) {
    for (slot, top_left) in [
        (HudSelectedObjectSlot::Icon, Vec2::new(550.0, 148.0)),
        (HudSelectedObjectSlot::Label, Vec2::new(550.0, 230.0)),
    ] {
        commands.spawn((
            Sprite {
                image: hud_assets.health_empty.clone(),
                custom_size: Some(Vec2::ZERO),
                rect: Some(Rect::new(0.0, 0.0, 0.0, 0.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 721.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::BaseTopLeft {
                top_left,
                size: Vec2::ZERO,
            },
            HudSelectedObjectSprite { slot },
            Name::new("hud_selected_object"),
        ));
    }
}

fn spawn_hud_health_bar(commands: &mut Commands, hud_assets: &HudAssets) {
    for (segment, image) in [
        (HudHealthSegmentKind::Full, hud_assets.health_full.clone()),
        (HudHealthSegmentKind::Lost, hud_assets.health_lost.clone()),
        (HudHealthSegmentKind::Empty, hud_assets.health_empty.clone()),
    ] {
        commands.spawn((
            Sprite {
                image,
                custom_size: Some(Vec2::ZERO),
                rect: Some(Rect::new(0.0, 0.0, 0.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 722.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::BaseTopLeft {
                top_left: Vec2::new(562.0, 213.0),
                size: Vec2::ZERO,
            },
            HudHealthSegment { segment },
            Name::new("hud_health"),
        ));
    }
}

fn spawn_hud_grenade_indicator(commands: &mut Commands, hud_assets: &HudAssets) {
    commands.spawn((
        Sprite {
            image: hud_grenade_icon(hud_assets, TeamType::Red),
            custom_size: Some(Vec2::ZERO),
            rect: Some(Rect::new(0.0, 0.0, 0.0, 0.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 723.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::BaseTopLeft {
            top_left: GRENADE_ICON_TOP_LEFT,
            size: Vec2::ZERO,
        },
        HudGrenadeIcon,
        Name::new("hud_grenade_icon"),
    ));

    commands.spawn((
        Text2d::new("00"),
        TextFont {
            font: hud_assets.font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Left),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(0.0, 0.0, 724.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::BaseTopLeft {
            top_left: GRENADE_TEXT_TOP_LEFT,
            size: Vec2::ZERO,
        },
        HudGrenadeText,
        Name::new("hud_grenade_text"),
    ));
}

fn spawn_hud_button(commands: &mut Commands, hud_assets: &HudAssets, spec: HudButtonSpec) {
    let Some(images) = hud_assets.buttons.get(spec.kind as usize) else {
        return;
    };
    let image = match spec.initial_state {
        HudButtonState::Active => images.active.clone(),
        HudButtonState::Inactive => images.inactive.clone(),
        HudButtonState::Pressed => images.pressed.clone(),
    };
    let anchor = if spec.fixed_x {
        HudAnchor::FixedXBaseY {
            top_left: spec.top_left,
            size: spec.size,
        }
    } else {
        HudAnchor::BaseTopLeft {
            top_left: spec.top_left,
            size: spec.size,
        }
    };

    commands.spawn((
        Sprite {
            image,
            custom_size: Some(spec.size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 720.0),
        RenderLayers::layer(HUD_LAYER),
        anchor,
        HudButton {
            kind: spec.kind,
            state: spec.initial_state,
        },
        Name::new(format!("hud_button_{:?}", spec.kind)),
    ));
}

pub(crate) fn hud_button_specs() -> [HudButtonSpec; 9] {
    const BUTTON: Vec2 = Vec2::new(24.0, 20.0);
    const MENU: Vec2 = Vec2::new(56.0, 20.0);

    [
        HudButtonSpec {
            kind: HudButtonKind::A,
            asset_name: "a_button",
            top_left: Vec2::new(556.0, 8.0),
            size: BUTTON,
            initial_state: HudButtonState::Inactive,
            fixed_x: false,
        },
        HudButtonSpec {
            kind: HudButtonKind::B,
            asset_name: "b_button",
            top_left: Vec2::new(68.0, 458.0),
            size: BUTTON,
            initial_state: HudButtonState::Inactive,
            fixed_x: true,
        },
        HudButtonSpec {
            kind: HudButtonKind::D,
            asset_name: "d_button",
            top_left: Vec2::new(586.0, 264.0),
            size: BUTTON,
            initial_state: HudButtonState::Active,
            fixed_x: false,
        },
        HudButtonSpec {
            kind: HudButtonKind::G,
            asset_name: "g_button",
            top_left: Vec2::new(98.0, 458.0),
            size: BUTTON,
            initial_state: HudButtonState::Inactive,
            fixed_x: true,
        },
        HudButtonSpec {
            kind: HudButtonKind::Menu,
            asset_name: "menu_button",
            top_left: Vec2::new(482.0, 458.0),
            size: MENU,
            initial_state: HudButtonState::Active,
            fixed_x: false,
        },
        HudButtonSpec {
            kind: HudButtonKind::R,
            asset_name: "r_button",
            top_left: Vec2::new(8.0, 458.0),
            size: BUTTON,
            initial_state: HudButtonState::Inactive,
            fixed_x: true,
        },
        HudButtonSpec {
            kind: HudButtonKind::T,
            asset_name: "t_button",
            top_left: Vec2::new(556.0, 264.0),
            size: BUTTON,
            initial_state: HudButtonState::Active,
            fixed_x: false,
        },
        HudButtonSpec {
            kind: HudButtonKind::V,
            asset_name: "v_button",
            top_left: Vec2::new(38.0, 458.0),
            size: BUTTON,
            initial_state: HudButtonState::Inactive,
            fixed_x: true,
        },
        HudButtonSpec {
            kind: HudButtonKind::Z,
            asset_name: "z_button",
            top_left: Vec2::new(616.0, 264.0),
            size: BUTTON,
            initial_state: HudButtonState::Active,
            fixed_x: false,
        },
    ]
}

impl HudLayout {
    pub(crate) fn for_map(map: &ZMap) -> Self {
        let map_pixel_size = Vec2::new(
            map.basics.width as f32 * TILE_SIZE,
            map.basics.height as f32 * TILE_SIZE,
        );
        let map_ratio = map.basics.width as f32 / map.basics.height as f32;
        let max_ratio = MINIMAP_MAX_W / MINIMAP_MAX_H;
        let mut render_size = if map_ratio < max_ratio {
            Vec2::new(map_ratio * MINIMAP_MAX_H, MINIMAP_MAX_H)
        } else {
            Vec2::new(MINIMAP_MAX_W, MINIMAP_MAX_W / map_ratio)
        };
        let mut render_offset = (Vec2::new(MINIMAP_MAX_W, MINIMAP_MAX_H) - render_size) * 0.5;

        render_offset += Vec2::splat(2.0);
        render_size -= Vec2::splat(4.0);
        render_size = render_size.max(Vec2::ZERO);

        Self {
            map_pixel_size,
            render_offset,
            render_size,
            render_ratio: render_size.y / map_pixel_size.y,
        }
    }

    pub(crate) fn map_pixel_to_minimap_local(self, map_pixel: Vec2) -> Vec2 {
        Vec2::new(
            self.render_offset.x + map_pixel.x * self.render_ratio,
            self.render_offset.y + map_pixel.y * self.render_ratio,
        )
    }

    pub(crate) fn bottom_right_offset_for_minimap_local(self, local: Vec2) -> Vec2 {
        let slot_top_left_from_bottom_right = Vec2::new(-93.0, -185.0);
        Vec2::new(
            slot_top_left_from_bottom_right.x + local.x,
            slot_top_left_from_bottom_right.y + local.y,
        )
    }

    pub(crate) fn minimap_screen_to_world(
        self,
        screen_pos: Vec2,
        window_size: Vec2,
    ) -> Option<Vec2> {
        let slot_top_left = window_size + Vec2::new(-93.0, -185.0);
        let local = screen_pos - slot_top_left;

        if local.x < self.render_offset.x
            || local.x > self.render_offset.x + self.render_size.x
            || local.y < self.render_offset.y
            || local.y > self.render_offset.y + self.render_size.y
        {
            return None;
        }

        let x_percent = (local.x - self.render_offset.x) / self.render_size.x;
        let y_percent = (local.y - self.render_offset.y) / self.render_size.y;
        let map_pixel = Vec2::new(
            x_percent * self.map_pixel_size.x,
            y_percent * self.map_pixel_size.y,
        );

        Some(Vec2::new(map_pixel.x, -map_pixel.y))
    }
}

pub(crate) fn minimap_zone_color(team: TeamType) -> Color {
    let linear = team.color().to_linear();
    Color::srgba(
        (linear.red * 0.4).clamp(0.0, 1.0),
        (linear.green * 0.4).clamp(0.0, 1.0),
        (linear.blue * 0.4).clamp(0.0, 1.0),
        0.55,
    )
}

pub(crate) fn handle_minimap_camera_focus(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    hud_layout: Res<HudLayout>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(screen_pos) = window.cursor_position() else {
        return;
    };
    let Some(world_pos) =
        hud_layout.minimap_screen_to_world(screen_pos, Vec2::new(window.width(), window.height()))
    else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation.x = world_pos.x;
    camera_transform.translation.y = world_pos.y;
}

pub(crate) fn handle_hud_button_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    hud_assets: Res<HudAssets>,
    attack_alert: Res<HudAttackAlert>,
    fort_warning: Res<FortUnderAttackWarning>,
    mut command_queue: ResMut<HudCommandQueue>,
    mut button_query: Query<(&mut HudButton, &mut Sprite)>,
) {
    if !mouse.just_pressed(MouseButton::Left) && !mouse.just_released(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());

    for (mut button, mut sprite) in &mut button_query {
        if mouse.just_pressed(MouseButton::Left) {
            if button.state == HudButtonState::Active
                && hud_button_contains(button.kind, cursor, window_size)
            {
                set_hud_button_state(
                    &hud_assets,
                    &mut sprite,
                    &mut button,
                    HudButtonState::Pressed,
                );
            }
        }

        if mouse.just_released(MouseButton::Left) && button.state == HudButtonState::Pressed {
            if hud_button_contains(button.kind, cursor, window_size) {
                if let Some(command) = hud_command_for_button(button.kind) {
                    command_queue.pending.push(command);
                }
            }
            set_hud_button_state(
                &hud_assets,
                &mut sprite,
                &mut button,
                HudButtonState::Active,
            );
        }

        if mouse.just_released(MouseButton::Left)
            && button.kind == HudButtonKind::A
            && hud_button_contains(button.kind, cursor, window_size)
        {
            if let Some(ref_id) = attack_alert.target_ref_id {
                command_queue.pending.push(HudCommand::JumpToObject(ref_id));
            }
        }
    }

    if mouse.just_released(MouseButton::Left)
        && computer_message_contains(cursor, window_size, fort_warning.message)
    {
        if let Some(ref_id) = fort_warning.message.target_ref_id {
            command_queue.pending.push(HudCommand::JumpToObject(ref_id));
        }
    }
}

pub(crate) fn hud_command_for_button(kind: HudButtonKind) -> Option<HudCommand> {
    match kind {
        HudButtonKind::B => Some(HudCommand::BeginBuildingAction),
        HudButtonKind::G => Some(HudCommand::SelectGroup(ObjectSelectionGroup::Cannon)),
        HudButtonKind::R => Some(HudCommand::SelectGroup(ObjectSelectionGroup::Robot)),
        HudButtonKind::V => Some(HudCommand::SelectGroup(ObjectSelectionGroup::Vehicle)),
        _ => None,
    }
}

fn set_hud_button_state(
    hud_assets: &HudAssets,
    sprite: &mut Sprite,
    button: &mut HudButton,
    state: HudButtonState,
) {
    let Some(images) = hud_assets.buttons.get(button.kind as usize) else {
        return;
    };
    sprite.image = match state {
        HudButtonState::Active => images.active.clone(),
        HudButtonState::Inactive => images.inactive.clone(),
        HudButtonState::Pressed => images.pressed.clone(),
    };
    button.state = state;
}

fn hud_button_contains(kind: HudButtonKind, cursor: Vec2, window_size: Vec2) -> bool {
    let Some(spec) = hud_button_spec(kind) else {
        return false;
    };
    let top_left = hud_button_screen_top_left(spec, window_size);

    cursor.x >= top_left.x
        && cursor.x <= top_left.x + spec.size.x
        && cursor.y >= top_left.y
        && cursor.y <= top_left.y + spec.size.y
}

pub(crate) fn hud_button_screen_top_left(spec: HudButtonSpec, window_size: Vec2) -> Vec2 {
    let base_top_left = window_size - Vec2::new(648.0, 484.0);
    if spec.fixed_x {
        Vec2::new(spec.top_left.x, base_top_left.y + spec.top_left.y)
    } else {
        base_top_left + spec.top_left
    }
}

pub(crate) fn hud_button_spec(kind: HudButtonKind) -> Option<HudButtonSpec> {
    hud_button_specs()
        .into_iter()
        .find(|spec| spec.kind == kind)
}

pub(crate) fn update_minimap_dots(
    layout: Res<HudLayout>,
    mut dot_query: Query<(&MinimapDot, &mut HudAnchor)>,
    object_query: Query<(&GameObjectEntity, &Transform)>,
) {
    for (dot, mut anchor) in &mut dot_query {
        let Some((_, object_transform)) = object_query
            .iter()
            .find(|(object, _)| object.ref_id == dot.ref_id)
        else {
            continue;
        };
        let world = object_transform.translation.truncate();
        let map_pixel = Vec2::new(world.x, -world.y);
        let local = layout.map_pixel_to_minimap_local(map_pixel);
        *anchor = HudAnchor::BottomRight {
            offset: layout.bottom_right_offset_for_minimap_local(local),
        };
    }
}

pub(crate) fn update_minimap_view_box(
    layout: Res<HudLayout>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut box_query: Query<(&mut HudAnchor, &mut Sprite), With<MinimapViewBox>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let Ok((mut anchor, mut sprite)) = box_query.single_mut() else {
        return;
    };

    let view_size = game_view_size(window);
    let top_left_map = Vec2::new(
        camera_transform.translation.x - view_size.x * 0.5,
        -camera_transform.translation.y - view_size.y * 0.5,
    )
    .clamp(Vec2::ZERO, layout.map_pixel_size);
    let center_map = top_left_map + view_size * 0.5;
    let local = layout.map_pixel_to_minimap_local(center_map);

    *anchor = HudAnchor::BottomRight {
        offset: layout.bottom_right_offset_for_minimap_local(local),
    };
    sprite.custom_size = Some((view_size * layout.render_ratio).max(Vec2::splat(1.0)));
}

pub(crate) fn update_hud_button_availability(
    hud_assets: Res<HudAssets>,
    object_query: Query<(&GameObjectEntity, &ObjectTeam, &ObjectStats)>,
    mut button_query: Query<(&mut HudButton, &mut Sprite)>,
) {
    let mut building_available = false;
    let mut cannon_available = false;
    let mut robot_available = false;
    let mut vehicle_available = false;

    for (object, team, stats) in &object_query {
        if team.0 != TeamType::Red || stats.destroyed() {
            continue;
        }

        match object.kind {
            ObjectKind::Building(_) => building_available = true,
            ObjectKind::Cannon(_) => cannon_available = true,
            ObjectKind::Robot(_) => robot_available = true,
            ObjectKind::Vehicle(_) => vehicle_available = true,
            _ => {}
        }
    }

    for (mut button, mut sprite) in &mut button_query {
        if button.state == HudButtonState::Pressed {
            continue;
        }

        let desired_state = match button.kind {
            HudButtonKind::B if building_available => HudButtonState::Active,
            HudButtonKind::G if cannon_available => HudButtonState::Active,
            HudButtonKind::R if robot_available => HudButtonState::Active,
            HudButtonKind::V if vehicle_available => HudButtonState::Active,
            HudButtonKind::B | HudButtonKind::G | HudButtonKind::R | HudButtonKind::V => {
                HudButtonState::Inactive
            }
            _ => button.state,
        };

        if desired_state != button.state {
            set_hud_button_state(&hud_assets, &mut sprite, &mut button, desired_state);
        }
    }
}

#[derive(Clone, Copy)]
struct AttackAlertSnapshot {
    ref_id: u32,
    team: TeamType,
    destroyed: bool,
    attack_target: Option<u32>,
}

pub(crate) fn update_hud_attack_alert(
    time: Res<Time>,
    hud_assets: Res<HudAssets>,
    mut alert: ResMut<HudAttackAlert>,
    object_query: Query<(
        &GameObjectEntity,
        &ObjectTeam,
        &ObjectStats,
        Option<&AttackTarget>,
    )>,
    mut button_query: Query<(&mut HudButton, &mut Sprite)>,
) {
    let snapshots: Vec<AttackAlertSnapshot> = object_query
        .iter()
        .map(|(object, team, stats, attack)| AttackAlertSnapshot {
            ref_id: object.ref_id,
            team: team.0,
            destroyed: stats.destroyed(),
            attack_target: attack.map(|attack| attack.ref_id),
        })
        .collect();

    if alert.target_ref_id.is_none() {
        if let Some(ref_id) = first_attack_alert_target(&snapshots, TeamType::Red) {
            start_attack_alert(&mut alert, ref_id);
        }
    }

    if let Some(ref_id) = alert.target_ref_id {
        if !object_exists(&snapshots, ref_id) {
            clear_attack_alert(&mut alert);
        } else {
            alert.check_elapsed += time.delta_secs();
            while alert.check_elapsed >= 0.25 {
                alert.check_elapsed -= 0.25;
                process_attack_alert_check(
                    &mut alert,
                    target_is_under_attack(&snapshots, ref_id, TeamType::Red),
                );
            }

            alert.flash_elapsed += time.delta_secs();
            while alert.flash_elapsed >= 0.15 {
                alert.flash_elapsed -= 0.15;
                alert.visible = !alert.visible;
            }
        }
    }

    for (mut button, mut sprite) in &mut button_query {
        if button.kind != HudButtonKind::A || button.state == HudButtonState::Pressed {
            continue;
        }

        let desired_state = if alert.target_ref_id.is_some() && alert.visible {
            HudButtonState::Active
        } else {
            HudButtonState::Inactive
        };
        if desired_state != button.state {
            set_hud_button_state(&hud_assets, &mut sprite, &mut button, desired_state);
        }
    }
}

fn start_attack_alert(alert: &mut HudAttackAlert, ref_id: u32) {
    alert.target_ref_id = Some(ref_id);
    alert.visible = true;
    alert.not_under_attack_checks = 0;
    alert.check_elapsed = 0.0;
    alert.flash_elapsed = 0.0;
}

fn clear_attack_alert(alert: &mut HudAttackAlert) {
    alert.target_ref_id = None;
    alert.visible = false;
    alert.not_under_attack_checks = 0;
    alert.check_elapsed = 0.0;
    alert.flash_elapsed = 0.0;
}

fn process_attack_alert_check(alert: &mut HudAttackAlert, still_under_attack: bool) {
    if still_under_attack {
        alert.not_under_attack_checks = 0;
        return;
    }

    if alert.not_under_attack_checks < 10 {
        alert.not_under_attack_checks += 1;
    } else {
        clear_attack_alert(alert);
    }
}

fn first_attack_alert_target(
    snapshots: &[AttackAlertSnapshot],
    player_team: TeamType,
) -> Option<u32> {
    snapshots
        .iter()
        .filter(|snapshot| snapshot.team == player_team && !snapshot.destroyed)
        .filter(|target| target_is_under_attack(snapshots, target.ref_id, player_team))
        .map(|target| target.ref_id)
        .min()
}

fn target_is_under_attack(
    snapshots: &[AttackAlertSnapshot],
    target_ref_id: u32,
    player_team: TeamType,
) -> bool {
    snapshots.iter().any(|attacker| {
        attacker.team != player_team
            && attacker.team != TeamType::Null
            && !attacker.destroyed
            && attacker.attack_target == Some(target_ref_id)
    })
}

fn object_exists(snapshots: &[AttackAlertSnapshot], ref_id: u32) -> bool {
    snapshots
        .iter()
        .any(|snapshot| snapshot.ref_id == ref_id && !snapshot.destroyed)
}

pub(crate) fn start_fort_under_attack_message(
    message: &mut ComputerMessageState,
    target_ref_id: u32,
) {
    *message = ComputerMessageState {
        target_ref_id: Some(target_ref_id),
        visible: true,
        blink_elapsed: 0.0,
        flips_remaining: COMPUTER_MESSAGE_BLINK_FLIPS,
        hold_remaining: COMPUTER_MESSAGE_HOLD_TIME,
    };
}

pub(crate) fn update_hud_computer_message(
    time: Res<Time>,
    mut fort_warning: ResMut<FortUnderAttackWarning>,
    mut message_query: Query<&mut Visibility, With<HudComputerMessage>>,
) {
    advance_computer_message_state(&mut fort_warning.message, time.delta_secs());
    let desired_visibility = if fort_warning.message.active() && fort_warning.message.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for mut visibility in &mut message_query {
        *visibility = desired_visibility;
    }
}

fn advance_computer_message_state(message: &mut ComputerMessageState, delta_secs: f32) {
    if !message.active() {
        return;
    }

    let mut remaining = delta_secs.max(0.0);
    while message.flips_remaining > 0 && remaining > 0.0 {
        let step = (COMPUTER_MESSAGE_BLINK_INTERVAL - message.blink_elapsed).min(remaining);
        message.blink_elapsed += step;
        remaining -= step;

        if message.blink_elapsed >= COMPUTER_MESSAGE_BLINK_INTERVAL {
            message.blink_elapsed = 0.0;
            message.flips_remaining -= 1;
            message.visible = !message.visible;
        }
    }

    if message.flips_remaining > 0 {
        return;
    }

    message.visible = true;
    message.hold_remaining -= remaining;
    if message.hold_remaining <= 0.0 {
        *message = ComputerMessageState::default();
    }
}

fn computer_message_contains(
    cursor: Vec2,
    window_size: Vec2,
    message: ComputerMessageState,
) -> bool {
    if !message.active() {
        return false;
    }

    let top_left = computer_message_screen_top_left(window_size);
    cursor.x >= top_left.x
        && cursor.x <= top_left.x + COMPUTER_MESSAGE_SIZE.x
        && cursor.y >= top_left.y
        && cursor.y <= top_left.y + COMPUTER_MESSAGE_SIZE.y
}

fn computer_message_screen_top_left(window_size: Vec2) -> Vec2 {
    Vec2::new(
        window_size.x * 0.5 - COMPUTER_MESSAGE_SIZE.x * 0.5,
        COMPUTER_MESSAGE_TOP_Y,
    )
}

pub(crate) fn update_hud_selected_object(
    asset_server: Res<AssetServer>,
    selection: Res<SelectionState>,
    object_query: Query<(&GameObjectEntity, &ObjectTeam)>,
    mut sprite_query: Query<(&HudSelectedObjectSprite, &mut HudAnchor, &mut Sprite)>,
) {
    let selected = selection.selected_refs.first().and_then(|selected_ref| {
        object_query
            .iter()
            .find(|(object, _)| object.ref_id == *selected_ref)
    });

    for (slot, mut anchor, mut sprite) in &mut sprite_query {
        let Some((object, team)) = selected else {
            sprite.custom_size = Some(Vec2::ZERO);
            sprite.rect = Some(Rect::new(0.0, 0.0, 0.0, 0.0));
            continue;
        };
        let Some(asset_name) = selected_hud_asset_name(object.kind, team.0, slot.slot) else {
            sprite.custom_size = Some(Vec2::ZERO);
            sprite.rect = Some(Rect::new(0.0, 0.0, 0.0, 0.0));
            continue;
        };

        let size = match slot.slot {
            HudSelectedObjectSlot::Icon => Vec2::new(96.0, 60.0),
            HudSelectedObjectSlot::Label => Vec2::new(96.0, 14.0),
        };
        let top_left = match slot.slot {
            HudSelectedObjectSlot::Icon => Vec2::new(550.0, 148.0),
            HudSelectedObjectSlot::Label => Vec2::new(550.0, 230.0),
        };

        sprite.image = asset_server.load(format!("other/hud/{asset_name}"));
        sprite.custom_size = Some(size);
        sprite.rect = None;
        *anchor = HudAnchor::BaseTopLeft { top_left, size };
    }
}

pub(crate) fn selected_hud_asset_name(
    kind: ObjectKind,
    team: TeamType,
    slot: HudSelectedObjectSlot,
) -> Option<String> {
    let name = match kind {
        ObjectKind::Robot(robot) => robot_hud_name(robot),
        ObjectKind::Vehicle(vehicle) => vehicle.folder(),
        ObjectKind::Cannon(cannon) => cannon_hud_name(cannon),
        _ => return None,
    };

    match slot {
        HudSelectedObjectSlot::Icon => Some(format!(
            "icon_{}_{}.png",
            name,
            team.atlas_team().asset_name()
        )),
        HudSelectedObjectSlot::Label => Some(format!("label_{name}.png")),
    }
}

pub(crate) fn update_hud_grenade_indicator(
    selection: Res<SelectionState>,
    object_query: Query<(&GameObjectEntity, &ObjectTeam, Option<&GrenadeInventory>)>,
    hud_assets: Res<HudAssets>,
    mut icon_query: Query<
        (&mut HudAnchor, &mut Sprite),
        (With<HudGrenadeIcon>, Without<HudGrenadeText>),
    >,
    mut text_query: Query<
        (&mut HudAnchor, &mut Text2d),
        (With<HudGrenadeText>, Without<HudGrenadeIcon>),
    >,
) {
    let selected = selection.selected_refs.first().and_then(|selected_ref| {
        object_query
            .iter()
            .find(|(object, _, _)| object.ref_id == *selected_ref)
    });

    let visible = selected.and_then(|(object, team, inventory)| {
        let inventory = inventory?;
        hud_grenade_indicator_for_selection(object.kind, team.0, inventory.amount)
    });

    if let Ok((mut anchor, mut sprite)) = icon_query.single_mut() {
        if let Some((team, _)) = visible {
            sprite.image = hud_grenade_icon(&hud_assets, team);
            sprite.custom_size = Some(GRENADE_ICON_SIZE);
            sprite.rect = None;
            *anchor = HudAnchor::BaseTopLeft {
                top_left: GRENADE_ICON_TOP_LEFT,
                size: GRENADE_ICON_SIZE,
            };
        } else {
            sprite.custom_size = Some(Vec2::ZERO);
            sprite.rect = Some(Rect::new(0.0, 0.0, 0.0, 0.0));
            *anchor = HudAnchor::BaseTopLeft {
                top_left: GRENADE_ICON_TOP_LEFT,
                size: Vec2::ZERO,
            };
        }
    }

    if let Ok((mut anchor, mut text)) = text_query.single_mut() {
        if let Some((_, amount)) = visible {
            text.0 = format_grenade_amount(amount);
            *anchor = HudAnchor::BaseTopLeft {
                top_left: GRENADE_TEXT_TOP_LEFT,
                size: Vec2::ZERO,
            };
        } else {
            text.0.clear();
            *anchor = HudAnchor::BaseTopLeft {
                top_left: GRENADE_TEXT_TOP_LEFT,
                size: Vec2::ZERO,
            };
        }
    }
}

fn hud_grenade_indicator_for_selection(
    kind: ObjectKind,
    team: TeamType,
    amount: u8,
) -> Option<(TeamType, u8)> {
    can_have_grenades(kind).then_some((team, amount))
}

fn format_grenade_amount(amount: u8) -> String {
    format!("{:02}", amount.min(99))
}

fn hud_grenade_icon(assets: &HudAssets, team: TeamType) -> Handle<Image> {
    let asset_team = if team == TeamType::Null {
        TeamType::Null
    } else {
        team.atlas_team()
    };
    assets
        .grenade_icons
        .iter()
        .find(|(icon_team, _)| *icon_team == asset_team)
        .map(|(_, image)| image.clone())
        .or_else(|| {
            assets
                .grenade_icons
                .iter()
                .find(|(icon_team, _)| *icon_team == TeamType::Null)
                .map(|(_, image)| image.clone())
        })
        .expect("grenade icon assets should include null fallback")
}

fn robot_hud_name(robot: RobotType) -> &'static str {
    match robot {
        RobotType::Grunt => "grunt",
        RobotType::Psycho => "psycho",
        RobotType::Sniper => "sniper",
        RobotType::Tough => "tough",
        RobotType::Pyro => "pyro",
        RobotType::Laser => "laser",
    }
}

fn cannon_hud_name(cannon: CannonType) -> &'static str {
    match cannon {
        CannonType::Gatling => "gatling",
        CannonType::Gun => "gun",
        CannonType::Howitzer => "howitzer",
        CannonType::MissileCannon => "missile_cannon",
    }
}

pub(crate) fn update_hud_health_bar(
    selection: Res<SelectionState>,
    object_query: Query<(&GameObjectEntity, &ObjectStats)>,
    mut segment_query: Query<(&HudHealthSegment, &mut HudAnchor, &mut Sprite)>,
) {
    let stats = selection.selected_refs.first().and_then(|selected| {
        object_query
            .iter()
            .find(|(object, _)| object.ref_id == *selected)
            .map(|(_, stats)| *stats)
    });

    let (full_w, lost_w, empty_w) = if let Some(stats) = stats {
        let full_w = (74.0 * stats.health / MAX_UNIT_HEALTH)
            .round()
            .max(if stats.health > 0.0 { 1.0 } else { 0.0 });
        let yellow_w = (74.0 * stats.max_health / MAX_UNIT_HEALTH)
            .round()
            .max(if stats.max_health > 0.0 { 1.0 } else { 0.0 });
        let lost_w = (yellow_w - full_w).max(0.0);
        let empty_w = (74.0 - yellow_w).max(0.0);
        (full_w, lost_w, empty_w)
    } else {
        (0.0, 0.0, 74.0)
    };

    for (segment, mut anchor, mut sprite) in &mut segment_query {
        let (x_offset, width) = match segment.segment {
            HudHealthSegmentKind::Full => (0.0, full_w),
            HudHealthSegmentKind::Lost => (full_w, lost_w),
            HudHealthSegmentKind::Empty => (full_w + lost_w, empty_w),
        };

        let top_left = Vec2::new(562.0 + x_offset, 213.0);
        let size = Vec2::new(width, 8.0);
        *anchor = HudAnchor::BaseTopLeft { top_left, size };
        sprite.custom_size = Some(size);
        sprite.rect = Some(Rect::new(x_offset, 0.0, x_offset + width, 8.0));
    }
}

pub(crate) fn update_hud_anchors(
    windows: Query<&Window>,
    mut sprite_query: Query<
        (&HudAnchor, &mut Transform, &mut Sprite),
        (With<Sprite>, Without<Text2d>),
    >,
    mut text_query: Query<(&HudAnchor, &mut Transform), (With<Text2d>, Without<Sprite>)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());

    for (anchor, mut transform, mut sprite) in &mut sprite_query {
        let screen_position = hud_anchor_screen_position(*anchor, window_size, Some(&mut sprite));
        let world_offset = hud_screen_to_world(screen_position, window_size);
        transform.translation.x = world_offset.x;
        transform.translation.y = world_offset.y;
    }

    for (anchor, mut transform) in &mut text_query {
        let screen_position = hud_anchor_screen_position(*anchor, window_size, None);
        let world_offset = hud_screen_to_world(screen_position, window_size);
        transform.translation.x = world_offset.x;
        transform.translation.y = world_offset.y;
    }
}

fn hud_anchor_screen_position(
    anchor: HudAnchor,
    window_size: Vec2,
    mut sprite: Option<&mut Sprite>,
) -> Vec2 {
    let base_top_left = window_size - Vec2::new(648.0, 484.0);
    match anchor {
        HudAnchor::SideFiller => {
            let height = (window_size.y - 484.0).max(1.0);
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(Vec2::new(HUD_WIDTH, height));
            }
            Vec2::new(window_size.x - HUD_WIDTH * 0.5, height * 0.5)
        }
        HudAnchor::SidePanel => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(Vec2::new(HUD_WIDTH, 484.0));
            }
            Vec2::new(window_size.x - HUD_WIDTH * 0.5, window_size.y - 484.0 * 0.5)
        }
        HudAnchor::BottomLeft => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(Vec2::new(206.0, HUD_HEIGHT));
            }
            Vec2::new(206.0 * 0.5, window_size.y - HUD_HEIGHT * 0.5)
        }
        HudAnchor::BottomCenter => {
            let right_edge = window_size.x - HUD_WIDTH - 130.0;
            let width = (right_edge - 206.0).max(1.0);
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(Vec2::new(width, HUD_HEIGHT));
            }
            Vec2::new(206.0 + width * 0.5, window_size.y - HUD_HEIGHT * 0.5)
        }
        HudAnchor::BottomRightCap => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(Vec2::new(130.0, HUD_HEIGHT));
            }
            Vec2::new(
                window_size.x - HUD_WIDTH - 130.0 * 0.5,
                window_size.y - HUD_HEIGHT * 0.5,
            )
        }
        HudAnchor::BaseTopLeft { top_left, size } => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(size);
            }
            base_top_left + top_left + size * 0.5
        }
        HudAnchor::FixedXBaseY { top_left, size } => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(size);
            }
            Vec2::new(
                top_left.x + size.x * 0.5,
                base_top_left.y + top_left.y + size.y * 0.5,
            )
        }
        HudAnchor::ScreenTopCenter { top_y, size } => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(size);
            }
            Vec2::new(window_size.x * 0.5, top_y + size.y * 0.5)
        }
        HudAnchor::BottomRight { offset } => window_size + offset,
    }
}

fn hud_screen_to_world(screen_position: Vec2, window_size: Vec2) -> Vec2 {
    Vec2::new(
        screen_position.x - window_size.x * 0.5,
        window_size.y * 0.5 - screen_position.y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alert_snapshot(
        ref_id: u32,
        team: TeamType,
        destroyed: bool,
        attack_target: Option<u32>,
    ) -> AttackAlertSnapshot {
        AttackAlertSnapshot {
            ref_id,
            team,
            destroyed,
            attack_target,
        }
    }

    #[test]
    fn grenade_indicator_visibility_matches_original_can_have_grenades() {
        assert_eq!(
            hud_grenade_indicator_for_selection(
                ObjectKind::Robot(RobotType::Grunt),
                TeamType::Red,
                0
            ),
            Some((TeamType::Red, 0))
        );
        assert_eq!(
            hud_grenade_indicator_for_selection(
                ObjectKind::Robot(RobotType::Pyro),
                TeamType::Blue,
                20
            ),
            Some((TeamType::Blue, 20))
        );
        assert_eq!(
            hud_grenade_indicator_for_selection(
                ObjectKind::Robot(RobotType::Tough),
                TeamType::Red,
                0
            ),
            None
        );
        assert_eq!(
            hud_grenade_indicator_for_selection(
                ObjectKind::Building(crate::original::objects::BuildingType::FortBack),
                TeamType::Red,
                0,
            ),
            None
        );
    }

    #[test]
    fn grenade_amount_text_matches_original_two_digits() {
        assert_eq!(format_grenade_amount(0), "00");
        assert_eq!(format_grenade_amount(7), "07");
        assert_eq!(format_grenade_amount(20), "20");
        assert_eq!(format_grenade_amount(99), "99");
        assert_eq!(format_grenade_amount(120), "99");
    }

    #[test]
    fn attack_alert_selects_owned_target_attacked_by_enemy() {
        let snapshots = [
            alert_snapshot(1, TeamType::Red, false, None),
            alert_snapshot(2, TeamType::Blue, false, Some(1)),
            alert_snapshot(3, TeamType::Red, false, None),
            alert_snapshot(4, TeamType::Blue, false, Some(3)),
        ];

        assert_eq!(
            first_attack_alert_target(&snapshots, TeamType::Red),
            Some(1)
        );
    }

    #[test]
    fn attack_alert_ignores_destroyed_and_non_owned_targets() {
        let snapshots = [
            alert_snapshot(1, TeamType::Red, true, None),
            alert_snapshot(2, TeamType::Blue, false, Some(1)),
            alert_snapshot(3, TeamType::Blue, false, None),
            alert_snapshot(4, TeamType::Red, false, Some(3)),
        ];

        assert_eq!(first_attack_alert_target(&snapshots, TeamType::Red), None);
    }

    #[test]
    fn attack_alert_check_counter_matches_original_reset_delay() {
        let mut alert = HudAttackAlert::default();
        start_attack_alert(&mut alert, 7);

        for expected in 1..=10 {
            process_attack_alert_check(&mut alert, false);
            assert_eq!(alert.target_ref_id, Some(7));
            assert_eq!(alert.not_under_attack_checks, expected);
        }

        process_attack_alert_check(&mut alert, false);
        assert_eq!(alert.target_ref_id, None);
        assert_eq!(alert.not_under_attack_checks, 0);
    }

    #[test]
    fn attack_alert_check_resets_counter_when_attack_continues() {
        let mut alert = HudAttackAlert {
            target_ref_id: Some(9),
            visible: true,
            not_under_attack_checks: 5,
            check_elapsed: 0.0,
            flash_elapsed: 0.0,
        };

        process_attack_alert_check(&mut alert, true);

        assert_eq!(alert.target_ref_id, Some(9));
        assert_eq!(alert.not_under_attack_checks, 0);
    }

    #[test]
    fn fort_message_blinks_ten_flips_then_holds_visible() {
        let mut message = ComputerMessageState::default();
        start_fort_under_attack_message(&mut message, 42);

        assert_eq!(message.target_ref_id, Some(42));
        assert!(message.visible);
        assert_eq!(message.flips_remaining, 10);

        advance_computer_message_state(&mut message, 0.3);
        assert!(message.active());
        assert!(!message.visible);
        assert_eq!(message.flips_remaining, 9);

        advance_computer_message_state(&mut message, 2.7);
        assert!(message.active());
        assert!(message.visible);
        assert_eq!(message.flips_remaining, 0);

        advance_computer_message_state(&mut message, 4.99);
        assert!(message.active());
        assert!(message.visible);

        advance_computer_message_state(&mut message, 0.02);
        assert!(!message.active());
        assert!(!message.visible);
    }

    #[test]
    fn fort_message_click_rect_uses_original_top_y() {
        let mut message = ComputerMessageState::default();
        start_fort_under_attack_message(&mut message, 42);
        let window_size = Vec2::new(800.0, 600.0);

        assert_eq!(
            computer_message_screen_top_left(window_size),
            Vec2::new(336.0, 20.0)
        );
        assert!(computer_message_contains(
            Vec2::new(400.0, 27.0),
            window_size,
            message
        ));
        assert!(!computer_message_contains(
            Vec2::new(400.0, 19.0),
            window_size,
            message
        ));
    }
}
