use async_trait::async_trait;

#[async_trait]
pub trait Emitter<T: Clone> {
    async fn emit(&self, event: &T) -> Result<(), String>;
}
