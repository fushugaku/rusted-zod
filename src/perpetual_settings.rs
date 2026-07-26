use bevy::prelude::Resource;

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub(crate) struct PerpetualServerSettings {
    pub(crate) loaded_from_file: bool,
    pub(crate) ignore_activation: bool,
    pub(crate) require_login: bool,
    pub(crate) use_database: bool,
    pub(crate) use_mysql: bool,
    pub(crate) start_map_paused: bool,
    pub(crate) bots_start_ignored: bool,
    pub(crate) allow_game_speed_change: bool,
    pub(crate) selectable_map_list: String,
}

impl Default for PerpetualServerSettings {
    fn default() -> Self {
        Self {
            loaded_from_file: false,
            ignore_activation: true,
            require_login: false,
            use_database: false,
            use_mysql: false,
            start_map_paused: true,
            bots_start_ignored: false,
            allow_game_speed_change: true,
            selectable_map_list: String::new(),
        }
    }
}

impl PerpetualServerSettings {
    pub(crate) fn load_platform() -> Self {
        let mut settings = Self::default();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = std::env::var_os("ZOD_PSETTINGS")
            && let Ok(contents) = std::fs::read_to_string(path)
        {
            settings.apply_source_file(&contents);
        }
        settings.apply_environment_overrides();
        settings
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_source_file(&mut self, contents: &str) {
        let mut loaded = false;
        for raw_line in contents.split_terminator('\n') {
            let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            loaded = true;
            let mut fields = line.splitn(3, '=');
            let variable = fields.next().unwrap_or_default();
            let value = fields.next().unwrap_or_default();
            self.apply_source_value(variable, value);
        }
        self.loaded_from_file = loaded;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_source_value(&mut self, variable: &str, value: &str) {
        let bool_value = source_atoi(value) != 0;
        match variable {
            "ignore_activation" => self.ignore_activation = bool_value,
            "require_login" => self.require_login = bool_value,
            "use_database" => self.use_database = bool_value,
            "use_mysql" => self.use_mysql = bool_value,
            "start_map_paused" => self.start_map_paused = bool_value,
            "bots_start_ignored" => self.bots_start_ignored = bool_value,
            "allow_game_speed_change" => self.allow_game_speed_change = bool_value,
            "selectable_map_list" => self.selectable_map_list = value.to_string(),
            _ => {}
        }
    }

    fn apply_environment_overrides(&mut self) {
        for (name, target) in [
            ("ZOD_IGNORE_ACTIVATION", &mut self.ignore_activation),
            ("ZOD_REQUIRE_LOGIN", &mut self.require_login),
            ("ZOD_USE_DATABASE", &mut self.use_database),
            ("ZOD_USE_MYSQL", &mut self.use_mysql),
            ("ZOD_START_MAP_PAUSED", &mut self.start_map_paused),
            ("ZOD_BOTS_START_IGNORED", &mut self.bots_start_ignored),
            (
                "ZOD_ALLOW_GAME_SPEED_CHANGE",
                &mut self.allow_game_speed_change,
            ),
        ] {
            if let Some(value) = source_env_bool(name) {
                *target = value;
            }
        }
        if let Ok(path) = std::env::var("ZOD_SELECTABLE_MAP_LIST") {
            self.selectable_map_list = path;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn source_atoi(value: &str) -> i32 {
    let value = value.trim_start();
    let (negative, digits) = match value.as_bytes().first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    let mut parsed = 0_i32;
    let mut found = false;
    for byte in digits.bytes() {
        if !byte.is_ascii_digit() {
            break;
        }
        found = true;
        parsed = parsed
            .saturating_mul(10)
            .saturating_add(i32::from(byte - b'0'));
    }
    if !found {
        0
    } else if negative {
        parsed.saturating_neg()
    } else {
        parsed
    }
}

fn source_env_bool(name: &str) -> Option<bool> {
    let value = std::env::var(name).ok()?;
    Some(matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_zpsettings_load_defaults() {
        let settings = PerpetualServerSettings::default();
        assert!(!settings.loaded_from_file);
        assert!(settings.ignore_activation);
        assert!(!settings.require_login);
        assert!(!settings.use_database);
        assert!(!settings.use_mysql);
        assert!(settings.start_map_paused);
        assert!(!settings.bots_start_ignored);
        assert!(settings.allow_game_speed_change);
        assert!(settings.selectable_map_list.is_empty());
    }

    #[test]
    fn source_file_parser_preserves_bool_atoi_comments_and_exact_map_path() {
        let mut settings = PerpetualServerSettings::default();
        settings.apply_source_file(
            "# comment\r\nrequire_login=2junk\r\nuse_database=-1\nuse_mysql=0\n\
             start_map_paused=0\nbots_start_ignored=1\nallow_game_speed_change=0\n\
             ignore_activation=0\nselectable_map_list=maps/custom list.txt\nunknown=x\n",
        );

        assert!(settings.loaded_from_file);
        assert!(settings.require_login);
        assert!(settings.use_database);
        assert!(!settings.use_mysql);
        assert!(!settings.start_map_paused);
        assert!(settings.bots_start_ignored);
        assert!(!settings.allow_game_speed_change);
        assert!(!settings.ignore_activation);
        assert_eq!(settings.selectable_map_list, "maps/custom list.txt");
    }
}
