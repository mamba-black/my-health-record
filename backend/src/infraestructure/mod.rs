pub mod api {
    tonic::include_proto!("api");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("api_descriptor");
}
