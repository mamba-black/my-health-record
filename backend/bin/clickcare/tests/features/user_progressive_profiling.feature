# language: es
Característica: Caso de Uso 1 - Registro progresivo de usuario y datos obligatorios para cita médica

  Escenario: Registro exitoso con un UUID v7 válido
    Dado un entorno activo con servicio gRPC de usuario y base de datos
    Cuando se envía una solicitud de registro con un UUID v7 válido
    Entonces la respuesta de registro es exitosa y el usuario se persiste en la base de datos

  Escenario: Registro fallido cuando el ID de usuario no es UUID v7
    Dado un entorno activo con servicio gRPC de usuario y base de datos
    Cuando se envía una solicitud de registro con un UUID v4 inválido
    Entonces la respuesta de registro devuelve un error indicando UUID v7 inválido

  Escenario: Exigencia obligatoria de DNI y teléfono al agendar cita médica
    Dado un usuario registrado con perfil mínimo
    Cuando el usuario intenta agendar una cita médica en una clínica
    Entonces el sistema exige la captura obligatoria de DNI y teléfono según la Ley N° 30024
    Y al ingresar el DNI y teléfono la cita médica queda guardada en estado CONFIRMADA
