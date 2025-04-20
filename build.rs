// extern crate prost_build;

// fn main() {
//     println!("test this");

//     prost_build::compile_protos(
//         &["src/cp_model.proto", "src/sat_parameters.proto"],
//         &["src/"],
//     )
//     .unwrap();

//     if std::env::var("DOCS_RS").is_err() {
//         let ortools_prefix = std::env::var("ORTOOLS_PREFIX")
//             .ok()
//             .unwrap_or_else(|| "e:/gicp/opt/ortools".into());
//         println!("##############{}", ortools_prefix);
//         cc::Build::new()
//             .cpp(true)
//             .flag("/std:c++17") // Use /std:c++20 or /std:c++17
//             .file("src/cp_sat_wrapper.cpp")
//             .include(&[&ortools_prefix, "/include"].concat())
//             .compile("cp_sat_wrapper.a");

//         println!("cargo:rustc-link-lib=dylib=ortools");
//         println!("cargo:rustc-link-search=native={}/lib", ortools_prefix);
//     }
// }
// build.rs

// ####################################
// set ORTOOLS_PREFIX=E:\gicp\opt\ortools; cargo build --release
// ####################################

use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Compile your .proto files
    prost_build::compile_protos(
        &["src/cp_model.proto", "src/sat_parameters.proto"],
        &["src/"],
    )?;

    // 2) If we're not on docs.rs, build the C++ wrapper:
    if env::var("DOCS_RS").is_err() {
        // Where did the user install OR-Tools?
        let prefix: PathBuf = env::var("ORTOOLS_PREFIX")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                if cfg!(target_os = "windows") {
                    // change this default if your Windows OR-Tools is elsewhere
                    PathBuf::from(r"E:\gicp\opt\ortools")
                } else {
                    PathBuf::from("/opt/ortools")
                }
            });

        // Re-run if the user changes this env var
        println!("cargo:rerun-if-env-changed=ORTOOLS_PREFIX");
        println!(
            "cargo:warning=Building C++ wrapper; ORTOOLS_PREFIX={}",
            prefix.display()
        );

        // Set up cc to compile your tiny cp_sat_wrapper.cpp
        let mut build = cc::Build::new();
        build
            .cpp(true)
            .file("src/cp_sat_wrapper.cpp")
            // point it at OR-Tools headers
            .include(prefix.join("include"));

        // Different flags for MSVC vs. others:
        let target = env::var("TARGET").unwrap();
        if target.contains("msvc") {
            // MSVC wants /std:c++17
            build.flag("/std:c++20");
        } else {
            // GCC/Clang wants -std=c++17
            build.flag("-std=c++20");
        }

        // This will produce either libcp_sat_wrapper.a (Unix) or cp_sat_wrapper.lib (Windows)
        build.compile("cp_sat_wrapper");

        // Link against our wrapper _and_ the real ortools shared lib:
        println!("cargo:rustc-link-lib=static=cp_sat_wrapper");
        println!("cargo:rustc-link-lib=dylib=ortools");
        println!(
            "cargo:rustc-link-search=native={}",
            prefix.join("lib").display()
        );
        // Link our wrapper _and_ the real OR‑Tools shared lib:
        println!("cargo:rustc-link-lib=static=cp_sat_wrapper");
        println!("cargo:rustc-link-lib=dylib=ortools");
        println!("cargo:rustc-link-search=native={}/lib", prefix.display());

        // Protobuf runtime is a static archive named libprotobuf.lib
        println!("cargo:rustc-link-lib=static=libprotobuf");
        // If you end up using the “lite” runtime instead, you can also add:
        // println!("cargo:rustc-link-lib=static=libprotobuf-lite");
    }

    Ok(())
}
