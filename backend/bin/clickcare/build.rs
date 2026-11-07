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
        .compile_protos(&["api.proto"], &["../../../proto/"])?;

    // Declarar un `rerun-if-changed` desactiva la detección por defecto de Cargo, así
    // que hay que listar todo lo que se lee: el contrato y las migraciones que
    // `embed_migrations!` embebe al compilar.
    println!("cargo:rerun-if-changed=../../../proto/api.proto");
    println!("cargo:rerun-if-changed=../../migrations");

    Ok(())
}
