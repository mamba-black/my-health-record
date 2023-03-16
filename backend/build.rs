fn main() {
    let protobuf_dir = "protobuf";

    tonic_build::configure()
        .build_server(true)
        .include_file("backend_api.rs")
        .compile(
            &[
                format!("{}/{}", protobuf_dir, "health.proto"),
            ],
            &[""],
        )
        .unwrap_or_else(|e| panic!("Protobuf compile error: {:?}", e));
}