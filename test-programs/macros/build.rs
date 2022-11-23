use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use wit_component::ComponentEncoder;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mut components = Vec::new();

    println!("cargo:rerun-if-changed=../../");
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--release")
        .current_dir("../../")
        .arg("--target=wasm32-wasi")
        .env("CARGO_TARGET_DIR", &out_dir)
        .env("RUSTFLAGS", "-Clink-args=--import-memory")
        .env_remove("CARGO_ENCODED_RUSTFLAGS");
    let status = cmd.status().unwrap();
    assert!(status.success());

    let wasi_adapter = out_dir.join("wasm32-wasi/release/wasi_snapshot_preview1.wasm");
    println!("wasi adapter: {:?}", &wasi_adapter);

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .current_dir("..")
        .arg("--target=wasm32-wasi")
        .env("CARGO_TARGET_DIR", &out_dir)
        .env("CARGO_PROFILE_DEV_DEBUG", "1")
        .env_remove("CARGO_ENCODED_RUSTFLAGS");
    let status = cmd.status().unwrap();
    assert!(status.success());

    for file in out_dir.join("wasm32-wasi/debug").read_dir().unwrap() {
        let file = file.unwrap().path();
        if file.extension().and_then(|s| s.to_str()) != Some("wasm") {
            continue;
        }
        let stem = file.file_stem().unwrap().to_str().unwrap().to_string();

        let module = fs::read(&file).expect("read wasm module");
        let component = ComponentEncoder::default()
            .module(module.as_slice())
            .unwrap()
            .validate(true)
            .adapter_file(&wasi_adapter)
            .unwrap()
            .encode()
            .expect(&format!(
                "module {:?} can be translated to a component",
                file
            ));
        let component_path = out_dir.join(format!("{}.component.wasm", &stem));
        fs::write(&component_path, component).expect("write component to disk");
        components.push((stem, component_path));
    }

    let src = format!("const COMPONENTS: &[(&str, &str)] = &{:?};", components);
    std::fs::write(out_dir.join("components.rs"), src).unwrap();
}