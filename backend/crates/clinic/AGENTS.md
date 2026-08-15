# Clínica (`crates/clinic`)

## Estado del Crate

⚠️ **Andamiaje sin implementar.** Es miembro del workspace y compila, pero `src/lib.rs`
contiene únicamente la plantilla `add()` generada por `cargo new --lib`. No existe
`src/domain/`, ni `src/application/`, ni `src/infrastructure/`.

**Solapamiento pendiente de resolver**: la entidad `Organization` (la clínica como entidad
legal) **ya está implementada** en [crates/administration](../administration/AGENTS.md)
(`src/domain/organization.rs`), y es la que crea el worker al procesar `UserCreatedEvent`.
Antes de escribir código aquí hay que decidir si este crate absorbe ese agregado o si se
elimina en favor de `administration`.

---

## Alcance previsto

* **Responsabilidad**: La clínica como entidad legal y operativa — datos fiscales (RUC), sedes y configuración.
* **Recurso FHIR de referencia**: `Organization` (HL7 FHIR R4).
