use std::path::PathBuf;

fn main() {
    metadeps::probe().unwrap();

    let output = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rustc-link-lib=urcu-memb");
    println!("cargo:rerun-if-changed=src/header.h");
    bindgen::Builder::default()
        .header("src/header.h")
        .blocklist_type("rcu_flavor_struct")
        .blocklist_type("rcu_head")
        .blocklist_type("urcu_atfork")
        .blocklist_type("urcu_gp_poll_state")
        .allowlist_function("urcu_memb_.*")
        .generate()
        .unwrap()
        .write_to_file(output.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
