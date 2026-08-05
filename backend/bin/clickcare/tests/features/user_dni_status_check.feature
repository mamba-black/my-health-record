# language: es
Característica: Consulta previa de estado de DNI para guía dinámica de registro

  Escenario: Consulta de DNI no registrado en el sistema
    Dado un cliente que consulta el estado de un DNI no existente en la base de datos
    Cuando solicita la verificación previa a la API de usuarios
    Entonces la API responde que el DNI está DISPONIBLE para registro normal

  Escenario: Consulta de DNI asociado a cuenta activa verificada
    Dado un cliente que consulta un DNI ya vinculado a una cuenta activa y verificada
    Cuando solicita la verificación previa a la API
    Entonces la API responde con un estado de CONFLICTO sugiriendo iniciar sesión o recuperar cuenta

  Escenario: Consulta de DNI con historia clínica existente sin cuenta verificada
    Dado un cliente que consulta un DNI presente en historias clínicas pero sin cuenta verificada
    Cuando solicita la verificación previa a la API
    Entonces la API responde que existe historial disponible y sugiere solicitar vinculación presencial
