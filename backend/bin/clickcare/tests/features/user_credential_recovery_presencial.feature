# language: es
Característica: Recuperación de credenciales y revinculación presencial en clínica

  Escenario: Registro con nuevo correo cuando existe historial previo de DNI
    Dado un usuario que perdió acceso a sus credenciales anteriores
    Cuando registra una nueva cuenta ingresando su DNI existente y solicita vinculación presencial
    Entonces la cuenta es creada exitosamente con un enlace de identidad en nivel de certeza Level1 Pendiente
    Y la reserva de su cita médica queda 100% CONFIRMADA en la agenda

  Escenario: Aprobación y elevación de certeza durante check-in presencial en recepción
    Dado un paciente con vinculación de DNI en nivel Level1 Pendiente que llega a su cita
    Cuando la recepcionista verifica el DNI físico y aprueba la vinculación en el sistema
    Entonces el nivel de certeza del enlace se eleva a Level3 o Level4 Verificado
    Y la cuenta anterior queda desactivada transfiriendo el acceso al historial clínico
