pub trait BuildConfig {
    fn cargo_link(&self, lib: &'static str);

    fn configure_bindgen(&self, builder: bindgen::Builder) -> bindgen::Builder;

    fn configure_cc<'a>(&'a self, builder: &'a mut cc::Build) -> &'a mut cc::Build;

    fn default_bindgen(&self) -> bindgen::Builder {
        self.configure_bindgen(bindgen::Builder::default())
    }

    fn default_cc(&self) -> cc::Build {
        let mut builder = cc::Build::default();
        self.configure_cc(&mut builder);
        builder
    }
}

#[cfg(feature = "static")]
mod static_linking {
    use std::path::PathBuf;
    use std::str::FromStr;

    use super::*;

    pub struct StaticBuildConfig {
        include_dir: PathBuf,
    }

    impl StaticBuildConfig {
        pub fn new() -> Self {
            let build_dir = PathBuf::from_str(env!("BUILD_DIR")).unwrap();
            let lib_dir = build_dir.join("lib");
            let pkgconfig_dir = lib_dir.join("pkgconfig");

            std::env::set_var("PKG_CONFIG_PATH", pkgconfig_dir);
            println!("cargo:rustc-link-search=native={}", lib_dir.display());

            Self {
                include_dir: build_dir.join("include"),
            }
        }
    }

    impl BuildConfig for StaticBuildConfig {
        fn cargo_link(&self, lib: &'static str) {
            println!("cargo:rustc-link-lib=static={}", lib);
        }

        fn configure_bindgen(&self, builder: bindgen::Builder) -> bindgen::Builder {
            builder.clang_arg(format!("-I{}", self.include_dir.display()))
        }

        fn configure_cc<'a>(&'a self, builder: &'a mut cc::Build) -> &'a mut cc::Build {
            builder.include(self.include_dir.clone())
        }
    }
}

#[cfg(not(feature = "static"))]
mod dynamic_linking {
    use super::*;

    pub struct DynamicBuildConfig;

    impl BuildConfig for DynamicBuildConfig {
        fn cargo_link(&self, lib: &'static str) {
            println!("cargo:rustc-link-lib={}", lib);
        }

        fn configure_bindgen(&self, builder: bindgen::Builder) -> bindgen::Builder {
            builder
        }

        fn configure_cc<'a>(&'a self, builder: &'a mut cc::Build) -> &'a mut cc::Build {
            builder
        }
    }
}

pub fn build_config() -> Box<dyn BuildConfig> {
    #[cfg(feature = "static")]
    return Box::new(static_linking::StaticBuildConfig::new());

    #[cfg(not(feature = "static"))]
    return Box::new(dynamic_linking::DynamicBuildConfig);
}