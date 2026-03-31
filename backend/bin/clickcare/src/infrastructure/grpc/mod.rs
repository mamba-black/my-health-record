pub(crate) mod patient_api_impl;
pub(crate) mod user_api_impl;


include!(concat!(env!("OUT_DIR"), "/grpc.rs"));
pub(super) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("api_descriptor");
