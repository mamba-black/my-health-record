use std::{env, io};
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path(out_dir.join("api_descriptor.bin"))
        .compile(&["api.proto"], &["../proto"])
}
