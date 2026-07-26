use bevy::{
    camera::visibility::RenderLayers,
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
};

use crate::{
    account::{AccountCommand, LocalAccountStore, LoginPromptState, process_account_command},
    components::{HudAnchor, HudAssets},
    constants::{HUD_HEIGHT, HUD_LAYER, HUD_WIDTH},
    local_player::LocalPlayerState,
    news::NewsLog,
};

const LOGIN_SIZE: Vec2 = Vec2::new(112.0, 100.0);
const CREATE_SIZE: Vec2 = Vec2::new(112.0, 157.0);
const BUTTON_SIZE: Vec2 = Vec2::new(38.0, 14.0);
const FIELD_SIZE: Vec2 = Vec2::new(99.0, 11.0);
const MAX_PLAYER_NAME_SIZE: usize = 30;
const MAX_EMAIL_SIZE: usize = 250;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum AccountMenuMode {
    #[default]
    Login,
    CreateUser,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum AccountMenuField {
    #[default]
    UserName,
    LoginName,
    Password,
    Email,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccountMenuButton {
    Login,
    Create,
    Ok,
    Cancel,
}

#[derive(Resource)]
pub(crate) struct AccountMenuState {
    mode: AccountMenuMode,
    selected: AccountMenuField,
    user_name: String,
    login_name: String,
    password: String,
    email: String,
    pressed: Option<AccountMenuButton>,
}

impl Default for AccountMenuState {
    fn default() -> Self {
        Self {
            mode: AccountMenuMode::Login,
            selected: AccountMenuField::LoginName,
            user_name: String::new(),
            login_name: String::new(),
            password: String::new(),
            email: String::new(),
            pressed: None,
        }
    }
}

impl AccountMenuState {
    pub(crate) fn from_env() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut state = Self::default();
            if std::env::var("ZOD_DEBUG_LOGIN")
                .is_ok_and(|value| value.eq_ignore_ascii_case("create"))
            {
                state.mode = AccountMenuMode::CreateUser;
                state.selected = AccountMenuField::UserName;
            }
            state
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self::default()
        }
    }
}

#[derive(Resource)]
pub(crate) struct AccountMenuAssets {
    login_background: Handle<Image>,
    create_background: Handle<Image>,
    login_button: Handle<Image>,
    login_button_pressed: Handle<Image>,
    create_button: Handle<Image>,
    create_button_pressed: Handle<Image>,
    ok_button: Handle<Image>,
    ok_button_pressed: Handle<Image>,
    cancel_button: Handle<Image>,
    cancel_button_pressed: Handle<Image>,
}

pub(crate) fn load_account_menu_assets(asset_server: &AssetServer) -> AccountMenuAssets {
    AccountMenuAssets {
        login_background: asset_server.load("other/menus/login_menu.png"),
        create_background: asset_server.load("other/menus/create_user_menu.png"),
        login_button: asset_server.load("other/menus/login_button.png"),
        login_button_pressed: asset_server.load("other/menus/login_button_pressed.png"),
        create_button: asset_server.load("other/menus/create_button.png"),
        create_button_pressed: asset_server.load("other/menus/create_button_pressed.png"),
        ok_button: asset_server.load("other/menus/ok_button.png"),
        ok_button_pressed: asset_server.load("other/menus/ok_button_pressed.png"),
        cancel_button: asset_server.load("other/menus/cancel_button.png"),
        cancel_button_pressed: asset_server.load("other/menus/cancel_button_pressed.png"),
    }
}

#[derive(Component)]
pub(crate) struct AccountMenuNode;

#[derive(Component)]
pub(crate) enum AccountMenuVisual {
    Background,
    Button(AccountMenuButton),
}

#[derive(Component)]
pub(crate) struct AccountMenuText(AccountMenuField);

pub(crate) fn spawn_account_menu(
    mut commands: Commands,
    assets: Res<AccountMenuAssets>,
    hud_assets: Res<HudAssets>,
) {
    commands.spawn((
        Sprite::from_image(assets.login_background.clone()),
        Visibility::Hidden,
        Transform::from_xyz(0.0, 0.0, 800.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::ScreenTopLeft {
            top_left: Vec2::ZERO,
            size: LOGIN_SIZE,
        },
        AccountMenuNode,
        AccountMenuVisual::Background,
        Name::new("account_menu_background"),
    ));
    for button in [
        AccountMenuButton::Login,
        AccountMenuButton::Create,
        AccountMenuButton::Ok,
        AccountMenuButton::Cancel,
    ] {
        commands.spawn((
            Sprite::from_image(button_image(&assets, button, false)),
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, 801.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::ScreenTopLeft {
                top_left: Vec2::ZERO,
                size: BUTTON_SIZE,
            },
            AccountMenuNode,
            AccountMenuVisual::Button(button),
            Name::new("account_menu_button"),
        ));
    }
    for field in [
        AccountMenuField::UserName,
        AccountMenuField::LoginName,
        AccountMenuField::Password,
        AccountMenuField::Email,
    ] {
        commands.spawn((
            Text2d::new(""),
            TextFont {
                font: hud_assets.font.clone(),
                font_size: 8.0,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(Justify::Left),
            bevy::sprite::Anchor::TOP_LEFT,
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, 802.0),
            RenderLayers::layer(HUD_LAYER),
            HudAnchor::ScreenTopLeft {
                top_left: Vec2::ZERO,
                size: Vec2::ZERO,
            },
            AccountMenuNode,
            AccountMenuText(field),
            Name::new("account_menu_text"),
        ));
    }
}

pub(crate) fn process_account_menu_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut prompt: ResMut<LoginPromptState>,
    mut state: ResMut<AccountMenuState>,
    mut store: ResMut<LocalAccountStore>,
    mut player: ResMut<LocalPlayerState>,
    mut news: ResMut<NewsLog>,
) {
    prompt.captured_input = false;
    if !prompt.show_login {
        state.pressed = None;
        return;
    }
    prompt.captured_input = true;

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::Tab => select_next_field(&mut state),
            KeyCode::Enter | KeyCode::NumpadEnter => {
                submit_current_menu(&state, &mut store, &mut prompt, &mut player, &mut news)
            }
            KeyCode::Backspace => {
                state.selected_text_mut().pop();
            }
            _ => {
                if let Some(text) = &event.text {
                    push_source_text(&mut state, text);
                }
            }
        }
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let top_left = menu_top_left(window, state.mode);
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(field) = field_at(state.mode, cursor - top_left) {
            state.selected = field;
        }
        state.pressed = button_at(state.mode, cursor - top_left);
    }
    if mouse.just_released(MouseButton::Left) {
        let released = button_at(state.mode, cursor - top_left);
        let pressed = state.pressed.take();
        if pressed == released {
            match pressed {
                Some(AccountMenuButton::Login) => {
                    submit_current_menu(&state, &mut store, &mut prompt, &mut player, &mut news)
                }
                Some(AccountMenuButton::Create) => {
                    state.mode = AccountMenuMode::CreateUser;
                    state.selected = AccountMenuField::UserName;
                }
                Some(AccountMenuButton::Ok) => {
                    submit_current_menu(&state, &mut store, &mut prompt, &mut player, &mut news)
                }
                Some(AccountMenuButton::Cancel) => {
                    state.mode = AccountMenuMode::Login;
                    state.selected = AccountMenuField::LoginName;
                }
                None => {}
            }
        }
    }
}

pub(crate) fn sync_account_menu_visuals(
    prompt: Res<LoginPromptState>,
    state: Res<AccountMenuState>,
    assets: Res<AccountMenuAssets>,
    windows: Query<&Window>,
    mut visuals: Query<
        (
            &AccountMenuVisual,
            &mut Sprite,
            &mut Visibility,
            &mut HudAnchor,
        ),
        Without<AccountMenuText>,
    >,
    mut texts: Query<
        (
            &AccountMenuText,
            &mut Text2d,
            &mut Visibility,
            &mut HudAnchor,
        ),
        Without<AccountMenuVisual>,
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let top_left = menu_top_left(window, state.mode);
    for (visual, mut sprite, mut visibility, mut anchor) in &mut visuals {
        let (visible, point, size) = match *visual {
            AccountMenuVisual::Background => {
                sprite.image = match state.mode {
                    AccountMenuMode::Login => assets.login_background.clone(),
                    AccountMenuMode::CreateUser => assets.create_background.clone(),
                };
                (true, top_left, menu_size(state.mode))
            }
            AccountMenuVisual::Button(button) => {
                let shown = button_visible(state.mode, button);
                sprite.image = button_image(&assets, button, state.pressed == Some(button));
                (
                    shown,
                    top_left + button_offset(state.mode, button),
                    BUTTON_SIZE,
                )
            }
        };
        *anchor = HudAnchor::ScreenTopLeft {
            top_left: point,
            size,
        };
        *visibility = if prompt.show_login && visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (field, mut text, mut visibility, mut anchor) in &mut texts {
        let visible = field_visible(state.mode, field.0);
        let mut value = state.field_text(field.0).to_string();
        if field.0 == AccountMenuField::Password {
            value = "*".repeat(value.chars().count());
        }
        if state.selected == field.0 {
            value.push('{');
        }
        *text = Text2d::new(source_visible_text_tail(&value));
        *anchor = HudAnchor::ScreenTopLeft {
            top_left: top_left + field_text_offset(state.mode, field.0),
            size: Vec2::ZERO,
        };
        *visibility = if prompt.show_login && visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

impl AccountMenuState {
    fn field_text(&self, field: AccountMenuField) -> &str {
        match field {
            AccountMenuField::UserName => &self.user_name,
            AccountMenuField::LoginName => &self.login_name,
            AccountMenuField::Password => &self.password,
            AccountMenuField::Email => &self.email,
        }
    }

    fn selected_text_mut(&mut self) -> &mut String {
        match self.selected {
            AccountMenuField::UserName => &mut self.user_name,
            AccountMenuField::LoginName => &mut self.login_name,
            AccountMenuField::Password => &mut self.password,
            AccountMenuField::Email => &mut self.email,
        }
    }
}

fn submit_current_menu(
    state: &AccountMenuState,
    store: &mut LocalAccountStore,
    prompt: &mut LoginPromptState,
    player: &mut LocalPlayerState,
    news: &mut NewsLog,
) {
    let command = match state.mode {
        AccountMenuMode::Login => AccountCommand::Login {
            login_name: state.login_name.clone(),
            password: state.password.clone(),
        },
        AccountMenuMode::CreateUser => AccountCommand::CreateUser {
            user_name: state.user_name.clone(),
            login_name: state.login_name.clone(),
            password: state.password.clone(),
            email: state.email.clone(),
        },
    };
    process_account_command(store, prompt, player, news, command);
}

fn select_next_field(state: &mut AccountMenuState) {
    state.selected = match (state.mode, state.selected) {
        (AccountMenuMode::Login, AccountMenuField::LoginName) => AccountMenuField::Password,
        (AccountMenuMode::Login, _) => AccountMenuField::LoginName,
        (AccountMenuMode::CreateUser, AccountMenuField::UserName) => AccountMenuField::LoginName,
        (AccountMenuMode::CreateUser, AccountMenuField::LoginName) => AccountMenuField::Password,
        (AccountMenuMode::CreateUser, AccountMenuField::Password) => AccountMenuField::Email,
        (AccountMenuMode::CreateUser, AccountMenuField::Email) => AccountMenuField::UserName,
    };
}

fn push_source_text(state: &mut AccountMenuState, text: &str) {
    let max = if state.selected == AccountMenuField::Email {
        MAX_EMAIL_SIZE
    } else {
        MAX_PLAYER_NAME_SIZE
    };
    for character in text.chars() {
        if state.selected_text_mut().len() >= max {
            break;
        }
        if character.is_ascii_alphanumeric() || matches!(character, ' ' | '@' | '.' | '_' | '-') {
            state.selected_text_mut().push(character);
        }
    }
}

fn menu_size(mode: AccountMenuMode) -> Vec2 {
    match mode {
        AccountMenuMode::Login => LOGIN_SIZE,
        AccountMenuMode::CreateUser => CREATE_SIZE,
    }
}

fn menu_top_left(window: &Window, mode: AccountMenuMode) -> Vec2 {
    let size = menu_size(mode);
    Vec2::new(
        ((window.width() - HUD_WIDTH - size.x) * 0.5).floor(),
        ((window.height() - HUD_HEIGHT - size.y) * 0.5).floor(),
    )
}

pub(crate) fn account_menu_contains_cursor(
    prompt: &LoginPromptState,
    state: &AccountMenuState,
    window: &Window,
    cursor: Vec2,
) -> bool {
    prompt.show_login
        && point_in_rect(
            cursor,
            menu_top_left(window, state.mode),
            menu_size(state.mode),
        )
}

fn button_visible(mode: AccountMenuMode, button: AccountMenuButton) -> bool {
    matches!(
        (mode, button),
        (
            AccountMenuMode::Login,
            AccountMenuButton::Login | AccountMenuButton::Create
        ) | (
            AccountMenuMode::CreateUser,
            AccountMenuButton::Ok | AccountMenuButton::Cancel
        )
    )
}

fn button_offset(mode: AccountMenuMode, button: AccountMenuButton) -> Vec2 {
    match (mode, button) {
        (AccountMenuMode::Login, AccountMenuButton::Login) => Vec2::new(8.0, 83.0),
        (AccountMenuMode::Login, AccountMenuButton::Create) => Vec2::new(66.0, 83.0),
        (AccountMenuMode::CreateUser, AccountMenuButton::Ok) => Vec2::new(8.0, 140.0),
        (AccountMenuMode::CreateUser, AccountMenuButton::Cancel) => Vec2::new(66.0, 140.0),
        _ => Vec2::ZERO,
    }
}

fn button_at(mode: AccountMenuMode, local: Vec2) -> Option<AccountMenuButton> {
    [
        AccountMenuButton::Login,
        AccountMenuButton::Create,
        AccountMenuButton::Ok,
        AccountMenuButton::Cancel,
    ]
    .into_iter()
    .find(|button| {
        button_visible(mode, *button)
            && point_in_rect(local, button_offset(mode, *button), BUTTON_SIZE)
    })
}

fn field_visible(mode: AccountMenuMode, field: AccountMenuField) -> bool {
    mode == AccountMenuMode::CreateUser
        || matches!(
            field,
            AccountMenuField::LoginName | AccountMenuField::Password
        )
}

fn field_offset(mode: AccountMenuMode, field: AccountMenuField) -> Vec2 {
    match (mode, field) {
        (AccountMenuMode::Login, AccountMenuField::LoginName)
        | (AccountMenuMode::CreateUser, AccountMenuField::UserName) => Vec2::new(6.0, 35.0),
        (AccountMenuMode::Login, AccountMenuField::Password)
        | (AccountMenuMode::CreateUser, AccountMenuField::LoginName) => Vec2::new(6.0, 64.0),
        (AccountMenuMode::CreateUser, AccountMenuField::Password) => Vec2::new(6.0, 94.0),
        (AccountMenuMode::CreateUser, AccountMenuField::Email) => Vec2::new(6.0, 122.0),
        _ => Vec2::ZERO,
    }
}

fn field_text_offset(mode: AccountMenuMode, field: AccountMenuField) -> Vec2 {
    field_offset(mode, field) + Vec2::splat(2.0)
}

fn field_at(mode: AccountMenuMode, local: Vec2) -> Option<AccountMenuField> {
    [
        AccountMenuField::UserName,
        AccountMenuField::LoginName,
        AccountMenuField::Password,
        AccountMenuField::Email,
    ]
    .into_iter()
    .find(|field| {
        field_visible(mode, *field) && point_in_rect(local, field_offset(mode, *field), FIELD_SIZE)
    })
}

fn point_in_rect(point: Vec2, top_left: Vec2, size: Vec2) -> bool {
    point.x >= top_left.x
        && point.y >= top_left.y
        && point.x <= top_left.x + size.x
        && point.y <= top_left.y + size.y
}

fn button_image(
    assets: &AccountMenuAssets,
    button: AccountMenuButton,
    pressed: bool,
) -> Handle<Image> {
    match (button, pressed) {
        (AccountMenuButton::Login, false) => assets.login_button.clone(),
        (AccountMenuButton::Login, true) => assets.login_button_pressed.clone(),
        (AccountMenuButton::Create, false) => assets.create_button.clone(),
        (AccountMenuButton::Create, true) => assets.create_button_pressed.clone(),
        (AccountMenuButton::Ok, false) => assets.ok_button.clone(),
        (AccountMenuButton::Ok, true) => assets.ok_button_pressed.clone(),
        (AccountMenuButton::Cancel, false) => assets.cancel_button.clone(),
        (AccountMenuButton::Cancel, true) => assets.cancel_button_pressed.clone(),
    }
}

fn source_visible_text_tail(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    chars[chars.len().saturating_sub(18)..].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_tab_cycles_match_both_windows() {
        let mut state = AccountMenuState::default();
        select_next_field(&mut state);
        assert_eq!(state.selected, AccountMenuField::Password);
        select_next_field(&mut state);
        assert_eq!(state.selected, AccountMenuField::LoginName);

        state.mode = AccountMenuMode::CreateUser;
        state.selected = AccountMenuField::UserName;
        for expected in [
            AccountMenuField::LoginName,
            AccountMenuField::Password,
            AccountMenuField::Email,
            AccountMenuField::UserName,
        ] {
            select_next_field(&mut state);
            assert_eq!(state.selected, expected);
        }
    }

    #[test]
    fn text_box_character_filter_and_limits_match_source() {
        let mut state = AccountMenuState::default();
        push_source_text(&mut state, "a! b@._-Ж");
        assert_eq!(state.login_name, "a b@._-");
        push_source_text(&mut state, &"x".repeat(100));
        assert_eq!(state.login_name.len(), MAX_PLAYER_NAME_SIZE);
    }

    #[test]
    fn source_button_and_field_hit_boxes_use_original_offsets() {
        assert_eq!(
            button_at(AccountMenuMode::Login, Vec2::new(8.0, 83.0)),
            Some(AccountMenuButton::Login)
        );
        assert_eq!(
            button_at(AccountMenuMode::CreateUser, Vec2::new(66.0, 140.0)),
            Some(AccountMenuButton::Cancel)
        );
        assert_eq!(
            field_at(AccountMenuMode::CreateUser, Vec2::new(8.0, 124.0)),
            Some(AccountMenuField::Email)
        );
    }
}
