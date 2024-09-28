use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let build_config = urcu_src::build_config();
    metadeps::probe().unwrap();

    build_config.cargo_link("urcu-memb");
    build_config
        .default_bindgen()
        .header("src/header.h")
        .blocklist_type("rcu_flavor_struct")
        .blocklist_type("rcu_head")
        .blocklist_type("urcu_atfork")
        .blocklist_type("urcu_gp_poll_state")
        .allowlist_function("urcu_memb_.*")
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/header.h");
}
