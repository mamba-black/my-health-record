
pub mod backend_api {
    tonic::include_proto!("backend_api");

    pub use health::backend::api::v1 as api;
}