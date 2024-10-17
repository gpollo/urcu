use std::path::PathBuf;

use bindgen::callbacks::{FieldInfo, ParseCallbacks};
use bindgen::FieldVisibilityKind;

#[derive(Debug)]
struct BindgenCallbacks;

impl ParseCallbacks for BindgenCallbacks {
    fn field_visibility(&self, info: FieldInfo<'_>) -> Option<FieldVisibilityKind> {
        match info.type_name {
            "urcu_gp_poll_state" | "rcu_head" => Some(FieldVisibilityKind::Private),
            _ => None,
        }
    }
}

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let build_config = urcu_src::build_config();

    if std::env::var("DOCS_RS").is_err() {
        metadeps::probe().unwrap();
    }

    build_config.cargo_link("urcu-cds");
    build_config.cargo_link("urcu");
    build_config
        .default_bindgen()
        .header("src/header.h")
        .allowlist_item("cds_.*")
        .allowlist_item("__cds_.*")
        .allowlist_item("rcu_.*")
        .allowlist_item("urcu_gp_poll_state")
        .allowlist_var("CDS_.*")
        .parse_callbacks(Box::new(BindgenCallbacks))
        .derive_default(true)
        .wrap_static_fns(true)
        .wrap_static_fns_path(out_dir.join("static_fns.c"))
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    build_config
        .default_cc()
        .include(env!("CARGO_MANIFEST_DIR"))
        .file(out_dir.join("static_fns.c"))
        .compile("static_fns");

    println!("cargo:rerun-if-changed=src/header.h");
}
