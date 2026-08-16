pub mod clinic_api_impl;
pub mod patient_api_impl;
pub mod user_api_impl;

// Submódulo aislado donde Clippy no entrará a fiscalizar
#[allow(clippy::all, clippy::pedantic, clippy::restriction)]
pub mod grpc_gen {
    include!(concat!(env!("OUT_DIR"), "/grpc.rs"));
}

// Re-exportas todo lo generado para no romper tus imports en el resto del proyecto
pub use grpc_gen::*;

pub(super) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("api_descriptor");
