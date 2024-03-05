use std::process::Command;

use std::path::PathBuf;

static PROTOBUF_FILE: &str = "ProtobufDevice_0000E007.proto";
static PROTOBUF_DIR: &str = "src/protobuf/proto";

fn gen_protobuf() {
    let mut protofile = PathBuf::from(PROTOBUF_DIR);
    protofile.push(PROTOBUF_FILE);

    prost_build::compile_protos(&[protofile], &[PROTOBUF_DIR]).unwrap();
}

fn restore_client_clibs_libman() {
    // execute libman restore in src/bin/clicker-data-collector-server
    let output = Command::new("libman")
        .arg("restore")
        .current_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/bin/clicker-data-collector-server"
        ))
        .output()
        .expect("Failed to execute 'libman restore'");

    if !output.status.success() {
        println!(
            "cargo:warning=libman restore failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    println!("cargo:rerun-if-changed=src/bin/clicker-data-collector-server/libman.json")
}

fn main() {
    restore_client_clibs_libman();
    gen_protobuf();
}
