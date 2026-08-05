# language: es
Característica: Corrección de DNI registrado por error y conversión a perfil dependiente

  Escenario: Conversión de DNI de familiar en perfil gestionado e ingreso de DNI titular
    Dado un usuario titular que registró por error el DNI de un familiar en su cuenta
    Cuando solicita mover el DNI del familiar a un perfil dependiente de Paciente
    Entonces se crea un perfil de Paciente gestionado vinculado al titular
    Y al ingresar el DNI real del titular su identidad principal queda actualizada limpiamente
