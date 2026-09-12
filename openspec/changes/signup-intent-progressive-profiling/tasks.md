## 1. Contrato Protobuf y Eventos

- [x] 1.1 Agregar el enum `SignUpIntent` y el campo `intent = 18` en `backend/proto/api.proto`, verificando la compilación con `cargo check -p clickcare`.
- [x] 1.2 Definir el evento de dominio `FounderRegistered { user_id: Uuid, person_id: Uuid, organization_id: Uuid }` en `crates/core/src/domain/events.rs`.

## 2. Aplicación y Dominio en crates/user

- [x] 2.1 Actualizar `SignUpCommand` en `crates/user` para recibir `SignUpIntent`.
- [x] 2.2 Modificar el caso de uso `SignUpUseCase` para generar anticipadamente `organization_id = Uuid::now_v7()` y emitir `FounderRegistered` condicionalmente si la intención es `INTENT_CLINIC_OWNER` (o fallback `create_clinic == true`).
- [x] 2.3 Agregar pruebas unitarias para el caso de uso cubriendo los escenarios de `INTENT_CLINIC_OWNER`, `INTENT_PRACTITIONER` e `INTENT_PATIENT`.

## 3. Adaptador gRPC y Verificación de Integración

- [x] 3.1 Mapear el campo `intent` en `UserApiImpl` (`bin/clickcare/src/infrastructure/grpc/user_api_impl.rs`) hacia el comando de aplicación.
- [x] 3.2 Verificar que toda la suite del workspace compile limpiamente con `cargo clippy --workspace -- -D warnings`.
