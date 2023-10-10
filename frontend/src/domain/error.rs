use crate::domain::error::AppError::Grpc;
use leptos_router::ParamsError;
use std::error::Error;
use tonic::Status;

#[derive(Debug)]
pub enum AppError {
    GenericError(String),
    Grpc(String),
}

impl From<Status> for AppError {
    fn from(value: Status) -> Self {
        Grpc(value.message().to_string())
    }
}
