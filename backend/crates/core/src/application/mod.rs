pub trait UseCase {
    type Command;
    type Response;
    type Error;

    fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error>;
}

