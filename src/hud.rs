use bevy::{camera::visibility::RenderLayers, prelude::*};

use crate::{
    account::LoginPromptState,
    account_ui::{AccountMenuState, account_menu_contains_cursor},
    camera::game_view_size,
    chat::ChatInputState,
    components::*,
    constants::*,
    grenades::can_have_grenades,
    news::{NEWS_MAX_HISTORY, NewsLog},
    original::{
        map::ZMap,
        objects::{BuildingType, ObjectKind},
        types::TeamType,
    },
    portrait::{PortraitAssets, spawn_hud_portrait},
    selectable_maps::SelectableMapListState,
    units,
    units::buildings::production_logic::MAX_STORED_CANNONS,
    units::unit_stats::MAX_UNIT_HEALTH,
    vote::{GameVoteState, LocalVotePlayers, VoteDisplaySnapshot, vote_display_snapshot},
};

const GRENADE_ICON_TOP_LEFT: Vec2 = Vec2::new(575.0, 185.0);
const GRENADE_ICON_SIZE: Vec2 = Vec2::new(24.0, 20.0);
const GRENADE_TEXT_TOP_LEFT: Vec2 = Vec2::new(600.0, 187.0);
const COMPUTER_MESSAGE_TOP_Y: f32 = 20.0;
const COMPUTER_MESSAGE_SIZE: Vec2 = Vec2::new(128.0, 14.0);
const RESUME_PROMPT_SIZE: Vec2 = Vec2::new(170.0, 14.0);
const COMPUTER_MESSAGE_BLINK_INTERVAL: f32 = 0.3;
const COMPUTER_MESSAGE_BLINK_FLIPS: u8 = 10;
const COMPUTER_MESSAGE_HOLD_TIME: f32 = 5.0;
const MAX_RENDERABLE_STORED_GUNS: usize = 8;
const STORED_GUN_ICON_TOP_LEFT: Vec2 = Vec2::new(8.0, 8.0);
const STORED_GUN_ICON_SIZE: Vec2 = Vec2::new(16.0, 14.0);
const STORED_GUN_ROW_GAP: f32 = 2.0;
const STORED_GUN_MULTIPLIER_OFFSET: Vec2 = Vec2::new(20.0, 3.0);
const VOTE_PANEL_TOP_RIGHT: Vec2 = Vec2::new(4.0, 4.0);
const VOTE_PANEL_SIZE: Vec2 = Vec2::new(112.0, 73.0);
const VOTE_PANEL_ALPHA: f32 = 200.0 / 255.0;
const VOTE_TEXT_FONT_SIZE: f32 = 10.0;
const VOTE_DESCRIPTION_OFFSET: Vec2 = Vec2::new(57.0, 41.0);
const VOTE_HAVE_OFFSET: Vec2 = Vec2::new(57.0, 53.0);
const VOTE_NEEDED_OFFSET: Vec2 = Vec2::new(57.0, 64.0);
const VOTE_FOR_OFFSET: Vec2 = Vec2::new(22.0, 64.0);
const VOTE_AGAINST_OFFSET: Vec2 = Vec2::new(91.0, 64.0);
const NEWS_TEXT_LEFT: f32 = 5.0;
const NEWS_TEXT_FIRST_BOTTOM: f32 = 51.0;
const NEWS_TEXT_ROW_GAP: f32 = 15.0;
const NEWS_TEXT_FONT_SIZE: f32 = 12.0;
const CHAT_TEXT_LEFT: f32 = 209.0;
const CHAT_TEXT_BOTTOM: f32 = 19.0;
const CHAT_TEXT_FONT_SIZE: f32 = 12.0;
const CHAT_TEXT_MAX_CHARS: usize = 64;
const ATTACK_ALERT_REPEAT_BASE_DELAY: f32 = 5.0;
const ATTACK_ALERT_REPEAT_RANDOM_STEPS: usize = 300;
const ATTACK_ALERT_REPEAT_RANDOM_STEP: f32 = 0.01;
const ATTACK_ALERT_REPEAT_VARIANTS: usize = 6;

pub(crate) fn spawn_hud(
    commands: &mut Commands,
    map: &ZMap,
    hud_assets: &HudAssets,
    portrait_assets: &PortraitAssets,
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
    spawn_hud_resume_prompt(commands, hud_assets);
    spawn_hud_vote_panel(commands, hud_assets);
    spawn_hud_news_log(commands, hud_assets);
    spawn_hud_chat_draft(commands, hud_assets);
    spawn_hud_stored_gun_indicators(commands, hud_assets);
    spawn_hud_portrait(commands, portrait_assets);

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
        Name::new("hud_computer_message"),
    ));
}

fn spawn_hud_resume_prompt(commands: &mut Commands, hud_assets: &HudAssets) {
    commands.spawn((
        Sprite {
            image: hud_assets.click_to_resume_message.clone(),
            custom_size: Some(RESUME_PROMPT_SIZE),
            ..default()
        },
        Visibility::Hidden,
        Transform::from_xyz(0.0, 0.0, 732.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::ScreenCenter {
            size: RESUME_PROMPT_SIZE,
        },
        HudResumePrompt,
        Name::new("hud_resume_prompt"),
    ));
}

fn spawn_hud_vote_panel(commands: &mut Commands, hud_assets: &HudAssets) {
    commands.spawn((
        Sprite {
            image: hud_assets.vote_in_progress_panel.clone(),
            color: Color::srgba(1.0, 1.0, 1.0, VOTE_PANEL_ALPHA),
            custom_size: Some(VOTE_PANEL_SIZE),
            ..default()
        },
        Visibility::Hidden,
        Transform::from_xyz(0.0, 0.0, 733.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::ScreenTopRight {
            top_right: VOTE_PANEL_TOP_RIGHT,
            size: VOTE_PANEL_SIZE,
        },
        HudVotePanel,
        Name::new("hud_vote_panel"),
    ));

    for (field, offset) in [
        (HudVoteTextField::Description, VOTE_DESCRIPTION_OFFSET),
        (HudVoteTextField::Have, VOTE_HAVE_OFFSET),
        (HudVoteTextField::Needed, VOTE_NEEDED_OFFSET),
        (HudVoteTextField::ForVotes, VOTE_FOR_OFFSET),
        (HudVoteTextField::AgainstVotes, VOTE_AGAINST_OFFSET),
    ] {
        commands.spawn((
            Text2d::new(""),
            TextFont {
                font: hud_assets.font.clone(),
                font_size: VOTE_TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(vote_text_color()),
            TextLayout::new_with_justify(Justify::Left),
            bevy::sprite::Anchor::TOP_LEFT,
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, 734.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::ScreenTopRight {
                top_right: vote_text_top_right(offset),
                size: Vec2::ZERO,
            },
            HudVoteText { field },
            Name::new("hud_vote_text"),
        ));
    }
}

fn spawn_hud_news_log(commands: &mut Commands, hud_assets: &HudAssets) {
    for slot in 0..NEWS_MAX_HISTORY {
        commands.spawn((
            Text2d::new(""),
            TextFont {
                font: hud_assets.font.clone(),
                font_size: NEWS_TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(Justify::Left),
            bevy::sprite::Anchor::TOP_LEFT,
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, 735.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::ScreenBottomLeft {
                bottom_left: news_text_bottom_left(slot),
                size: Vec2::ZERO,
            },
            HudNewsText { slot },
            Name::new("hud_news_text"),
        ));
    }
}

fn spawn_hud_chat_draft(commands: &mut Commands, hud_assets: &HudAssets) {
    commands.spawn((
        Text2d::new(""),
        TextFont {
            font: hud_assets.font.clone(),
            font_size: CHAT_TEXT_FONT_SIZE,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Left),
        bevy::sprite::Anchor::TOP_LEFT,
        Visibility::Hidden,
        Transform::from_xyz(0.0, 0.0, 736.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::ScreenBottomLeft {
            bottom_left: chat_text_bottom_left(),
            size: Vec2::ZERO,
        },
        HudChatText,
        Name::new("hud_chat_text"),
    ));
}

fn spawn_hud_stored_gun_indicators(commands: &mut Commands, hud_assets: &HudAssets) {
    for slot in 0..MAX_RENDERABLE_STORED_GUNS {
        let icon_top_left = stored_gun_icon_top_left(slot);
        commands.spawn((
            Sprite {
                image: hud_assets.stored_gun_indicator.clone(),
                custom_size: Some(STORED_GUN_ICON_SIZE),
                ..default()
            },
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, 731.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::ScreenTopLeft {
                top_left: icon_top_left,
                size: STORED_GUN_ICON_SIZE,
            },
            HudStoredGunIcon { slot },
            Name::new("hud_stored_gun_icon"),
        ));

        commands.spawn((
            Text2d::new(""),
            TextFont {
                font: hud_assets.font.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(Justify::Left),
            bevy::sprite::Anchor::TOP_LEFT,
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, 732.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::ScreenTopLeft {
                top_left: icon_top_left + STORED_GUN_MULTIPLIER_OFFSET,
                size: Vec2::ZERO,
            },
            HudStoredGunMultiplier { slot },
            Name::new("hud_stored_gun_multiplier"),
        ));
    }
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
    computer_display: Res<ComputerMessageDisplay>,
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
        && computer_message_contains(cursor, window_size, computer_display.message)
    {
        if let Some(command) = computer_message_focus_command(computer_display.message) {
            command_queue.pending.push(command);
        }
    }
}

pub(crate) fn handle_resume_prompt_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    pause: Res<GamePauseState>,
    mut click_state: ResMut<ResumePromptClickState>,
    mut command_queue: ResMut<HudCommandQueue>,
    mut production_window: ResMut<ProductionWindowState>,
    login_prompt: Res<LoginPromptState>,
    account_menu: Res<AccountMenuState>,
) {
    if !mouse.just_pressed(MouseButton::Left) && !mouse.just_released(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        click_state.pressed = false;
        return;
    };
    if account_menu_contains_cursor(&login_prompt, &account_menu, window, cursor) {
        click_state.pressed = false;
        return;
    }

    let window_size = Vec2::new(window.width(), window.height());
    let hovered = resume_prompt_contains(cursor, window_size, pause.paused);

    if mouse.just_pressed(MouseButton::Left) {
        click_state.pressed = hovered;
        if hovered {
            production_window.input_captured = true;
        }
    }

    if mouse.just_released(MouseButton::Left) {
        if click_state.pressed && hovered {
            command_queue.pending.push(HudCommand::ResumeGame);
            production_window.input_captured = true;
        }
        click_state.pressed = false;
    }
}

pub(crate) fn handle_stored_gun_hud_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut click_state: ResMut<StoredGunHudClickState>,
    mut command_queue: ResMut<HudCommandQueue>,
    mut production_window: ResMut<ProductionWindowState>,
    object_query: Query<(
        &GameObjectEntity,
        &ObjectTeam,
        &ObjectStats,
        &BuildingProduction,
    )>,
) {
    if !mouse.just_pressed(MouseButton::Left) && !mouse.just_released(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        click_state.pressed_ref_id = None;
        return;
    };

    let entries = stored_gun_hud_entries_from_query(&object_query, TeamType::Red);
    let hovered_ref_id = stored_gun_ref_at_cursor(cursor, &entries);

    if mouse.just_pressed(MouseButton::Left) {
        click_state.pressed_ref_id = hovered_ref_id;
        if hovered_ref_id.is_some() {
            production_window.input_captured = true;
        }
    }

    if mouse.just_released(MouseButton::Left) {
        if hovered_ref_id.is_some() || click_state.pressed_ref_id.is_some() {
            production_window.input_captured = true;
        }
        if let (Some(pressed_ref_id), Some(released_ref_id)) =
            (click_state.pressed_ref_id, hovered_ref_id)
        {
            if pressed_ref_id == released_ref_id {
                command_queue.pending.push(HudCommand::FocusObject {
                    ref_id: released_ref_id,
                    select_obj: false,
                    open_gui: true,
                });
            }
        }
        click_state.pressed_ref_id = None;
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
    destroyed: bool,
    attack_target: Option<u32>,
}

pub(crate) fn update_hud_attack_alert(
    time: Res<Time>,
    hud_assets: Res<HudAssets>,
    mut alert: ResMut<HudAttackAlert>,
    mut portrait_state: ResMut<PortraitAnimationState>,
    mut portrait_sounds: ResMut<PortraitAnimationSoundQueue>,
    mut rng: ResMut<CombatRng>,
    object_query: Query<(&GameObjectEntity, &ObjectStats, Option<&AttackTarget>)>,
    mut button_query: Query<(&mut HudButton, &mut Sprite)>,
) {
    let snapshots: Vec<AttackAlertSnapshot> = object_query
        .iter()
        .map(|(object, stats, attack)| AttackAlertSnapshot {
            ref_id: object.ref_id,
            destroyed: stats.destroyed(),
            attack_target: attack.map(|attack| attack.ref_id),
        })
        .collect();

    if let Some(ref_id) = alert.target_ref_id {
        if !object_exists(&snapshots, ref_id) {
            clear_attack_alert(&mut alert);
        } else {
            alert.check_elapsed += time.delta_secs();
            while alert.check_elapsed >= 0.25 {
                alert.check_elapsed -= 0.25;
                process_attack_alert_check(&mut alert, target_is_under_attack(&snapshots, ref_id));
            }

            if alert.target_ref_id == Some(ref_id) {
                alert.flash_elapsed += time.delta_secs();
                while alert.flash_elapsed >= 0.15 {
                    alert.flash_elapsed -= 0.15;
                    alert.visible = !alert.visible;
                }

                process_attack_alert_repeat_animation(
                    &mut alert,
                    ref_id,
                    time.delta_secs(),
                    &mut portrait_state,
                    &mut portrait_sounds,
                    &mut rng,
                );
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

fn clear_attack_alert(alert: &mut HudAttackAlert) {
    alert.source_clear();
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

fn process_attack_alert_repeat_animation(
    alert: &mut HudAttackAlert,
    target_ref_id: u32,
    delta_secs: f32,
    portrait_state: &mut PortraitAnimationState,
    portrait_sounds: &mut PortraitAnimationSoundQueue,
    rng: &mut CombatRng,
) -> bool {
    alert.anim_elapsed += delta_secs.max(0.0);
    if alert.anim_elapsed < alert.anim_delay {
        return false;
    }

    let next_delay = attack_alert_next_repeat_delay(rng);
    alert.source_schedule_next_anim(next_delay);
    let next_anim = attack_alert_next_repeat_anim(alert.last_anim, rng);
    if portrait_state.doing_anim() {
        return false;
    }

    alert.last_anim = Some(next_anim);
    let kind = PortraitAnimationKind::UnderAttackRepeat(next_anim);
    portrait_state.start(PortraitAnimationEvent {
        ref_id: target_ref_id,
        kind,
    });
    portrait_sounds.pending.push(kind);
    true
}

pub(crate) fn attack_alert_next_repeat_delay(rng: &mut CombatRng) -> f32 {
    ATTACK_ALERT_REPEAT_BASE_DELAY
        + ATTACK_ALERT_REPEAT_RANDOM_STEP * rng.index(ATTACK_ALERT_REPEAT_RANDOM_STEPS) as f32
}

fn attack_alert_next_repeat_anim(last_anim: Option<u8>, rng: &mut CombatRng) -> u8 {
    let mut next_anim = rng.index(ATTACK_ALERT_REPEAT_VARIANTS) as u8;
    while Some(next_anim) == last_anim {
        next_anim = rng.index(ATTACK_ALERT_REPEAT_VARIANTS) as u8;
    }
    next_anim
}

fn target_is_under_attack(snapshots: &[AttackAlertSnapshot], target_ref_id: u32) -> bool {
    snapshots
        .iter()
        .any(|attacker| !attacker.destroyed && attacker.attack_target == Some(target_ref_id))
}

fn object_exists(snapshots: &[AttackAlertSnapshot], ref_id: u32) -> bool {
    snapshots
        .iter()
        .any(|snapshot| snapshot.ref_id == ref_id && !snapshot.destroyed)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StoredGunHudSnapshot {
    ref_id: u32,
    kind: ObjectKind,
    team: TeamType,
    destroyed: bool,
    stored_cannon_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StoredGunHudEntry {
    ref_id: u32,
    stored_cannon_count: usize,
}

pub(crate) fn update_hud_stored_guns(
    object_query: Query<(
        &GameObjectEntity,
        &ObjectTeam,
        &ObjectStats,
        &BuildingProduction,
    )>,
    mut icon_query: Query<
        (&HudStoredGunIcon, &mut HudAnchor, &mut Visibility),
        Without<HudStoredGunMultiplier>,
    >,
    mut text_query: Query<
        (
            &HudStoredGunMultiplier,
            &mut HudAnchor,
            &mut Text2d,
            &mut Visibility,
        ),
        Without<HudStoredGunIcon>,
    >,
) {
    let entries = stored_gun_hud_entries_from_query(&object_query, TeamType::Red);

    for (icon, mut anchor, mut visibility) in &mut icon_query {
        let top_left = stored_gun_icon_top_left(icon.slot);
        *anchor = HudAnchor::ScreenTopLeft {
            top_left,
            size: STORED_GUN_ICON_SIZE,
        };
        *visibility = if entries.get(icon.slot).is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (multiplier, mut anchor, mut text, mut visibility) in &mut text_query {
        let top_left = stored_gun_icon_top_left(multiplier.slot) + STORED_GUN_MULTIPLIER_OFFSET;
        *anchor = HudAnchor::ScreenTopLeft {
            top_left,
            size: Vec2::ZERO,
        };

        let Some(entry) = entries.get(multiplier.slot) else {
            text.0.clear();
            *visibility = Visibility::Hidden;
            continue;
        };

        if let Some(multiplier_text) = stored_gun_multiplier_text(entry.stored_cannon_count) {
            text.0 = multiplier_text;
            *visibility = Visibility::Visible;
        } else {
            text.0.clear();
            *visibility = Visibility::Hidden;
        }
    }
}

fn stored_gun_hud_entries_from_query(
    object_query: &Query<(
        &GameObjectEntity,
        &ObjectTeam,
        &ObjectStats,
        &BuildingProduction,
    )>,
    player_team: TeamType,
) -> Vec<StoredGunHudEntry> {
    let mut snapshots: Vec<_> = object_query
        .iter()
        .map(|(object, team, stats, production)| StoredGunHudSnapshot {
            ref_id: object.ref_id,
            kind: object.kind,
            team: team.0,
            destroyed: stats.destroyed(),
            stored_cannon_count: production.stored_cannons.len(),
        })
        .collect();
    snapshots.sort_by_key(|snapshot| snapshot.ref_id);
    stored_gun_hud_entries(&snapshots, player_team)
}

fn stored_gun_hud_entries(
    snapshots: &[StoredGunHudSnapshot],
    player_team: TeamType,
) -> Vec<StoredGunHudEntry> {
    snapshots
        .iter()
        .filter(|snapshot| {
            snapshot.team == player_team
                && !snapshot.destroyed
                && snapshot.stored_cannon_count > 0
                && stored_gun_hud_building(snapshot.kind)
        })
        .take(MAX_RENDERABLE_STORED_GUNS)
        .map(|snapshot| StoredGunHudEntry {
            ref_id: snapshot.ref_id,
            stored_cannon_count: snapshot.stored_cannon_count,
        })
        .collect()
}

fn stored_gun_hud_building(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Building(
            BuildingType::FortFront
                | BuildingType::FortBack
                | BuildingType::RobotFactory
                | BuildingType::VehicleFactory
        )
    )
}

fn stored_gun_ref_at_cursor(cursor: Vec2, entries: &[StoredGunHudEntry]) -> Option<u32> {
    entries
        .iter()
        .enumerate()
        .find_map(|(slot, entry)| stored_gun_slot_contains(slot, cursor).then_some(entry.ref_id))
}

fn stored_gun_slot_contains(slot: usize, cursor: Vec2) -> bool {
    let top_left = stored_gun_icon_top_left(slot);
    cursor.x >= top_left.x
        && cursor.x <= top_left.x + STORED_GUN_ICON_SIZE.x
        && cursor.y >= top_left.y
        && cursor.y <= top_left.y + STORED_GUN_ICON_SIZE.y
}

fn stored_gun_multiplier_text(stored_cannon_count: usize) -> Option<String> {
    (stored_cannon_count > 1 && stored_cannon_count <= MAX_STORED_CANNONS)
        .then(|| format!("X{stored_cannon_count}"))
}

fn stored_gun_icon_top_left(slot: usize) -> Vec2 {
    Vec2::new(
        STORED_GUN_ICON_TOP_LEFT.x,
        STORED_GUN_ICON_TOP_LEFT.y + slot as f32 * (STORED_GUN_ICON_SIZE.y + STORED_GUN_ROW_GAP),
    )
}

pub(crate) fn start_computer_message(
    message: &mut ComputerMessageState,
    kind: ComputerMessageKind,
    target_ref_id: u32,
) {
    *message = ComputerMessageState {
        kind: Some(kind),
        target_ref_id: Some(target_ref_id),
        visible: true,
        blink_elapsed: 0.0,
        flips_remaining: COMPUTER_MESSAGE_BLINK_FLIPS,
        hold_remaining: COMPUTER_MESSAGE_HOLD_TIME,
    };
}

pub(crate) fn start_fort_under_attack_message(
    message: &mut ComputerMessageState,
    target_ref_id: u32,
) {
    start_computer_message(message, ComputerMessageKind::FortUnderAttack, target_ref_id);
}

pub(crate) fn computer_message_space_bar_event(
    kind: ComputerMessageKind,
    ref_id: u32,
) -> SpaceBarEvent {
    let (select_obj, open_gui) = computer_message_focus_flags(kind);
    SpaceBarEvent::new(ref_id, select_obj, open_gui)
}

pub(crate) fn update_hud_computer_message(
    time: Res<Time>,
    hud_assets: Res<HudAssets>,
    mut display: ResMut<ComputerMessageDisplay>,
    mut message_query: Query<(&mut Visibility, &mut Sprite), With<HudComputerMessage>>,
) {
    advance_computer_message_state(&mut display.message, time.delta_secs());
    let desired_visibility = if display.message.active() && display.message.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for (mut visibility, mut sprite) in &mut message_query {
        *visibility = desired_visibility;
        if let Some(kind) = display.message.kind {
            sprite.image = computer_message_image(&hud_assets, kind);
        }
    }
}

pub(crate) fn update_hud_resume_prompt(
    pause: Res<GamePauseState>,
    mut prompt_query: Query<&mut Visibility, With<HudResumePrompt>>,
) {
    let desired_visibility = if pause.paused {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for mut visibility in &mut prompt_query {
        *visibility = desired_visibility;
    }
}

pub(crate) fn update_hud_vote_display(
    vote: Res<GameVoteState>,
    vote_players: Res<LocalVotePlayers>,
    selectable_maps: Res<SelectableMapListState>,
    mut panel_query: Query<&mut Visibility, (With<HudVotePanel>, Without<HudVoteText>)>,
    mut text_query: Query<(&HudVoteText, &mut Text2d, &mut Visibility), Without<HudVotePanel>>,
) {
    let snapshot = vote_display_snapshot(&vote, &vote_players, selectable_maps.maps());
    let desired_visibility = if snapshot.is_some() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for mut visibility in &mut panel_query {
        *visibility = desired_visibility;
    }

    for (vote_text, mut text, mut visibility) in &mut text_query {
        if let Some(snapshot) = snapshot.as_ref() {
            text.0 = vote_text_value(vote_text.field, snapshot);
            *visibility = Visibility::Visible;
        } else {
            text.0.clear();
            *visibility = Visibility::Hidden;
        }
    }
}

pub(crate) fn update_hud_news_log(
    time: Res<Time<Real>>,
    mut news_log: ResMut<NewsLog>,
    mut text_query: Query<(
        &HudNewsText,
        &mut Text2d,
        &mut TextColor,
        &mut Visibility,
        &mut HudAnchor,
    )>,
) {
    news_log.advance(time.delta_secs());

    for (news_text, mut text, mut text_color, mut visibility, mut anchor) in &mut text_query {
        if let Some(entry) = news_log.display_entry(news_text.slot) {
            text.0 = entry.message.to_string();
            text_color.0 = entry.color.to_bevy(entry.alpha);
            *visibility = Visibility::Visible;
            *anchor = HudAnchor::ScreenBottomLeft {
                bottom_left: news_text_bottom_left(news_text.slot),
                size: Vec2::ZERO,
            };
        } else {
            text.0.clear();
            *visibility = Visibility::Hidden;
        }
    }
}

pub(crate) fn update_hud_chat_draft(
    chat: Res<ChatInputState>,
    mut text_query: Query<
        (&mut Text2d, &mut Visibility, &mut HudAnchor),
        (With<HudChatText>, Without<HudNewsText>),
    >,
) {
    for (mut text, mut visibility, mut anchor) in &mut text_query {
        if chat.collecting() {
            text.0 = chat_draft_text(chat.message());
            *visibility = Visibility::Visible;
            *anchor = HudAnchor::ScreenBottomLeft {
                bottom_left: chat_text_bottom_left(),
                size: Vec2::ZERO,
            };
        } else {
            text.0.clear();
            *visibility = Visibility::Hidden;
        }
    }
}

fn computer_message_image(hud_assets: &HudAssets, kind: ComputerMessageKind) -> Handle<Image> {
    match kind {
        ComputerMessageKind::RobotManufactured => hud_assets.robot_manufactured_message.clone(),
        ComputerMessageKind::VehicleManufactured => hud_assets.vehicle_manufactured_message.clone(),
        ComputerMessageKind::GunManufactured => hud_assets.gun_manufactured_message.clone(),
        ComputerMessageKind::FortUnderAttack => hud_assets.fort_under_attack_message.clone(),
    }
}

fn vote_text_value(field: HudVoteTextField, snapshot: &VoteDisplaySnapshot) -> String {
    match field {
        HudVoteTextField::Description => snapshot.description.clone(),
        HudVoteTextField::Have => snapshot.have_votes.to_string(),
        HudVoteTextField::Needed => snapshot.needed_votes.to_string(),
        HudVoteTextField::ForVotes => snapshot.for_votes.to_string(),
        HudVoteTextField::AgainstVotes => snapshot.against_votes.to_string(),
    }
}

fn vote_text_top_right(offset: Vec2) -> Vec2 {
    Vec2::new(
        VOTE_PANEL_TOP_RIGHT.x + VOTE_PANEL_SIZE.x - offset.x,
        VOTE_PANEL_TOP_RIGHT.y + offset.y,
    )
}

fn vote_text_color() -> Color {
    Color::srgba(1.0, 0.92, 0.2, VOTE_PANEL_ALPHA)
}

fn news_text_bottom_left(slot: usize) -> Vec2 {
    Vec2::new(
        NEWS_TEXT_LEFT,
        NEWS_TEXT_FIRST_BOTTOM + NEWS_TEXT_ROW_GAP * slot as f32,
    )
}

fn chat_text_bottom_left() -> Vec2 {
    Vec2::new(CHAT_TEXT_LEFT, CHAT_TEXT_BOTTOM)
}

fn chat_draft_text(message: &str) -> String {
    let draft = format!("Say:: {message}");
    let char_count = draft.chars().count();
    if char_count <= CHAT_TEXT_MAX_CHARS {
        draft
    } else {
        draft
            .chars()
            .skip(char_count - CHAT_TEXT_MAX_CHARS)
            .collect()
    }
}

fn computer_message_focus_command(message: ComputerMessageState) -> Option<HudCommand> {
    let ref_id = message.target_ref_id?;
    let kind = message.kind?;
    let (select_obj, open_gui) = computer_message_focus_flags(kind);
    Some(HudCommand::FocusObject {
        ref_id,
        select_obj,
        open_gui,
    })
}

fn computer_message_focus_flags(kind: ComputerMessageKind) -> (bool, bool) {
    match kind {
        ComputerMessageKind::RobotManufactured | ComputerMessageKind::VehicleManufactured => {
            (true, false)
        }
        ComputerMessageKind::GunManufactured => (false, true),
        ComputerMessageKind::FortUnderAttack => (false, false),
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

fn resume_prompt_contains(cursor: Vec2, window_size: Vec2, paused: bool) -> bool {
    if !paused {
        return false;
    }

    let top_left = resume_prompt_screen_top_left(window_size);
    cursor.x >= top_left.x
        && cursor.x <= top_left.x + RESUME_PROMPT_SIZE.x
        && cursor.y >= top_left.y
        && cursor.y <= top_left.y + RESUME_PROMPT_SIZE.y
}

fn resume_prompt_screen_top_left(window_size: Vec2) -> Vec2 {
    (window_size - RESUME_PROMPT_SIZE) * 0.5
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
    match slot {
        HudSelectedObjectSlot::Icon => units::selected_hud_icon_asset_name(kind, team),
        HudSelectedObjectSlot::Label => units::selected_hud_label_asset_name(kind),
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
        HudAnchor::BasePoint { point } => base_top_left + point,
        HudAnchor::FixedXBaseY { top_left, size } => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(size);
            }
            Vec2::new(
                top_left.x + size.x * 0.5,
                base_top_left.y + top_left.y + size.y * 0.5,
            )
        }
        HudAnchor::ScreenTopLeft { top_left, size } => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(size);
            }
            top_left + size * 0.5
        }
        HudAnchor::ScreenTopRight { top_right, size } => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(size);
            }
            Vec2::new(
                window_size.x - top_right.x - size.x * 0.5,
                top_right.y + size.y * 0.5,
            )
        }
        HudAnchor::ScreenTopCenter { top_y, size } => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(size);
            }
            Vec2::new(window_size.x * 0.5, top_y + size.y * 0.5)
        }
        HudAnchor::ScreenCenter { size } => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(size);
            }
            window_size * 0.5
        }
        HudAnchor::ScreenBottomLeft { bottom_left, size } => {
            if let Some(sprite) = sprite.as_deref_mut() {
                sprite.custom_size = Some(size);
            }
            Vec2::new(
                bottom_left.x + size.x * 0.5,
                window_size.y - bottom_left.y - size.y * 0.5,
            )
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
    use crate::original::objects::RobotType;

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
    fn attack_alert_check_counter_matches_original_reset_delay() {
        let mut alert = HudAttackAlert::default();
        alert.source_set_ref_id(7, 5.0);

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
            anim_elapsed: 0.0,
            anim_delay: 5.0,
            last_anim: None,
        };

        process_attack_alert_check(&mut alert, true);

        assert_eq!(alert.target_ref_id, Some(9));
        assert_eq!(alert.not_under_attack_checks, 0);
    }

    #[test]
    fn attack_alert_repeat_delay_matches_source_range() {
        let mut rng = CombatRng::default();
        for _ in 0..10 {
            let delay = attack_alert_next_repeat_delay(&mut rng);
            assert!((5.0..=7.99).contains(&delay));
        }
    }

    #[test]
    fn attack_alert_repeat_waits_for_scheduled_time() {
        let mut alert = HudAttackAlert::default();
        alert.source_set_ref_id(7, 5.0);
        let mut portrait_state = PortraitAnimationState::default();
        let mut portrait_sounds = PortraitAnimationSoundQueue::default();
        let mut rng = CombatRng::default();

        assert!(!process_attack_alert_repeat_animation(
            &mut alert,
            7,
            4.99,
            &mut portrait_state,
            &mut portrait_sounds,
            &mut rng,
        ));
        assert!(portrait_sounds.pending.is_empty());

        assert!(process_attack_alert_repeat_animation(
            &mut alert,
            7,
            0.01,
            &mut portrait_state,
            &mut portrait_sounds,
            &mut rng,
        ));
        assert!(portrait_state.doing_anim());
        assert!(matches!(
            portrait_sounds.pending.as_slice(),
            [PortraitAnimationKind::UnderAttackRepeat(_)]
        ));
        assert!(alert.last_anim.is_some());
        assert!(alert.anim_delay >= 5.0);
    }

    #[test]
    fn attack_alert_repeat_keeps_last_anim_when_portrait_busy() {
        let mut alert = HudAttackAlert::default();
        alert.source_set_ref_id(7, 0.0);
        alert.last_anim = Some(2);
        let mut portrait_state = PortraitAnimationState::default();
        portrait_state.start(PortraitAnimationEvent {
            ref_id: 3,
            kind: PortraitAnimationKind::TargetDestroyed,
        });
        let mut portrait_sounds = PortraitAnimationSoundQueue::default();
        let mut rng = CombatRng::default();

        assert!(!process_attack_alert_repeat_animation(
            &mut alert,
            7,
            0.0,
            &mut portrait_state,
            &mut portrait_sounds,
            &mut rng,
        ));
        assert_eq!(alert.last_anim, Some(2));
        assert!(portrait_sounds.pending.is_empty());
        assert!(alert.anim_delay >= 5.0);
    }

    #[test]
    fn attack_alert_repeat_does_not_repeat_last_anim() {
        let mut alert = HudAttackAlert::default();
        alert.source_set_ref_id(7, 0.0);
        let mut rng = CombatRng::default();

        for _ in 0..20 {
            let previous = alert.last_anim;
            let mut portrait_state = PortraitAnimationState::default();
            let mut portrait_sounds = PortraitAnimationSoundQueue::default();
            let delta_secs = alert.anim_delay;
            assert!(process_attack_alert_repeat_animation(
                &mut alert,
                7,
                delta_secs,
                &mut portrait_state,
                &mut portrait_sounds,
                &mut rng,
            ));
            assert_ne!(alert.last_anim, previous);
        }
    }

    #[test]
    fn fort_message_blinks_ten_flips_then_holds_visible() {
        let mut message = ComputerMessageState::default();
        start_fort_under_attack_message(&mut message, 42);

        assert_eq!(message.kind, Some(ComputerMessageKind::FortUnderAttack));
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

    #[test]
    fn manufactured_computer_message_focus_flags_match_source_events() {
        assert_eq!(
            computer_message_space_bar_event(ComputerMessageKind::RobotManufactured, 10),
            SpaceBarEvent::new(10, true, false)
        );
        assert_eq!(
            computer_message_space_bar_event(ComputerMessageKind::VehicleManufactured, 11),
            SpaceBarEvent::new(11, true, false)
        );
        assert_eq!(
            computer_message_space_bar_event(ComputerMessageKind::GunManufactured, 12),
            SpaceBarEvent::new(12, false, true)
        );
        assert_eq!(
            computer_message_space_bar_event(ComputerMessageKind::FortUnderAttack, 13),
            SpaceBarEvent::new(13, false, false)
        );
    }

    #[test]
    fn resume_prompt_click_rect_uses_original_centering() {
        let window_size = Vec2::new(800.0, 600.0);

        assert_eq!(
            resume_prompt_screen_top_left(window_size),
            Vec2::new(315.0, 293.0)
        );
        assert!(resume_prompt_contains(
            Vec2::new(400.0, 300.0),
            window_size,
            true
        ));
        assert!(!resume_prompt_contains(
            Vec2::new(400.0, 292.0),
            window_size,
            true
        ));
        assert!(!resume_prompt_contains(
            Vec2::new(400.0, 300.0),
            window_size,
            false
        ));
    }

    fn stored_gun_snapshot(
        ref_id: u32,
        kind: ObjectKind,
        team: TeamType,
        destroyed: bool,
        stored_cannon_count: usize,
    ) -> StoredGunHudSnapshot {
        StoredGunHudSnapshot {
            ref_id,
            kind,
            team,
            destroyed,
            stored_cannon_count,
        }
    }

    #[test]
    fn stored_gun_slots_match_source_positions_and_icon_hit_rect() {
        assert_eq!(STORED_GUN_ICON_SIZE, Vec2::new(16.0, 14.0));
        assert_eq!(stored_gun_icon_top_left(0), Vec2::new(8.0, 8.0));
        assert_eq!(stored_gun_icon_top_left(1), Vec2::new(8.0, 24.0));
        assert_eq!(stored_gun_icon_top_left(7), Vec2::new(8.0, 120.0));

        assert!(stored_gun_slot_contains(0, Vec2::new(8.0, 8.0)));
        assert!(stored_gun_slot_contains(0, Vec2::new(24.0, 22.0)));
        assert!(!stored_gun_slot_contains(0, Vec2::new(28.0, 11.0)));
        assert!(!stored_gun_slot_contains(0, Vec2::new(8.0, 23.0)));
    }

    #[test]
    fn vote_panel_positions_match_source_top_right_offsets() {
        let window_size = Vec2::new(1280.0, 720.0);
        assert_eq!(
            hud_anchor_screen_position(
                HudAnchor::ScreenTopRight {
                    top_right: VOTE_PANEL_TOP_RIGHT,
                    size: VOTE_PANEL_SIZE,
                },
                window_size,
                None,
            ),
            Vec2::new(1220.0, 40.5)
        );
        assert_eq!(
            vote_text_top_right(VOTE_DESCRIPTION_OFFSET),
            Vec2::new(59.0, 45.0)
        );
        assert_eq!(vote_text_top_right(VOTE_FOR_OFFSET), Vec2::new(94.0, 68.0));
        assert_eq!(
            vote_text_top_right(VOTE_AGAINST_OFFSET),
            Vec2::new(25.0, 68.0)
        );
    }

    #[test]
    fn vote_text_values_match_source_setup_images_fields() {
        let snapshot = VoteDisplaySnapshot {
            description: "Resume Game".to_string(),
            have_votes: 1,
            needed_votes: 3,
            for_votes: 1,
            against_votes: 0,
        };

        assert_eq!(
            vote_text_value(HudVoteTextField::Description, &snapshot),
            "Resume Game"
        );
        assert_eq!(vote_text_value(HudVoteTextField::Have, &snapshot), "1");
        assert_eq!(vote_text_value(HudVoteTextField::Needed, &snapshot), "3");
        assert_eq!(vote_text_value(HudVoteTextField::ForVotes, &snapshot), "1");
        assert_eq!(
            vote_text_value(HudVoteTextField::AgainstVotes, &snapshot),
            "0"
        );
    }

    #[test]
    fn news_text_positions_match_source_bottom_left_stack() {
        let window_size = Vec2::new(800.0, 600.0);

        assert_eq!(news_text_bottom_left(0), Vec2::new(5.0, 51.0));
        assert_eq!(news_text_bottom_left(1), Vec2::new(5.0, 66.0));
        assert_eq!(
            hud_anchor_screen_position(
                HudAnchor::ScreenBottomLeft {
                    bottom_left: news_text_bottom_left(0),
                    size: Vec2::ZERO,
                },
                window_size,
                None,
            ),
            Vec2::new(5.0, 549.0)
        );
    }

    #[test]
    fn chat_text_position_and_prefix_match_source_draft() {
        let window_size = Vec2::new(800.0, 600.0);

        assert_eq!(chat_text_bottom_left(), Vec2::new(209.0, 19.0));
        assert_eq!(chat_draft_text("hello"), "Say:: hello");
        assert_eq!(
            hud_anchor_screen_position(
                HudAnchor::ScreenBottomLeft {
                    bottom_left: chat_text_bottom_left(),
                    size: Vec2::ZERO,
                },
                window_size,
                None,
            ),
            Vec2::new(209.0, 581.0)
        );
    }

    #[test]
    fn chat_text_keeps_tail_when_source_area_would_clip_left() {
        let long = "a".repeat(CHAT_TEXT_MAX_CHARS + 10);
        let text = chat_draft_text(&long);

        assert_eq!(text.chars().count(), CHAT_TEXT_MAX_CHARS);
        assert!(text.chars().all(|character| character == 'a'));
    }

    #[test]
    fn stored_gun_entries_filter_and_cap_source_render_list() {
        let mut snapshots = vec![
            stored_gun_snapshot(
                99,
                ObjectKind::Building(BuildingType::RobotFactory),
                TeamType::Blue,
                false,
                2,
            ),
            stored_gun_snapshot(
                98,
                ObjectKind::Building(BuildingType::RobotFactory),
                TeamType::Red,
                true,
                2,
            ),
            stored_gun_snapshot(
                97,
                ObjectKind::Building(BuildingType::Radar),
                TeamType::Red,
                false,
                2,
            ),
            stored_gun_snapshot(
                96,
                ObjectKind::Building(BuildingType::RobotFactory),
                TeamType::Red,
                false,
                0,
            ),
        ];
        for ref_id in 1..=9 {
            snapshots.push(stored_gun_snapshot(
                ref_id,
                ObjectKind::Building(BuildingType::VehicleFactory),
                TeamType::Red,
                false,
                ref_id as usize,
            ));
        }
        snapshots.sort_by_key(|snapshot| snapshot.ref_id);

        let entries = stored_gun_hud_entries(&snapshots, TeamType::Red);
        assert_eq!(entries.len(), MAX_RENDERABLE_STORED_GUNS);
        assert_eq!(
            entries.iter().map(|entry| entry.ref_id).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6, 7, 8]
        );
    }

    #[test]
    fn stored_gun_multiplier_text_matches_source_count_gate() {
        assert_eq!(stored_gun_multiplier_text(0), None);
        assert_eq!(stored_gun_multiplier_text(1), None);
        assert_eq!(stored_gun_multiplier_text(2), Some("X2".to_string()));
        assert_eq!(stored_gun_multiplier_text(4), Some("X4".to_string()));
        assert_eq!(stored_gun_multiplier_text(5), None);
    }

    #[test]
    fn stored_gun_click_resolves_icon_slots_to_building_refs() {
        let entries = [
            StoredGunHudEntry {
                ref_id: 11,
                stored_cannon_count: 1,
            },
            StoredGunHudEntry {
                ref_id: 22,
                stored_cannon_count: 3,
            },
        ];

        assert_eq!(
            stored_gun_ref_at_cursor(Vec2::new(9.0, 9.0), &entries),
            Some(11)
        );
        assert_eq!(
            stored_gun_ref_at_cursor(Vec2::new(9.0, 25.0), &entries),
            Some(22)
        );
        assert_eq!(
            stored_gun_ref_at_cursor(Vec2::new(28.0, 12.0), &entries),
            None
        );
        assert_eq!(
            stored_gun_ref_at_cursor(Vec2::new(9.0, 41.0), &entries),
            None
        );
    }
}
