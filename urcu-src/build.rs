#[cfg(feature = "static")]
fn configure_lto(config: &mut autotools::Config) {
    let enable = match std::env::var("CARGO_ENCODED_RUSTFLAGS") {
        Ok(value) => value.contains("linker-plugin-lto"),
        Err(_) => false,
    };

    if enable {
        config.cflag("-flto");
        config.cxxflag("-flto");
    }
}

#[cfg(feature = "static")]
fn main() {
    use std::path::PathBuf;

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    println!("cargo::rustc-env=BUILD_DIR={}", out_dir.display());

    if out_dir.join("build").join("Makefile").is_file() {
        return;
    }

    let mut config = autotools::Config::new("vendor");
    configure_lto(&mut config);
    config.out_dir(out_dir).reconf("-ivf").build();
}

#[cfg(not(feature = "static"))]
fn main() {
    println!("cargo::rustc-env=BUILD_DIR=");
}
