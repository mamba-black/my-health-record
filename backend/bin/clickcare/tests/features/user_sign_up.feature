# language: es
Característica: Registro de usuario

  Escenario: Registro exitoso con un UUID v7 válido
    Dado un entorno activo con servicio gRPC de usuario y base de datos
    Cuando se envía una solicitud de registro con un UUID v7 válido
    Entonces la respuesta de registro es exitosa y el usuario se persiste en la base de datos

  Escenario: Registro fallido cuando el ID de usuario no es UUID v7
    Dado un entorno activo con servicio gRPC de usuario y base de datos
    Cuando se envía una solicitud de registro con un UUID v4 inválido
    Entonces la respuesta de registro devuelve un error indicando UUID v7 inválido
