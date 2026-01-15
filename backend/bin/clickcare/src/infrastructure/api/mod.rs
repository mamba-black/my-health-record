pub(crate) mod patient_service;
pub(crate) mod user_service;


include!(concat!(env!("OUT_DIR"), "/api.rs"));
pub(super) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("api_descriptor");
