use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let build_config = urcu_src::build_config();

    if std::env::var("DOCS_RS").is_err() {
        metadeps::probe().unwrap();
    }

    build_config.cargo_link("urcu-cds");
    build_config
        .default_bindgen()
        .header("src/header.h")
        .opaque_type("pthread.*")
        .blocklist_item("rcu.*")
        .allowlist_item("__cds.*")
        .allowlist_item("_cds.*")
        .allowlist_item("cds.*")
        .allowlist_item("CDS.*")
        .allowlist_var("CDS.*")
        .wrap_static_fns(true)
        .wrap_static_fns_path(out_dir.join("static_fns.c"))
        .derive_default(true)
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
