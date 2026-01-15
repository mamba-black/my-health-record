pub trait Emitter<T: Clone> {
    fn emit(&self, event: &T) -> Result<(), String>;
}
