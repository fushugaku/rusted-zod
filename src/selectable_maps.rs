use bevy::prelude::{Res, ResMut, Resource};

use crate::network_commands::{
    CommandPayload, GiveSelectableMapListPacket, RequestSelectableMapListCommand,
};
use crate::perpetual_settings::PerpetualServerSettings;

include!(concat!(env!("OUT_DIR"), "/selectable_maps.rs"));

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub(crate) struct SelectableMapListState {
    maps: Vec<String>,
}

#[derive(Clone, Debug)]
enum SelectableMapBytes {
    Embedded(&'static [u8]),
    #[cfg(not(target_arch = "wasm32"))]
    Native(Vec<u8>),
}

impl SelectableMapBytes {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Embedded(bytes) => bytes,
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native(bytes) => bytes,
        }
    }
}

#[derive(Clone, Debug)]
struct SelectableMapCatalogEntry {
    name: String,
    bytes: Option<SelectableMapBytes>,
}

#[derive(Clone, Debug, Resource)]
pub(crate) struct SelectableMapCatalog {
    entries: Vec<SelectableMapCatalogEntry>,
}

#[derive(Clone, Debug, Default, Resource)]
pub(crate) struct MapRotationState {
    names: Vec<String>,
    native_bytes: Vec<Option<Vec<u8>>>,
    random: bool,
    current_index: Option<usize>,
}

impl MapRotationState {
    pub(crate) fn load_platform() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = std::env::var_os("ZOD_MAP_LIST") {
            return load_native_map_rotation(std::path::Path::new(&path));
        }
        Self::default()
    }

    pub(crate) fn has_maps(&self) -> bool {
        !self.names.is_empty()
    }

    pub(crate) fn next_map<'a>(
        &'a mut self,
        catalog: &'a SelectableMapCatalog,
        random_index: impl FnOnce(usize) -> usize,
    ) -> Option<(&'a str, &'a [u8])> {
        if self.names.is_empty() {
            return None;
        }
        let index = if self.random {
            random_index(self.names.len()) % self.names.len()
        } else {
            self.current_index
                .map_or(0, |index| (index + 1) % self.names.len())
        };
        self.current_index = Some(index);
        let name = self.names.get(index)?;
        if let Some(bytes) = self.native_bytes.get(index)?.as_deref() {
            return Some((name, bytes));
        }
        catalog.source_map_named(name)
    }
}

impl Default for SelectableMapCatalog {
    fn default() -> Self {
        Self::embedded()
    }
}

impl SelectableMapCatalog {
    pub(crate) fn from_settings(_settings: &PerpetualServerSettings) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        if !_settings.selectable_map_list.is_empty() {
            return load_native_catalog_from_list_path(std::path::Path::new(
                &_settings.selectable_map_list,
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(folder) = std::env::var_os("ZOD_SELECTABLE_MAP_FOLDER") {
            return load_native_catalog_from_folder(std::path::Path::new(&folder));
        }

        Self::embedded()
    }

    fn embedded() -> Self {
        Self {
            entries: SOURCE_SELECTABLE_MAPS
                .iter()
                .map(|name| SelectableMapCatalogEntry {
                    name: (*name).to_string(),
                    bytes: source_selectable_map_bytes(name).map(SelectableMapBytes::Embedded),
                })
                .collect(),
        }
    }

    fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|entry| entry.name.as_str())
    }

    pub(crate) fn source_map<'a>(
        &'a self,
        client_maps: &[String],
        index: usize,
    ) -> Option<(&'a str, &'a [u8])> {
        let client_name = client_maps.get(index)?;
        let entry = self.entries.get(index)?;
        if entry.name != *client_name {
            return None;
        }
        Some((entry.name.as_str(), entry.bytes.as_ref()?.as_slice()))
    }

    fn source_map_named(&self, name: &str) -> Option<(&str, &[u8])> {
        let entry = self.entries.iter().find(|entry| entry.name == name)?;
        Some((entry.name.as_str(), entry.bytes.as_ref()?.as_slice()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native_catalog_from_list_path(path: &std::path::Path) -> SelectableMapCatalog {
    let entries = std::fs::read_to_string(path)
        .ok()
        .map(|contents| {
            contents
                .split_terminator('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line))
                .filter(|line| !line.is_empty())
                .map(|name| SelectableMapCatalogEntry {
                    name: name.to_string(),
                    bytes: std::fs::read(name).ok().map(SelectableMapBytes::Native),
                })
                .collect()
        })
        .unwrap_or_default();
    SelectableMapCatalog { entries }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native_catalog_from_folder(folder: &std::path::Path) -> SelectableMapCatalog {
    let mut names = std::fs::read_dir(folder)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            (path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("map")))
            .then(|| entry.file_name().to_string_lossy().to_string())
        })
        .collect::<Vec<_>>();
    names.sort();
    SelectableMapCatalog {
        entries: names
            .into_iter()
            .map(|name| SelectableMapCatalogEntry {
                bytes: std::fs::read(folder.join(&name))
                    .ok()
                    .map(SelectableMapBytes::Native),
                name,
            })
            .collect(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native_map_rotation(path: &std::path::Path) -> MapRotationState {
    let Some(contents) = std::fs::read_to_string(path).ok() else {
        return MapRotationState::default();
    };
    let mut lines = contents
        .split_terminator('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line));
    let random = lines.next().is_some_and(|line| source_atoi(line) != 0);
    let names = lines
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let native_bytes = names.iter().map(|name| std::fs::read(name).ok()).collect();
    MapRotationState {
        names,
        native_bytes,
        random,
        current_index: None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn source_atoi(value: &str) -> i32 {
    let value = value.trim_start();
    let mut bytes = value.bytes().peekable();
    let negative = match bytes.peek() {
        Some(b'-') => {
            bytes.next();
            true
        }
        Some(b'+') => {
            bytes.next();
            false
        }
        _ => false,
    };
    let mut value = 0_i32;
    for byte in bytes.take_while(u8::is_ascii_digit) {
        value = value
            .saturating_mul(10)
            .saturating_add(i32::from(byte - b'0'));
    }
    if negative {
        value.saturating_neg()
    } else {
        value
    }
}

#[derive(Default, Resource)]
pub(crate) struct SelectableMapListInitialRequestState {
    requested: bool,
}

impl SelectableMapListState {
    pub(crate) fn maps(&self) -> &[String] {
        &self.maps
    }
}

pub(crate) fn process_initial_selectable_map_list_request(
    mut initial_request: ResMut<SelectableMapListInitialRequestState>,
    catalog: Res<SelectableMapCatalog>,
    mut selectable_maps: ResMut<SelectableMapListState>,
) {
    if initial_request.requested {
        return;
    }

    relay_request_selectable_map_list(&catalog, &mut selectable_maps);
    initial_request.requested = true;
}

pub(crate) fn relay_request_selectable_map_list(
    catalog: &SelectableMapCatalog,
    selectable_maps: &mut SelectableMapListState,
) -> bool {
    let wire_packet = RequestSelectableMapListCommand.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(_request) = RequestSelectableMapListCommand::decode_payload(payload) else {
        return false;
    };

    let Some(packet) = GiveSelectableMapListPacket::new(catalog.names()) else {
        return false;
    };
    relay_give_selectable_map_list(selectable_maps, packet)
}

fn relay_give_selectable_map_list(
    selectable_maps: &mut SelectableMapListState,
    packet: GiveSelectableMapListPacket,
) -> bool {
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = GiveSelectableMapListPacket::decode_payload(payload) else {
        return false;
    };
    apply_selectable_map_list(selectable_maps, decoded_packet)
}

fn apply_selectable_map_list(
    selectable_maps: &mut SelectableMapListState,
    packet: GiveSelectableMapListPacket,
) -> bool {
    selectable_maps.maps = packet.maps;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_request_loads_generated_source_map_list() {
        let catalog = SelectableMapCatalog::embedded();
        let mut state = SelectableMapListState::default();

        assert!(relay_request_selectable_map_list(&catalog, &mut state));

        assert!(state.maps().contains(&"p02_bb_orig01.map".to_string()));
        assert_eq!(state.maps().len(), SOURCE_SELECTABLE_MAPS.len());
        assert!(state.maps().windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn give_selectable_map_list_replaces_existing_client_list() {
        let mut state = SelectableMapListState {
            maps: vec!["old.map".to_string()],
        };
        let packet = GiveSelectableMapListPacket::new(["a.map", "b.map"]).unwrap();

        assert!(relay_give_selectable_map_list(&mut state, packet));

        assert_eq!(state.maps(), &["a.map".to_string(), "b.map".to_string()]);
    }

    #[test]
    fn generated_selectable_maps_include_runtime_bytes() {
        let catalog = SelectableMapCatalog::embedded();
        let mut state = SelectableMapListState::default();
        assert!(relay_request_selectable_map_list(&catalog, &mut state));

        let index = state
            .maps()
            .iter()
            .position(|name| name == "p02_bb_orig01.map")
            .unwrap();
        let (name, bytes) = catalog.source_map(state.maps(), index).unwrap();

        assert_eq!(name, "p02_bb_orig01.map");
        assert_eq!(bytes, include_bytes!("../maps/p02_bb_orig01.map"));
    }

    #[test]
    fn sequential_rotation_advances_wraps_and_resolves_catalog_by_name() {
        let catalog = SelectableMapCatalog {
            entries: vec![
                SelectableMapCatalogEntry {
                    name: "p02_bb_orig01.map".to_string(),
                    bytes: Some(SelectableMapBytes::Embedded(include_bytes!(
                        "../maps/p02_bb_orig01.map"
                    ))),
                },
                SelectableMapCatalogEntry {
                    name: "p02_bb_orig03.map".to_string(),
                    bytes: Some(SelectableMapBytes::Embedded(include_bytes!(
                        "../maps/p02_bb_orig03.map"
                    ))),
                },
            ],
        };
        let mut rotation = MapRotationState {
            names: vec![
                "p02_bb_orig03.map".to_string(),
                "p02_bb_orig01.map".to_string(),
            ],
            native_bytes: vec![None, None],
            random: false,
            current_index: None,
        };

        let first = rotation.next_map(&catalog, |_| usize::MAX).unwrap();
        assert_eq!(first.0, "p02_bb_orig03.map");
        assert_eq!(first.1, include_bytes!("../maps/p02_bb_orig03.map"));
        let second = rotation.next_map(&catalog, |_| usize::MAX).unwrap();
        assert_eq!(second.0, "p02_bb_orig01.map");
        let wrapped = rotation.next_map(&catalog, |_| usize::MAX).unwrap();
        assert_eq!(wrapped.0, "p02_bb_orig03.map");
    }

    #[test]
    fn random_rotation_uses_source_modulo_index_without_sequential_increment() {
        let catalog = SelectableMapCatalog::embedded();
        let mut rotation = MapRotationState {
            names: vec![
                "p02_bb_orig01.map".to_string(),
                "p02_bb_orig03.map".to_string(),
            ],
            native_bytes: vec![None, None],
            random: true,
            current_index: None,
        };

        assert_eq!(
            rotation.next_map(&catalog, |_| 5).unwrap().0,
            "p02_bb_orig03.map"
        );
        assert_eq!(
            rotation.next_map(&catalog, |_| 0).unwrap().0,
            "p02_bb_orig01.map"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_rotation_file_reads_random_flag_order_and_owned_map_bytes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let first = root.join("maps/p02_bb_orig03.map");
        let second = root.join("maps/p02_bb_orig01.map");
        let list_path =
            std::env::temp_dir().join(format!("zod-map-rotation-{}.txt", std::process::id()));
        std::fs::write(
            &list_path,
            format!(
                "  +1random\r\n{}\n\n{}\r\n",
                first.display(),
                second.display()
            ),
        )
        .unwrap();

        let mut rotation = load_native_map_rotation(&list_path);
        assert!(rotation.random);
        assert_eq!(
            rotation.names,
            [first.to_string_lossy(), second.to_string_lossy()]
        );
        let catalog = SelectableMapCatalog::embedded();
        let selected = rotation.next_map(&catalog, |_| 1).unwrap();
        assert_eq!(selected.0, second.to_string_lossy());
        assert_eq!(selected.1, include_bytes!("../maps/p02_bb_orig01.map"));

        let _ = std::fs::remove_file(list_path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_source_list_preserves_order_names_and_runtime_bytes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let first = root.join("maps/p02_bb_orig03.map");
        let second = root.join("maps/p02_bb_orig01.map");
        let missing = root.join("maps/missing.map");
        let list_path =
            std::env::temp_dir().join(format!("zod-selectable-maps-{}.txt", std::process::id()));
        std::fs::write(
            &list_path,
            format!(
                "{}\n\n{}\r\n{}\n",
                first.display(),
                second.display(),
                missing.display()
            ),
        )
        .unwrap();

        let catalog = load_native_catalog_from_list_path(&list_path);
        let mut state = SelectableMapListState::default();
        assert!(relay_request_selectable_map_list(&catalog, &mut state));

        assert_eq!(
            state.maps(),
            &[
                first.to_string_lossy().to_string(),
                second.to_string_lossy().to_string(),
                missing.to_string_lossy().to_string(),
            ]
        );
        assert_eq!(
            catalog.source_map(state.maps(), 0).unwrap().1,
            include_bytes!("../maps/p02_bb_orig03.map")
        );
        assert!(catalog.source_map(state.maps(), 2).is_none());
        let _ = std::fs::remove_file(list_path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_source_folder_filters_map_extension_and_sorts_bare_names() {
        let catalog = load_native_catalog_from_folder(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("maps"),
        );
        let mut state = SelectableMapListState::default();
        assert!(relay_request_selectable_map_list(&catalog, &mut state));

        assert_eq!(
            state.maps().first().map(String::as_str),
            Some("p02_bb_orig01.map")
        );
        assert!(state.maps().windows(2).all(|pair| pair[0] <= pair[1]));
        assert_eq!(
            catalog.source_map(state.maps(), 0).unwrap().1,
            include_bytes!("../maps/p02_bb_orig01.map")
        );
    }
}
