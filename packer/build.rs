use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../unpacker/src/main.rs");

    let compile_args = ["build", "-p", "unpacker", "--release"];

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = PathBuf::from("../target/unpacker-build/release/unpacker");

    let status = Command::new("cargo")
        .args(compile_args)
        .env("CARGO_TARGET_DIR", "../target/unpacker-build")
        .status()
        .expect("Failed to build unpacker");

    assert!(status.success(), "Unpacker build failure");

    let unpacker = out_dir.join("unpacker");
    fs::copy(&out_file, &unpacker).expect("Failed to move unpacker into another dirrectory");

    let status = Command::new("cargo")
        .args(compile_args)
        .args(["-F", "decrypt"])
        .env("CARGO_TARGET_DIR", "../target/unpacker-build")
        .status()
        .expect("Failed to build unpacker");

    assert!(status.success(), "Unpacker build failure");

    let decrypt_unpacker = out_dir.join("decrypt_unpacker");
    fs::copy(&out_file, &decrypt_unpacker)
        .expect("Failed to move unpacker into another dirrectory");

    fs::write(
        "src/include_bin.rs",
        format!(
            r#"
const UNPACKER: &[u8] = include_bytes!(r"{}");
const DECRYPT_UNPACKER: &[u8] = include_bytes!(r"{}");
"#,
            unpacker.display(),
            decrypt_unpacker.display()
        ),
    )
    .expect("Failed to write include file");
}
