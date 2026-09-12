# language: es
Característica: Caso de Uso 1 - Registro progresivo de usuario y datos obligatorios para cita médica

    Escenario: Intento fallido de reserva de cita por falta de DNI y teléfono requeridos por Ley N° 30024
        Dado un usuario registrado con perfil mínimo email "maria.gomez@example.com" sin DNI ni teléfono
        Cuando el usuario intenta agendar una cita médica sin proporcionar DNI ni teléfono
        Entonces el sistema rechaza la reserva exigiendo la captura obligatoria de DNI y teléfono según la Ley N° 30024

    Escenario: Reserva exitosa de cita médica al completar progresivamente el DNI y teléfono
        Dado un usuario registrado con perfil mínimo email "maria.gomez@example.com"
        Cuando el usuario completa su perfil ingresando DNI "77778888" y teléfono "999888777" al agendar una cita médica
        Entonces la reserva de la cita médica se procesa exitosamente y queda guardada en estado "CONFIRMADA"
