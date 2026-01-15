use crate::domain::user::User;
use app_core::domain::repository::emitter::Emitter;
use tokio::sync::broadcast::Sender;


pub(crate) struct EmitterImpl {
    pub(crate) sender: Sender<User>
}

impl Emitter<User> for EmitterImpl {

    fn emit(&self, event: &User) -> Result<(), String> {
        // FIXME: Guardar en la base de datos
        let _ = self.sender.send(event.clone());
        Ok(())
    }
}
