// Módulo del agregado con el mismo nombre que su carpeta: el archivo de entidad vive dentro de `domain/<agregado>/`.
#[allow(clippy::module_inception)]
pub mod membership;
pub mod membership_repository;

pub use membership::*;
