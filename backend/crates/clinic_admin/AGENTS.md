# Administración de Clínica (`crates/clinic_admin`)

## Estado del Crate

⚠️ **Andamiaje sin implementar.** Es miembro del workspace y compila, pero `src/lib.rs`
contiene únicamente la plantilla `add()` generada por `cargo new --lib`. No existe
`src/domain/`, ni `src/application/`, ni `src/infrastructure/`.

**Solapamiento pendiente de resolver**: el rol de administrador de clínica hoy se expresa
mediante `User.is_owner` en [crates/user](../user/AGENTS.md) y mediante el
`PersonLinkTarget::Organization` de `Person.link`. Antes de escribir código aquí hay que
decidir qué agrega este contexto acotado sobre lo que ya modelan `user` y `administration`.

---

## Alcance previsto

* **Responsabilidad**: Gestión del rol de administrador de una clínica — altas de personal, permisos operativos y configuración de la organización.
* **Regla vigente que le aplica**: el DNI del administrador se vuelve obligatorio al activar o crear la `Organization`, o al configurar facturación / RUC. Ver el registro progresivo en [crates/user/AGENTS.md](../user/AGENTS.md).
