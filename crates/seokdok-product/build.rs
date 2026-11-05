//! Turns `products/*.json` into Rust constants, so a front end's GUIDs and
//! names can be used in `const` items.

use serde_json::Value;
use std::fmt::Write;
use std::path::Path;

fn text(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_else(|| panic!("products: {key} is missing")).escape_debug().to_string()
}

fn list(v: &Value, key: &str) -> String {
    let items = v[key].as_array().unwrap_or_else(|| panic!("products: {key} is missing"));
    let quoted: Vec<String> = items.iter().map(|i| format!("\"{}\"", i.as_str().expect("a list of strings").escape_debug())).collect();
    format!("&[{}]", quoted.join(", "))
}

fn guid(v: &Value, key: &str) -> String {
    format!("0x{}", text(v, key).replace('-', ""))
}

fn constant(out: &mut String, name: &str, file: &str) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../products");
    let path = dir.join(file);
    println!("cargo:rerun-if-changed={}", path.display());
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let p: Value = serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let w = &p["windows"];
    writeln!(
        out,
        "pub const {name}: Product = Product {{
    id: \"{id}\",
    hanja: \"{hanja}\",
    hangul: \"{hangul}\",
    latin: \"{latin}\",
    glyph: \"{glyph}\",
    config_dir: \"{config_dir}\",
    conversion_label: \"{conversion_label}\",
    menu_conversion: \"{menu_conversion}\",
    scripts: {scripts},
    default_script: \"{default_script}\",
    hidden_settings: {hidden},
    windows: Windows {{
        clsid: {clsid},
        profile: {profile},
        da_input: {da_input},
        da_converted: {da_converted},
        da_focus: {da_focus},
        settings_exe: \"{settings_exe}\",
        service_name: \"{service_name}\",
        candidate_class: \"{candidate_class}\",
        display_name: \"{display_name}\",
    }},
    json: {raw:?},
}};",
        id = text(&p, "id"),
        hanja = text(&p, "hanja"),
        hangul = text(&p, "hangul"),
        latin = text(&p, "latin"),
        glyph = text(&p, "glyph"),
        config_dir = text(&p, "config_dir"),
        conversion_label = text(&p, "conversion_label"),
        menu_conversion = text(&p, "menu_conversion"),
        scripts = list(&p, "scripts"),
        default_script = text(&p, "default_script"),
        hidden = list(&p, "hidden_settings"),
        clsid = guid(w, "clsid"),
        profile = guid(w, "profile"),
        da_input = guid(w, "da_input"),
        da_converted = guid(w, "da_converted"),
        da_focus = guid(w, "da_focus"),
        settings_exe = text(w, "settings_exe"),
        service_name = text(w, "service_name"),
        candidate_class = text(w, "candidate_class"),
        display_name = text(w, "display_name"),
    )
    .unwrap();
}

fn main() {
    let mut out = String::new();
    constant(&mut out, "SEOKDOK", "seokdok.json");
    constant(&mut out, "EUMDOK", "eumdok.json");
    let dest = Path::new(&std::env::var("OUT_DIR").unwrap()).join("products.rs");
    std::fs::write(dest, out).unwrap();
}
