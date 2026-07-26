use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=maps");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let maps_dir = manifest_dir.join("maps");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("out dir"));
    let out_file = out_dir.join("selectable_maps.rs");

    let mut maps = Vec::new();
    if let Ok(entries) = fs::read_dir(&maps_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("map"))
                && let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str())
            {
                println!("cargo:rerun-if-changed={}", path.display());
                maps.push(file_name.to_string());
            }
        }
    }
    maps.sort();

    let mut generated = String::from("pub(crate) const SOURCE_SELECTABLE_MAPS: &[&str] = &[\n");
    for map in &maps {
        generated.push_str("    ");
        generated.push_str(&format!("{map:?}"));
        generated.push_str(",\n");
    }
    generated.push_str("];\n");
    generated.push_str(
        "pub(crate) fn source_selectable_map_bytes(name: &str) -> Option<&'static [u8]> {\n    match name {\n",
    );
    for map in maps {
        generated.push_str(&format!(
            "        {map:?} => Some(include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/maps/{map}\"))),\n"
        ));
    }
    generated.push_str("        _ => None,\n    }\n}\n");

    fs::write(out_file, generated).expect("write generated selectable maps");
}
