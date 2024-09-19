use std::path::PathBuf;

use bindgen::callbacks::{FieldInfo, ParseCallbacks};
use bindgen::FieldVisibilityKind;

#[derive(Debug)]
struct CustomCallbacks;

impl ParseCallbacks for CustomCallbacks {
    fn field_visibility(&self, info: FieldInfo<'_>) -> Option<FieldVisibilityKind> {
        match info.type_name {
            "urcu_gp_poll_state" | "rcu_head" => Some(FieldVisibilityKind::Private),
            _ => None,
        }
    }
}

fn main() {
    metadeps::probe().unwrap();

    let output = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let package = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    println!("cargo:rustc-link-lib=urcu");
    println!("cargo:rerun-if-changed=src/header.h");
    bindgen::Builder::default()
        .header("src/header.h")
        .allowlist_item("cds_.*")
        .allowlist_item("rcu_.*")
        .allowlist_item("urcu_gp_poll_state")
        .allowlist_var("CDS_.*")
        .parse_callbacks(Box::new(CustomCallbacks))
        .derive_default(true)
        .wrap_static_fns(true)
        .wrap_static_fns_path(output.join("static_fns.c"))
        .generate()
        .unwrap()
        .write_to_file(output.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .include(package)
        .file(output.join("static_fns.c"))
        .compile("foo");
}
