# language: es
Característica: Registro progresivo de usuario y datos obligatorios para cita médica

  Escenario: Registro inicial con datos mínimos sin DNI
    Dado un usuario que inicia sesión o se registra con proveedor OAuth u correo
    Cuando envía sus datos iniciales de registro sin DNI ni teléfono
    Entonces la cuenta de usuario es creada activamente con perfil mínimo

  Escenario: Exigencia obligatoria de DNI y teléfono al agendar cita médica
    Dado un usuario registrado con perfil mínimo
    Cuando el usuario intenta agendar una cita médica en una clínica
    Entonces el sistema exige la captura obligatoria de DNI y teléfono según la Ley N° 30024
    Y al ingresar el DNI y teléfono la cita médica queda guardada en estado CONFIRMADA
