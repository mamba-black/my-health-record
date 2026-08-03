use crate::domain::user::User;
use app_core::domain::repository::emitter::Emitter;
use async_trait::async_trait;
use tokio::sync::broadcast::Sender;

#[derive(Debug)]
pub(crate) struct EmitterImpl {
    pub(crate) sender: Sender<User>,
}

#[async_trait]
impl Emitter<User> for EmitterImpl {
    #[tracing::instrument(name = "call_external_api", fiels(otel.kind = "client", peer.service = "api-user"))]
    async fn emit(&self, event: &User) -> Result<(), String> {
        // FIXME: Guardar en la base de datos
        let _ = self.sender.send(event.clone());
        Ok(())
    }
}
