// Módulo del agregado con el mismo nombre que su carpeta: el archivo de entidad vive dentro de `domain/<agregado>/`.
#[allow(clippy::module_inception)]
pub mod practitioner;
pub mod practitioner_repository;

pub use practitioner::*;
