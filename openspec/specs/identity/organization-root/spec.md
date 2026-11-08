# identity/organization-root Specification

## Purpose

Define la organización de cada dueño según FHIR R4: una `Organization` raíz, su empresa o grupo, de la que cuelgan todas sus clínicas. La raíz es la unidad de aislamiento («red clínica»): ninguna regla, búsqueda ni invitación cruza de una raíz a otra, también en desarrollo local.

## Requirements

### Requirement: Una Organization raíz por dueño
Cada dueño SHALL tener una única Organization raíz. SHALL crearse al registrarse como dueño o, a más tardar, al crear su primera clínica, y SHALL NOT depender de la red del enlace por el que se registró ni de una red por defecto. Dueños distintos SHALL NOT compartir raíz.

#### Scenario: Dos dueños
- **WHEN** dos personas se registran como dueñas desde el mismo sitio, sin enlace de clínica
- **THEN** cada una tiene su propia Organization raíz

#### Scenario: Dueño con dos clínicas
- **WHEN** un dueño crea una segunda clínica
- **THEN** la clínica cuelga de la misma raíz que la primera

### Requirement: Clínicas con partOf
Cada clínica SHALL ser una Organization cuyo `partOf` apunta a la raíz de su dueño, y su red SHALL derivarse siempre de esa raíz. El cliente SHALL NOT poder elegir la red de una clínica.

#### Scenario: El cliente envía otra red
- **WHEN** un dueño crea una clínica indicando un `network_id` cualquiera
- **THEN** el sistema lo ignora y la clínica cuelga de la raíz del dueño

### Requirement: Vínculos de la persona
La cuenta del dueño SHALL vincularse por `Person.link` con su raíz y con cada una de sus clínicas. El personal SHALL vincularse a cada clínica por su membresía (FHIR `PractitionerRole`).

#### Scenario: Clínica nueva
- **WHEN** un dueño crea una clínica
- **THEN** su persona queda vinculada con la raíz y con la clínica nueva

### Requirement: Cuentas sin mover entre redes
El sistema SHALL NOT mover una cuenta de una red a otra. La cuenta de quien se registra como dueño SHALL nacer en su raíz; la de quien entra por el enlace de una clínica, en la raíz de esa clínica.

#### Scenario: Personal registrado por el enlace
- **WHEN** una persona se registra por el enlace de una clínica
- **THEN** su cuenta queda en la raíz de esa clínica, donde la encuentran las invitaciones y solicitudes de esa organización

### Requirement: Inicio de sesión sin enlace
Al iniciar sesión sin el enlace de una clínica, el sistema SHALL encontrar la cuenta del correo en cualquier raíz si es una sola, y SHALL pedir el enlace de la clínica si el correo tiene cuentas en varias organizaciones. Encontrar la cuenta SHALL NOT mezclar datos entre organizaciones.

#### Scenario: Dueño entra desde el sitio general
- **WHEN** un dueño inicia sesión sin enlace de clínica
- **THEN** entra en la cuenta de su organización

#### Scenario: Correo en dos organizaciones
- **WHEN** un médico con cuentas en dos organizaciones inicia sesión sin enlace
- **THEN** el servidor le pide entrar por el enlace de su clínica
