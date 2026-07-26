use aes::{
    Aes128,
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray},
};
use bevy::prelude::Resource;

use crate::{
    local_player::{
        LocalPlayerState, relay_account_login, relay_account_logout, relay_spend_voting_power,
    },
    network_commands::{
        BuyRegistrationKeyCommand, CommandPayload, CreateUserCommand, GiveLoginOffPacket,
        PollBuyRegistrationKeyPacket, RequestLoginOffCommand, ReturnRegistrationKeyPacket,
        SendLoginCommand,
    },
    news::NewsLog,
    perpetual_settings::PerpetualServerSettings,
};

const MAX_PLAYER_NAME_SIZE: usize = 30;
const MAX_EMAIL_SIZE: usize = 250;
const REGISTRATION_COST: i32 = 1;
const SOURCE_REGISTRATION_AES_KEY: [u8; 16] = [
    0xFE, 0xEA, 0x42, 0x35, 0x78, 0x02, 0x57, 0xEC, 0xEE, 0x92, 0x11, 0x58, 0xC2, 0x5D, 0xC3, 0x23,
];
#[cfg(not(target_arch = "wasm32"))]
const REGISTRATION_FILE_NAME: &str = "registration.zkey";
#[cfg(target_arch = "wasm32")]
const REGISTRATION_STORAGE_KEY: &str = "zod.registration.zkey";
#[cfg(target_arch = "wasm32")]
const REGISTRATION_STORAGE_PROBE_KEY: &str = "zod.registration.write-probe";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AccountCommand {
    Login {
        login_name: String,
        password: String,
    },
    Logout,
    CreateUser {
        user_name: String,
        login_name: String,
        password: String,
        email: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalAccount {
    db_id: i32,
    user_name: String,
    login_name: String,
    password: String,
    email: String,
    activated: bool,
    voting_power: i32,
    total_games: i32,
}

#[derive(Debug, Resource)]
pub(crate) struct LocalAccountStore {
    use_database: bool,
    use_mysql: bool,
    require_login: bool,
    next_db_id: i32,
    accounts: Vec<LocalAccount>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub(crate) struct LoginPromptState {
    pub(crate) show_login: bool,
    pub(crate) captured_input: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum RegistrationPersistence {
    #[default]
    Memory,
    Platform,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub(crate) struct RegistrationState {
    pub(crate) is_registered: bool,
    device_id: [u8; 16],
    stored_key: Option<[u8; 16]>,
    persistence: RegistrationPersistence,
}

impl Default for RegistrationState {
    fn default() -> Self {
        Self {
            is_registered: false,
            device_id: *b"zod-rust-client!",
            stored_key: None,
            persistence: RegistrationPersistence::Memory,
        }
    }
}

impl RegistrationState {
    pub(crate) fn load_platform() -> Self {
        let mut state = Self {
            persistence: RegistrationPersistence::Platform,
            ..Self::default()
        };
        state.reload_stored_key();
        state
    }

    fn storage_writable(&self) -> bool {
        self.persistence == RegistrationPersistence::Memory || platform_registration_writable()
    }

    fn store_returned_key(&mut self, key: [u8; 16]) -> bool {
        if self.persistence == RegistrationPersistence::Memory {
            self.stored_key = Some(key);
            self.is_registered = source_decrypt_registration_key(key) == self.device_id;
            return true;
        }
        if !platform_store_registration_key(key) {
            return false;
        }
        self.reload_stored_key();
        true
    }

    fn reload_stored_key(&mut self) {
        self.stored_key = platform_load_registration_key();
        self.is_registered = self
            .stored_key
            .is_some_and(|key| source_decrypt_registration_key(key) == self.device_id);
    }
}

impl Default for LocalAccountStore {
    fn default() -> Self {
        Self::from_settings(&PerpetualServerSettings::default())
    }
}

impl LocalAccountStore {
    pub(crate) fn from_settings(settings: &PerpetualServerSettings) -> Self {
        Self {
            use_database: settings.use_database,
            use_mysql: settings.use_mysql,
            require_login: settings.require_login,
            next_db_id: 1,
            accounts: Vec::new(),
        }
    }
}

pub(crate) fn process_buy_registration(
    store: &mut LocalAccountStore,
    registration: &mut RegistrationState,
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
) {
    let poll = PollBuyRegistrationKeyPacket.encode_packet();
    if poll
        .get(8..)
        .and_then(PollBuyRegistrationKeyPacket::decode_payload)
        .is_none()
    {
        return;
    }
    if registration.is_registered {
        relay_account_news(
            news_log,
            "the zod engine is already registered on this computer",
        );
        return;
    }
    if !registration.storage_writable() {
        relay_account_news(
            news_log,
            "can not open registration file for writing, please visit www.nighsoft.com for troubleshooting",
        );
        return;
    }

    let command = BuyRegistrationKeyCommand {
        device_id: registration.device_id,
    };
    let wire = command.encode_packet();
    let Some(command) = wire
        .get(8..)
        .and_then(BuyRegistrationKeyCommand::decode_payload)
    else {
        return;
    };
    if !store.use_database || !store.use_mysql {
        relay_account_news(
            news_log,
            "buy reg key error: server not configured for selling registration keys",
        );
        return;
    }
    if !local_player.logged_in() {
        relay_account_news(news_log, "buy reg key error: you must be logged in");
        return;
    }
    if !local_player.activated() {
        relay_account_news(
            news_log,
            "buy reg key error: you must be activated, please visit www.nighsoft.com",
        );
        return;
    }
    if local_player.voting_power() < REGISTRATION_COST {
        relay_account_news(
            news_log,
            "buy reg key error: more voting power to spend required, please visit www.nighsoft.com",
        );
        return;
    }
    if !relay_spend_voting_power(local_player, REGISTRATION_COST) {
        return;
    }
    if let Some(account) = store
        .accounts
        .iter_mut()
        .find(|account| account.db_id == local_player.db_id())
    {
        account.voting_power = account.voting_power.saturating_sub(REGISTRATION_COST);
    }

    let packet = ReturnRegistrationKeyPacket {
        encrypted_key: source_encrypt_registration_key(command.device_id),
    };
    let wire = packet.encode_packet();
    let Some(packet) = wire
        .get(8..)
        .and_then(ReturnRegistrationKeyPacket::decode_payload)
    else {
        return;
    };
    if !registration.store_returned_key(packet.encrypted_key) {
        relay_account_news(
            news_log,
            "new registration key bought but could not save to file! please visit www.nighsoft.com",
        );
        return;
    }
    if registration.is_registered {
        relay_account_news(news_log, "congratulations the zod engine is now registered");
    } else {
        relay_account_news(
            news_log,
            "new registration key bought but we still are not registered! please visit www.nighsoft.com",
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn registration_file_path() -> std::path::PathBuf {
    std::env::var_os("ZOD_REGISTRATION_KEY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(REGISTRATION_FILE_NAME))
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_registration_writable() -> bool {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(registration_file_path())
        .is_ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_load_registration_key() -> Option<[u8; 16]> {
    load_registration_key_at(&registration_file_path())
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_store_registration_key(key: [u8; 16]) -> bool {
    store_registration_key_at(&registration_file_path(), key)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_registration_key_at(path: &std::path::Path) -> Option<[u8; 16]> {
    let bytes = std::fs::read(path).ok()?;
    bytes.get(..16)?.try_into().ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn store_registration_key_at(path: &std::path::Path, key: [u8; 16]) -> bool {
    std::fs::write(path, key).is_ok()
}

#[cfg(target_arch = "wasm32")]
fn browser_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
fn platform_registration_writable() -> bool {
    let Some(storage) = browser_storage() else {
        return false;
    };
    if storage
        .set_item(REGISTRATION_STORAGE_PROBE_KEY, "1")
        .is_err()
    {
        return false;
    }
    storage.remove_item(REGISTRATION_STORAGE_PROBE_KEY).is_ok()
}

#[cfg(target_arch = "wasm32")]
fn platform_load_registration_key() -> Option<[u8; 16]> {
    let encoded = browser_storage()?
        .get_item(REGISTRATION_STORAGE_KEY)
        .ok()??;
    decode_registration_key_hex(&encoded)
}

#[cfg(target_arch = "wasm32")]
fn platform_store_registration_key(key: [u8; 16]) -> bool {
    browser_storage().is_some_and(|storage| {
        storage
            .set_item(REGISTRATION_STORAGE_KEY, &encode_registration_key_hex(key))
            .is_ok()
    })
}

#[cfg(target_arch = "wasm32")]
fn encode_registration_key_hex(key: [u8; 16]) -> String {
    key.into_iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(target_arch = "wasm32")]
fn decode_registration_key_hex(encoded: &str) -> Option<[u8; 16]> {
    if encoded.len() != 32 {
        return None;
    }
    let mut key = [0; 16];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(encoded.get(index * 2..index * 2 + 2)?, 16).ok()?;
    }
    Some(key)
}

fn source_encrypt_registration_key(input: [u8; 16]) -> [u8; 16] {
    let cipher = Aes128::new_from_slice(&SOURCE_REGISTRATION_AES_KEY)
        .expect("source registration AES-128 key");
    let mut block = GenericArray::clone_from_slice(&input);
    cipher.encrypt_block(&mut block);
    block.into()
}

fn source_decrypt_registration_key(input: [u8; 16]) -> [u8; 16] {
    let cipher = Aes128::new_from_slice(&SOURCE_REGISTRATION_AES_KEY)
        .expect("source registration AES-128 key");
    let mut block = GenericArray::clone_from_slice(&input);
    cipher.decrypt_block(&mut block);
    block.into()
}

pub(crate) fn process_account_command(
    store: &mut LocalAccountStore,
    login_prompt: &mut LoginPromptState,
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
    command: AccountCommand,
) {
    match command {
        AccountCommand::Login {
            login_name,
            password,
        } => {
            let command = SendLoginCommand {
                login_name,
                password,
            };
            let wire = command.encode_packet();
            let Some(command) = wire.get(8..).and_then(SendLoginCommand::decode_payload) else {
                return;
            };
            attempt_login(store, local_player, news_log, command);
            relay_login_required(store, local_player, login_prompt);
        }
        AccountCommand::Logout => {
            if store.use_database {
                relay_account_logout(local_player, news_log);
            }
            relay_login_required(store, local_player, login_prompt);
        }
        AccountCommand::CreateUser {
            user_name,
            login_name,
            password,
            email,
        } => {
            let command = CreateUserCommand {
                user_name,
                login_name,
                password,
                email,
            };
            let wire = command.encode_packet();
            let Some(command) = wire.get(8..).and_then(CreateUserCommand::decode_payload) else {
                return;
            };
            attempt_create_user(store, local_player, news_log, command);
            relay_login_required(store, local_player, login_prompt);
        }
    }
}

fn attempt_login(
    store: &LocalAccountStore,
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
    command: SendLoginCommand,
) {
    if !store.use_database {
        relay_account_news(news_log, "login error: no database used");
        return;
    }
    if local_player.logged_in() {
        relay_account_news(news_log, "login error: please log off first");
        return;
    }
    if !source_good_user_field(&command.login_name, MAX_PLAYER_NAME_SIZE)
        || !source_good_user_field(&command.password, MAX_PLAYER_NAME_SIZE)
    {
        relay_account_news(
            news_log,
            "login error: only alphanumeric characters and entries under 30 characters long allowed",
        );
        return;
    }

    let Some(account) = store
        .accounts
        .iter()
        .find(|account| {
            account.login_name == command.login_name && account.password == command.password
        })
        .cloned()
    else {
        relay_account_news(news_log, "login error: invalid login details");
        return;
    };

    relay_account_login(
        local_player,
        news_log,
        &account.user_name,
        account.db_id,
        account.activated,
        account.voting_power,
        account.total_games,
    );
}

fn attempt_create_user(
    store: &mut LocalAccountStore,
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
    command: CreateUserCommand,
) {
    if !store.use_database {
        relay_account_news(news_log, "create user error: no database used");
        return;
    }
    if local_player.logged_in() {
        relay_account_news(news_log, "create user error: please log off first");
        return;
    }
    if !source_good_user_field(&command.user_name, MAX_PLAYER_NAME_SIZE)
        || !source_good_user_field(&command.login_name, MAX_PLAYER_NAME_SIZE)
        || !source_good_user_field(&command.password, MAX_PLAYER_NAME_SIZE)
        || !source_good_user_field(&command.email, MAX_EMAIL_SIZE)
    {
        relay_account_news(
            news_log,
            "create user error: only alphanumeric characters and entries under 30 characters long allowed",
        );
        return;
    }
    if store.accounts.iter().any(|account| {
        account.user_name == command.user_name || account.login_name == command.login_name
    }) {
        relay_account_news(news_log, "create user error: user already exists");
        return;
    }

    let account = LocalAccount {
        db_id: store.next_db_id,
        user_name: command.user_name,
        login_name: command.login_name,
        password: command.password,
        email: command.email,
        activated: true,
        voting_power: 0,
        total_games: 0,
    };
    store.next_db_id = store.next_db_id.saturating_add(1);
    relay_account_news(news_log, format!("user {} created", account.user_name));
    let login = SendLoginCommand {
        login_name: account.login_name.clone(),
        password: account.password.clone(),
    };
    store.accounts.push(account);
    attempt_login(store, local_player, news_log, login);
}

fn relay_login_required(
    store: &LocalAccountStore,
    local_player: &LocalPlayerState,
    login_prompt: &mut LoginPromptState,
) {
    let request = RequestLoginOffCommand.encode_packet();
    if request
        .get(8..)
        .and_then(RequestLoginOffCommand::decode_payload)
        .is_none()
    {
        return;
    }
    let packet = GiveLoginOffPacket {
        show_login: store.require_login && !local_player.logged_in(),
    };
    let wire = packet.encode_packet();
    if let Some(packet) = wire.get(8..).and_then(GiveLoginOffPacket::decode_payload) {
        login_prompt.show_login = packet.show_login;
    }
}

fn source_good_user_field(value: &str, max_len: usize) -> bool {
    if value.is_empty() || value.len() > max_len || value.starts_with(' ') || value.ends_with(' ') {
        return false;
    }
    if value.as_bytes().windows(2).any(|pair| pair == b"  ") {
        return false;
    }
    value.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, ' ' | '@' | '.' | '_' | '-')
    })
}

fn relay_account_news(news_log: &mut NewsLog, message: impl Into<String>) {
    news_log.relay_source_news(message, 0, 0, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_store(require_login: bool) -> LocalAccountStore {
        LocalAccountStore {
            use_database: true,
            use_mysql: true,
            require_login,
            next_db_id: 1,
            accounts: Vec::new(),
        }
    }

    #[test]
    fn create_login_logout_updates_roster_and_login_prompt() {
        let mut store = enabled_store(true);
        let mut prompt = LoginPromptState::default();
        let mut player = LocalPlayerState::default();
        crate::local_player::relay_send_player_info(&mut player, &mut NewsLog::default());
        crate::local_player::relay_request_player_list(&mut player);
        let mut news = NewsLog::default();

        process_account_command(
            &mut store,
            &mut prompt,
            &mut player,
            &mut news,
            AccountCommand::CreateUser {
                user_name: "Alice".to_string(),
                login_name: "alice_login".to_string(),
                password: "secret".to_string(),
                email: "alice@example.test".to_string(),
            },
        );
        assert!(player.logged_in());
        assert_eq!(player.name(), "Alice");
        assert!(!prompt.show_login);
        assert_eq!(
            news.display_entry(1).map(|entry| entry.message),
            Some("user Alice created")
        );
        assert_eq!(
            news.display_entry(0).map(|entry| entry.message),
            Some("Alice logged in")
        );

        process_account_command(
            &mut store,
            &mut prompt,
            &mut player,
            &mut news,
            AccountCommand::Logout,
        );
        assert!(!player.logged_in());
        assert_eq!(player.name(), "Player");
        assert!(prompt.show_login);
        assert_eq!(
            news.display_entry(0).map(|entry| entry.message),
            Some("Alice logged out")
        );
    }

    #[test]
    fn default_source_settings_reject_account_database_commands() {
        let mut store = LocalAccountStore {
            use_database: false,
            use_mysql: false,
            require_login: false,
            next_db_id: 1,
            accounts: Vec::new(),
        };
        let mut prompt = LoginPromptState::default();
        let mut player = LocalPlayerState::default();
        let mut news = NewsLog::default();

        process_account_command(
            &mut store,
            &mut prompt,
            &mut player,
            &mut news,
            AccountCommand::Login {
                login_name: "alice".to_string(),
                password: "secret".to_string(),
            },
        );
        assert_eq!(
            news.display_entry(0).map(|entry| entry.message),
            Some("login error: no database used")
        );
        assert!(!prompt.show_login);
    }

    #[test]
    fn source_user_character_and_length_rules_are_preserved() {
        assert!(source_good_user_field("A user-name_1@example.test", 30));
        assert!(!source_good_user_field(" bad", 30));
        assert!(!source_good_user_field("bad ", 30));
        assert!(!source_good_user_field("bad  name", 30));
        assert!(!source_good_user_field("bad!name", 30));
        assert!(!source_good_user_field(&"x".repeat(31), 30));
    }

    #[test]
    fn registration_purchase_spends_one_voting_power_and_verifies_source_aes_key() {
        let mut store = enabled_store(false);
        store.accounts.push(LocalAccount {
            db_id: 1,
            user_name: "Alice".to_string(),
            login_name: "alice".to_string(),
            password: "secret".to_string(),
            email: "alice@example.test".to_string(),
            activated: true,
            voting_power: 1,
            total_games: 0,
        });
        store.next_db_id = 2;
        let mut player = LocalPlayerState::default();
        crate::local_player::relay_send_player_info(&mut player, &mut NewsLog::default());
        crate::local_player::relay_request_player_list(&mut player);
        let mut news = NewsLog::default();
        attempt_login(
            &store,
            &mut player,
            &mut news,
            SendLoginCommand {
                login_name: "alice".to_string(),
                password: "secret".to_string(),
            },
        );
        let mut registration = RegistrationState::default();

        process_buy_registration(&mut store, &mut registration, &mut player, &mut news);

        assert!(registration.is_registered);
        assert_eq!(player.voting_power(), 0);
        assert_eq!(store.accounts[0].voting_power, 0);
        assert_eq!(
            registration.stored_key.map(source_decrypt_registration_key),
            Some(registration.device_id)
        );
        assert_eq!(
            news.display_entry(0).map(|entry| entry.message),
            Some("congratulations the zod engine is now registered")
        );
    }

    #[test]
    fn registration_purchase_default_server_configuration_matches_source_error() {
        let mut store = LocalAccountStore {
            use_database: false,
            use_mysql: false,
            require_login: false,
            next_db_id: 1,
            accounts: Vec::new(),
        };
        let mut registration = RegistrationState::default();
        let mut player = LocalPlayerState::default();
        let mut news = NewsLog::default();

        process_buy_registration(&mut store, &mut registration, &mut player, &mut news);

        assert!(!registration.is_registered);
        assert_eq!(
            news.display_entry(0).map(|entry| entry.message),
            Some("buy reg key error: server not configured for selling registration keys")
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_registration_file_uses_exact_source_sixteen_byte_payload() {
        let path = std::env::temp_dir().join(format!(
            "zod-registration-{}-{}.zkey",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = std::fs::remove_file(&path);
        let key = source_encrypt_registration_key(*b"zod-rust-client!");

        assert!(store_registration_key_at(&path, key));
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 16);
        assert_eq!(load_registration_key_at(&path), Some(key));
        assert_eq!(
            load_registration_key_at(&path).map(source_decrypt_registration_key),
            Some(*b"zod-rust-client!")
        );

        std::fs::write(&path, [0_u8; 15]).unwrap();
        assert_eq!(load_registration_key_at(&path), None);
        let _ = std::fs::remove_file(path);
    }
}
