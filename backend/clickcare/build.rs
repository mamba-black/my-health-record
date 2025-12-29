// use std::path::PathBuf;
// use std::env;

use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // tonic_prost_build::configure()
    //     .build_server(true)
    //     .build_client(false)
    //     .file_descriptor_set_path(out_dir.join("api_descriptor.bin"))
    //     .compile_protos(&["api.proto"], &["../proto"])
    //     .unwrap();

    // tonic_prost_build::compile_protos("../proto/api.proto")?;
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("api_descriptor.bin"))
        .compile_protos(&["api.proto"], &["../proto/"])?;

    Ok(())
}
