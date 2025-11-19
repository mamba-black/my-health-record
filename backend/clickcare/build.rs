use std::path::PathBuf;
use std::env;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path(out_dir.join("api_descriptor.bin"))
        .compile_protos(&["api.proto"], &["../proto"])
        .unwrap();
}
