use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let xml_path = manifest_dir.join("../assets/LogMessages.xml");
    println!("cargo:rerun-if-changed={}", xml_path.display());
    let generated = ullam_ardupilot_log_codegen::generate_from_path(&xml_path)
        .unwrap_or_else(|error| panic!("failed to generate ArduPilot log bindings: {error}"));
    let output_path =
        PathBuf::from(env::var_os("OUT_DIR").expect("build output directory")).join("messages.rs");
    fs::write(output_path, generated).expect("write generated ArduPilot bindings");
}
