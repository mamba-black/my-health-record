use async_trait::async_trait;

#[async_trait]
pub trait UseCase: Sync + Send {
    type Command;
    type Response;
    type Error;

    async fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error>;
}
