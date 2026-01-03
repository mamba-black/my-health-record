pub(crate) mod usecase;
pub(crate) mod create_user_usecase;

pub(crate) trait UseCase {
    type Command;
    type Response;
    type Error;

    fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error>;
}


