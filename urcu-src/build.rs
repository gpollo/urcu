#[cfg(feature = "static")]
fn configure_opt(config: &mut autotools::Config) {
    match std::env::var("OPT_LEVEL") {
        Err(_) => (),
        Ok(v) => {
            if matches!(v.as_str(), "0" | "1" | "2" | "3" | "s") {
                config.cflag(format!("-O{}", v));
                config.cxxflag(format!("-O{}", v));
            }
        }
    }

    config.cflag("-g");
    config.cxxflag("-g");
}

#[cfg(feature = "static")]
fn configure_lto(config: &mut autotools::Config) {
    let enable = match std::env::var("CARGO_ENCODED_RUSTFLAGS") {
        Err(_) => false,
        Ok(value) => value.contains("linker-plugin-lto"),
    };

    if enable {
        config.cflag("-flto");
        config.cxxflag("-flto");
    }
}

#[cfg(feature = "static")]
fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        println!("cargo::rustc-env=BUILD_DIR=");
        return;
    }

    use std::path::PathBuf;

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    println!("cargo::rustc-env=BUILD_DIR={}", out_dir.display());

    if out_dir.join("build").join("Makefile").is_file() {
        return;
    }

    let mut config = autotools::Config::new("vendor");
    configure_opt(&mut config);
    configure_lto(&mut config);
    config.out_dir(out_dir).reconf("-ivf").build();
}

#[cfg(not(feature = "static"))]
fn main() {
    println!("cargo::rustc-env=BUILD_DIR=");
}
