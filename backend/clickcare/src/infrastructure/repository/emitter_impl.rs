use tokio::sync::broadcast::Sender;
use crate::domain::user::User;
use crate::infrastructure::repository::emitter::Emitter;


pub struct EmitterImpl {
    pub(crate) sender: Sender<User>
}

impl Emitter<User> for EmitterImpl {

    fn emit(&self, event: &User) -> Result<(), String> {
        // FIXME: Guardar en la base de datos
        self.sender.send(event.clone());
        Ok(())
    }
}
