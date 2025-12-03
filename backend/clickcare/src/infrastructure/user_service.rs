use core::str;

use fake::Fake;
use fake::faker::chrono::raw::*;
use fake::faker::internet::raw::*;
use fake::faker::name::raw::*;
use fake::faker::phone_number::raw::PhoneNumber;
use fake::locales::EN;
use tonic::*;
use ulid::Ulid;
use crate::infrastructure::api::patient_service_server::*;
use crate::infrastructure::api::*;

#[derive(Default)]
pub struct UserServiceImpl {}

impl UserService for UserServiceImpl {

}
