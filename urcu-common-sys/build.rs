use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let build_config = urcu_src::build_config();

    build_config.cargo_link("urcu-common");
    build_config
        .default_bindgen()
        .header("src/header.h")
        .opaque_type("pthread.*")
        .allowlist_item("__cds_wfcq.*")
        .allowlist_item("__cds_wfq.*")
        .allowlist_item("__cds_wfs.*")
        .allowlist_item("cds_wfcq.*")
        .allowlist_item("cds_wfq.*")
        .allowlist_item("cds_wfs.*")
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/header.h");
}
