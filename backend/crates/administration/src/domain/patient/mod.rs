// Módulo del agregado con el mismo nombre que su carpeta: el archivo de entidad vive dentro de `domain/<agregado>/`.
#[allow(clippy::module_inception)]
pub mod patient;
pub mod patient_link_repository;
pub mod patient_repository;

pub use patient::*;
