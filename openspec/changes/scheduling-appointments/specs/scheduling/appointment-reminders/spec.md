## Purpose

Define el aviso que recibe el paciente antes de cada cita: cuándo se envía, por qué canal y qué pasa con él si la cita se cancela, se reprograma o el envío falla.

## ADDED Requirements

### Requirement: Recordatorio antes de la cita
Por cada cita confirmada, el sistema SHALL enviar al paciente un único recordatorio con una anticipación configurable (por defecto 24 horas). El recordatorio SHALL incluir fecha y hora en la zona horaria de la clínica, médico, clínica y dirección, y SHALL NOT incluir motivo de consulta ni datos clínicos. Si la cita se reserva con menos anticipación que la configurada, SHALL enviarse en el momento de la reserva.

#### Scenario: Cita reservada con días de anticipación
- **WHEN** un paciente reserva una cita para el jueves a las 10:00 con la anticipación por defecto
- **THEN** recibe un recordatorio el miércoles a las 10:00 con la fecha, la hora, el médico y la clínica

#### Scenario: Cita para dentro de 3 horas
- **WHEN** un paciente reserva una cita que empieza dentro de 3 horas
- **THEN** el recordatorio se envía de inmediato

### Requirement: Canal de envío
El canal por defecto SHALL ser el correo electrónico del paciente. Si el paciente no tiene correo, como un paciente registrado por la clínica sin cuenta, el sistema SHALL omitir el recordatorio y dejarlo registrado como «sin canal», sin fallar la reserva. Agregar otros canales, como SMS o WhatsApp, SHALL NOT cambiar las reglas de este requisito.

#### Scenario: Paciente con cuenta
- **WHEN** se envía el recordatorio de un paciente con cuenta de Google
- **THEN** llega al correo de su cuenta

#### Scenario: Paciente sin correo
- **WHEN** llega la hora del recordatorio de un paciente registrado por recepción sin correo
- **THEN** no se envía nada, el recordatorio queda como «sin canal» y la cita no cambia

### Requirement: Cancelar o reprogramar actualiza el recordatorio
Si una cita se cancela, su recordatorio pendiente SHALL anularse. Si se reprograma, SHALL programarse para el nuevo horario. El paciente SHALL NOT recibir un recordatorio de una cita cancelada ni con un horario que ya no es el de la cita.

#### Scenario: Cita cancelada
- **WHEN** un paciente cancela una cita cuyo recordatorio aún no se envió
- **THEN** el recordatorio no se envía

#### Scenario: Cita reprogramada
- **WHEN** una cita del jueves se reprograma para el viernes a la misma hora
- **THEN** el recordatorio se envía el jueves con la hora del viernes, y no se envía el anterior

### Requirement: Envío confiable, antes repetido que perdido
Si el envío falla, el sistema SHALL reintentarlo un número limitado de veces y SHALL registrar el resultado final: enviado, fallido o sin canal. Ante la duda de si un recordatorio salió, por ejemplo si el proceso se cae a mitad del envío, el sistema SHALL volver a enviarlo: un recordatorio repetido es aceptable, uno perdido no. Un recordatorio ya registrado como enviado SHALL NOT reenviarse.

#### Scenario: Falla temporal del correo
- **WHEN** el primer intento de envío falla y el segundo funciona
- **THEN** el paciente recibe un solo recordatorio y queda registrado como enviado

#### Scenario: Aviso entregado dos veces
- **WHEN** la cola entrega dos veces el aviso de un recordatorio que ya se envió
- **THEN** la segunda entrega no envía nada y el recordatorio sigue registrado como enviado

#### Scenario: Caída a mitad del envío
- **WHEN** el servidor se cae después de enviar un recordatorio y antes de registrarlo como enviado
- **THEN** al volver, el recordatorio se envía de nuevo y el paciente puede recibirlo dos veces

#### Scenario: Reintentos agotados
- **WHEN** el envío falla en todos los intentos
- **THEN** el recordatorio queda registrado como fallido y la cita no cambia
