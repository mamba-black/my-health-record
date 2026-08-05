# language: es
Característica: Caso de Uso 1 - Registro progresivo de usuario y datos obligatorios para cita médica

  Escenario: Registro exitoso con un UUID v7 válido
    Dado un entorno activo con servicio gRPC de usuario y base de datos
    Cuando se envía una solicitud de registro con ID "auto", email "juan.perez@example.com" y nombre "Juan"
    Entonces la respuesta de registro es exitosa y el usuario con nombre "Juan" se persiste en la base de datos

  Escenario: Registro fallido cuando el ID de usuario no es UUID v7
    Dado un entorno activo con servicio gRPC de usuario y base de datos
    Cuando se envía una solicitud de registro con un UUID v4 inválido "52a92673-207b-4fcb-a2d5-c36fae4edf6a" y email "invalido@example.com"
    Entonces la respuesta de registro devuelve un error indicando "no es un UUID V7"

  Escenario: Exigencia obligatoria de DNI y teléfono al agendar cita médica
    Dado un usuario registrado con perfil mínimo email "maria.gomez@example.com"
    Cuando el usuario intenta agendar una cita médica ingresando DNI "77778888" y teléfono "999888777"
    Entonces el sistema exige la captura obligatoria de DNI y teléfono según la Ley N° 30024
    Y al ingresar el DNI y teléfono la cita médica queda guardada en estado "CONFIRMADA"
