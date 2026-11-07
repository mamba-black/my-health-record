# Administración de Clínica (`crates/clinic_admin`)

## Estado del Crate

⚠️ **Andamiaje sin implementar.** Es miembro del workspace y compila, pero `src/lib.rs`
contiene únicamente la plantilla `add()` generada por `cargo new --lib`. No existe
`src/domain/`, ni `src/application/`, ni `src/infrastructure/`.

**Solapamiento resuelto**: el rol de dueño o administrador de una clínica es una
**membresía** (`administration.membership`, rol `OWNER` o `ADMIN`) gestionada por
[crates/administration](../administration/AGENTS.md). `User.is_owner` quedó obsoleto: ya
no concede permisos y se eliminará cuando nada lo lea. Este crate solo tendrá sentido si
la administración de clínica crece más allá de invitar, confirmar colegiaturas, suspender
y activar la clínica, que hoy viven en `MembershipUseCases`.

---

## Alcance previsto

* **Responsabilidad**: Gestión del rol de administrador de una clínica — altas de personal, permisos operativos y configuración de la organización.
* **Regla vigente que le aplica**: activar la clínica exige al dueño nombres, apellido paterno, teléfono y documento, y a la clínica RUC válido (módulo 11) y razón social. Fuente oficial: [`access-control/action-authorization`](../../../openspec/specs/access-control/action-authorization/spec.md).
