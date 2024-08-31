use std::path::PathBuf;

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
