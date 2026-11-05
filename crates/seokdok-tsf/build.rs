//! Windows resources (icons, version info) and the export table. Everything
//! here is skipped for non-Windows targets.

fn main() {
    println!("cargo:rerun-if-changed=resources.rc");
    println!("cargo:rerun-if-changed=resources-eumdok.rc");
    println!("cargo:rerun-if-changed=seokdok.def");
    println!("cargo:rerun-if-changed=assets");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    windows_resources();
}

fn windows_resources() {
    use std::path::Path;

    // The profile icon is cosmetic: a missing resource compiler must not
    // block the build.
    let resources = if std::env::var_os("CARGO_FEATURE_EUMDOK").is_some() { "resources-eumdok.rc" } else { "resources.rc" };
    match embed_resource::compile(resources, embed_resource::NONE).manifest_optional() {
        Ok(()) => {}
        Err(e) => println!("cargo:warning=seokdok: icon resource embed skipped ({e})"),
    }

    // Point the MSVC linker at the .def file so the COM exports survive.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let def = Path::new(env!("CARGO_MANIFEST_DIR")).join("seokdok.def");
        println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def.display());
    }

    version_resource();
}

/// Embed a VERSIONINFO resource built from CARGO_PKG_VERSION.
fn version_resource() {
    let product = if std::env::var_os("CARGO_FEATURE_EUMDOK").is_some() { seokdok_product::EUMDOK } else { seokdok_product::SEOKDOK };
    let (id, hanja) = (product.id, product.hanja);
    let ver = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut n: Vec<&str> = ver.split('.').collect();
    while n.len() < 4 {
        n.push("0");
    }
    let fv = format!("{},{},{},{}", n[0], n[1], n[2], n[3]);
    let rc = format!(
        "1 VERSIONINFO\n\
         FILEVERSION {fv}\n\
         PRODUCTVERSION {fv}\n\
         FILEOS 0x40004L\n\
         FILETYPE 0x2L\n\
         BEGIN\n\
         \x20 BLOCK \"StringFileInfo\"\n\
         \x20 BEGIN\n\
         \x20   BLOCK \"040904b0\"\n\
         \x20   BEGIN\n\
         \x20     VALUE \"CompanyName\", \"mkpoli\"\n\
         \x20     VALUE \"FileDescription\", \"{id} - {hanja} input method for Windows (TSF)\"\n\
         \x20     VALUE \"FileVersion\", \"{ver}\"\n\
         \x20     VALUE \"InternalName\", \"{id}.dll\"\n\
         \x20     VALUE \"LegalCopyright\", \"MIT (c) mkpoli\"\n\
         \x20     VALUE \"OriginalFilename\", \"{id}.dll\"\n\
         \x20     VALUE \"ProductName\", \"{id}\"\n\
         \x20     VALUE \"ProductVersion\", \"{ver}\"\n\
         \x20   END\n\
         \x20 END\n\
         \x20 BLOCK \"VarFileInfo\"\n\
         \x20 BEGIN\n\
         \x20   VALUE \"Translation\", 0x409, 1200\n\
         \x20 END\n\
         END\n",
    );
    let out = std::env::var("OUT_DIR").expect("OUT_DIR");
    let path = std::path::Path::new(&out).join("version.rc");
    std::fs::write(&path, rc).expect("write version.rc");
    if let Err(e) = embed_resource::compile(&path, embed_resource::NONE).manifest_optional() {
        println!("cargo:warning=seokdok: version resource embed skipped ({e})");
    }
}
