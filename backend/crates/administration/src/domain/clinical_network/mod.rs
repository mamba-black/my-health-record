// Módulo del agregado con el mismo nombre que su carpeta: el archivo de entidad vive dentro de `domain/<agregado>/`.
#[allow(clippy::module_inception)]
pub mod clinical_network;
pub mod clinical_network_repository;

pub use clinical_network::*;
