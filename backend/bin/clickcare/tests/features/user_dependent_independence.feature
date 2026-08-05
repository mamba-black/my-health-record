# language: es
Característica: Independización de perfil dependiente por mayoría de edad

  Escenario: Registro de cuenta autónoma por dependiente que cumple mayoría de edad
    Dado un hijo registrado previamente como dependiente en la cuenta de sus padres
    Cuando registra una nueva cuenta autónoma con su correo y DNI al cumplir 18 años
    Entonces se crea la cuenta con estado de vinculación Level1 Pendiente

  Escenario: Verificación presencial y transferencia completa de historial clínico
    Dado el nuevo usuario adulto con cuenta autónoma en estado pendiente
    Cuando realiza la verificación presencial con su DNI físico en la clínica
    Entonces el sistema eleva la certeza a Level3 o Level4 Verificado
    Y desvincula el perfil dependiente transfiendole la propiedad total de su historia clínica
