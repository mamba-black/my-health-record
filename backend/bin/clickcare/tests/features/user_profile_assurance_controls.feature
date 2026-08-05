# language: es
Característica: Controles de edición de datos de perfil según nivel de certeza

  Escenario: Edición libre de datos en cuenta no verificada Level1
    Dado un usuario con nivel de certeza Level1 No Verificado
    Cuando solicita actualizar su nombre o DNI desde la aplicación
    Entonces el sistema actualiza directamente los datos de la persona sin restricciones

  Escenario: Restricción de edición directa de DNI en cuenta verificada Level3 o Level4
    Dado un usuario verificado presencialmente con nivel de certeza Level3 o Level4
    Cuando intenta modificar su DNI desde la aplicación móvil
    Entonces el sistema bloquea la edición directa y requiere solicitar actualización presencial en clínica
