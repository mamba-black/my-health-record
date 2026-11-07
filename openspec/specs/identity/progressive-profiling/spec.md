# identity/progressive-profiling Specification

## Purpose

Define qué datos personales y profesionales necesita cada rol, cómo se le presentan en su perfil y en qué acción concreta se vuelve obligatorio cada uno, para pedir solo lo necesario y en el momento en que se necesita.

> **Validación legal pendiente**: las normas citadas en este documento (Ley N° 30024, Ley N° 29733, NTS N° 139-MINSA, Ley N° 15173 del CMP y normas de SUSALUD) son la base propuesta y deben ser revisadas por un asesor legal o médico antes de tomarse como oficiales.

## Requirements

### Requirement: Catálogo de datos por rol
El sistema SHALL mantener un catálogo único de datos de perfil que indica, para cada dato, a qué roles aplica y qué acciones lo vuelven obligatorio. El formulario, el aviso y las validaciones SHALL usar solo ese catálogo. Ningún dato es obligatorio solo por figurar en él. La tabla completa está en `design.md`.

#### Scenario: Un dato no se exige si ninguna acción en curso lo requiere
- **WHEN** un paciente guarda su perfil sin DNI ni teléfono y no está intentando ninguna acción
- **THEN** el sistema guarda los datos ingresados y no rechaza el guardado por los datos faltantes

#### Scenario: Un dato que no aplica al rol no se pide
- **WHEN** un paciente sin membresía como médico ni como dueño abre su perfil
- **THEN** el formulario no muestra los campos de colegiatura, especialidad, RUC ni IPRESS

### Requirement: Datos comunes a todos los roles
Para todo rol, el catálogo SHALL incluir nombres, apellido paterno, apellido materno, correo, teléfono y documento de identidad (tipo DNI, carné de extranjería o pasaporte, y número). El correo SHALL provenir del proveedor de identidad y no ser editable. El apellido materno SHALL ser siempre opcional.

#### Scenario: Correo no editable
- **WHEN** un usuario abre su perfil
- **THEN** el correo aparece como solo lectura

#### Scenario: Extranjero sin DNI
- **WHEN** un paciente extranjero registra su carné de extranjería en lugar de DNI
- **THEN** el sistema lo acepta como documento válido para las acciones que exigen documento

### Requirement: Datos del paciente
Para el rol paciente, el catálogo SHALL incluir además fecha de nacimiento, sexo y dirección. Confirmar una cita SHALL exigir nombres, apellido paterno, documento y teléfono. Abrir su historia clínica en una atención SHALL exigir además fecha de nacimiento y sexo. La dirección SHALL ser siempre opcional.

#### Scenario: Confirmar cita
- **WHEN** un paciente con nombres, apellido paterno y teléfono, pero sin documento, confirma una cita
- **THEN** el sistema rechaza la acción indicando que falta el documento

#### Scenario: Primera atención
- **WHEN** un médico abre la primera atención de un paciente sin fecha de nacimiento ni sexo registrados
- **THEN** el sistema pide esos dos datos antes de crear la historia clínica, y pueden registrarlos el paciente o la clínica en admisión

### Requirement: Datos del médico
Para el rol médico, el catálogo SHALL incluir además colegio profesional (CMP, COP, CEP, CPsP u otro), número de colegiatura, especialidad y número de RNE. Activar el perfil médico SHALL exigir nombres, apellido paterno, teléfono, documento, colegio y número de colegiatura. La especialidad y el RNE SHALL ser opcionales.

#### Scenario: Activar sin colegiatura
- **WHEN** un médico invitado con documento y teléfono, pero sin colegiatura, activa su perfil
- **THEN** el sistema rechaza la acción indicando que faltan el colegio profesional y el número de colegiatura

#### Scenario: Especialidad opcional
- **WHEN** un médico activa su perfil con todos los datos exigidos y sin especialidad
- **THEN** la activación procede

### Requirement: Datos del dueño o administrador
Para el rol dueño, el catálogo SHALL incluir además RUC, razón social, nombre comercial y código IPRESS de la clínica. Crear una clínica SHALL exigir solo nombres; activarla SHALL exigir nombres, apellido paterno, teléfono y documento del dueño, más RUC y razón social. La exigencia del código IPRESS queda pendiente de validación legal.

#### Scenario: Crear clínica sin RUC
- **WHEN** un usuario con nombres registrados crea una clínica sin RUC
- **THEN** la clínica se crea en estado pendiente de activación

#### Scenario: Activar clínica sin RUC
- **WHEN** el dueño intenta activar la clínica sin RUC ni razón social
- **THEN** el sistema rechaza la acción indicando que faltan RUC y razón social

### Requirement: Formulario de perfil según rol
El formulario de perfil SHALL mostrar todos los datos del catálogo que aplican a los roles del usuario (su rol preferido y los de sus membresías), agrupados por propósito, y SHALL indicar junto a cada dato la acción para la que será obligatorio.

#### Scenario: Médico que también es paciente
- **WHEN** un usuario con rol preferido de paciente y una membresía de médico abre su perfil
- **THEN** el formulario muestra los datos de paciente y los de médico, y junto a la colegiatura indica «Obligatorio para activar tu perfil médico»

#### Scenario: Datos opcionales siempre
- **WHEN** un usuario con rol de paciente y de médico abre su perfil
- **THEN** el apellido materno, la dirección y la especialidad aparecen marcados como opcionales, sin acción asociada

### Requirement: Exigencia de datos al intentar una acción
Cuando el usuario intenta una acción protegida y le faltan datos que esa acción exige, el servidor SHALL rechazarla sin efectos y SHALL devolver la acción y la lista exacta de datos faltantes. Ningún dato SHALL exigirse fuera de una acción que lo requiera.

#### Scenario: Paciente sin DNI intenta confirmar una cita
- **WHEN** un paciente sin documento ni teléfono confirma una cita
- **THEN** el servidor no crea la cita y responde que la acción «confirmar cita» requiere documento y teléfono

### Requirement: Ventana «Completa tus datos»
Ante un rechazo por datos faltantes, el frontend SHALL abrir sobre la pantalla actual una ventana titulada «Completa tus datos» que nombre la acción, contenga un formulario solo con los datos que faltan, marcados como obligatorios, y ofrezca dos opciones: «Completar» y «Rechazar», además del acceso al formulario completo. La ventana SHALL conservar el progreso de la acción mientras esté abierta.

#### Scenario: Contenido de la ventana
- **WHEN** a un paciente sin documento ni teléfono se le rechaza la confirmación de una cita
- **THEN** ve la ventana «Completa tus datos» con el texto «Para confirmar tu cita necesitamos estos datos», solo los campos documento y teléfono, y los botones «Completar» y «Rechazar»

#### Scenario: Datos ya registrados no se piden
- **WHEN** a un paciente con teléfono pero sin documento se le rechaza la confirmación de una cita
- **THEN** la ventana muestra solo el campo documento

#### Scenario: Validación dentro de la ventana
- **WHEN** el paciente pulsa «Completar» con un DNI de 7 dígitos
- **THEN** la ventana sigue abierta, marca el error en el campo y no reintenta la acción

### Requirement: Acceso al formulario completo desde la ventana
La ventana «Completa tus datos» SHALL ofrecer la opción «Editar todos mis datos», que muestra en la misma ventana el formulario completo del perfil, con todos los datos de los roles del usuario, sean o no obligatorios. Los datos que exige la acción SHALL seguir marcados como obligatorios, la acción pendiente SHALL conservarse y «Completar» y «Rechazar» SHALL comportarse igual que en la vista reducida.

#### Scenario: Pasar al formulario completo
- **WHEN** un paciente al que le faltan documento y teléfono para confirmar una cita pulsa «Editar todos mis datos»
- **THEN** la ventana muestra todos sus datos de perfil, con documento y teléfono marcados como obligatorios para confirmar la cita y el resto como opcionales

#### Scenario: Lo escrito no se pierde al cambiar de vista
- **WHEN** el paciente escribió su teléfono en la vista reducida y luego pulsa «Editar todos mis datos»
- **THEN** el formulario completo muestra el teléfono que ya había escrito

#### Scenario: Editar datos opcionales y continuar
- **WHEN** en el formulario completo el paciente completa documento y teléfono, corrige su dirección y pulsa «Completar»
- **THEN** se guardan todos los cambios, incluida la dirección, y la cita se confirma

#### Scenario: Faltan los obligatorios en el formulario completo
- **WHEN** en el formulario completo el paciente edita solo datos opcionales y pulsa «Completar» sin documento
- **THEN** la ventana no se cierra, marca el documento como obligatorio y no reintenta la acción

#### Scenario: Volver a la vista reducida
- **WHEN** en el formulario completo el paciente pulsa «Ver solo lo necesario»
- **THEN** la ventana vuelve a mostrar solo los datos que faltan, sin perder lo escrito

### Requirement: Completar y continuar la acción
Al pulsar «Completar» con datos válidos, el sistema SHALL guardar los datos en el perfil del usuario y SHALL reintentar automáticamente la acción original, sin que el usuario vuelva a ingresar lo que ya había elegido.

#### Scenario: Completar y continuar
- **WHEN** el paciente completa documento y teléfono válidos y pulsa «Completar»
- **THEN** los datos quedan guardados en su perfil, la ventana se cierra y la cita se confirma con el médico y horario que había elegido

### Requirement: Rechazar con advertencia de pérdida
Al pulsar «Rechazar», si la ventana contiene algún dato obligatorio para la acción, el frontend SHALL advertir que sin esos datos no podrá continuar con la acción y que perderá lo ingresado, y SHALL pedir confirmación con las opciones «Volver a completar» y «Salir sin continuar». Si el usuario confirma la salida, la acción SHALL quedar sin ejecutar y SHALL descartarse lo ingresado en la ventana y en la acción.

#### Scenario: Advertencia al rechazar
- **WHEN** el paciente pulsa «Rechazar» en la ventana de la confirmación de cita
- **THEN** ve el aviso «Sin tu documento y teléfono no podrás confirmar la cita. Si sales, perderás los datos que ingresaste» con las opciones «Volver a completar» y «Salir sin continuar»

#### Scenario: Volver a completar
- **WHEN** el paciente elige «Volver a completar»
- **THEN** regresa a la ventana con los datos que ya había escrito en ella

#### Scenario: Salir sin continuar
- **WHEN** el paciente elige «Salir sin continuar»
- **THEN** la ventana se cierra, la cita no se confirma, no se guarda ningún dato de la ventana, se descarta la selección de médico y horario y el paciente queda en la pantalla donde inició la acción

#### Scenario: Cierre de la ventana por otros medios
- **WHEN** el paciente cierra la ventana con la tecla Escape o haciendo clic fuera de ella
- **THEN** el frontend lo trata como «Rechazar» y muestra la misma advertencia

### Requirement: Recordatorio al iniciar sesión
Al iniciar sesión, el sistema SHALL informar los datos que le faltan al usuario para las acciones habituales de sus roles, y SHALL permitirle ignorar el aviso y usar la aplicación. El aviso no SHALL bloquear ninguna acción que no requiera esos datos.

#### Scenario: Aviso ignorable
- **WHEN** un paciente sin documento inicia sesión
- **THEN** ve un aviso «Te falta tu documento, lo necesitarás para confirmar citas» con las opciones «Completar ahora» e «Ignorar», y al ignorar entra a la aplicación

#### Scenario: Sin datos faltantes no hay aviso
- **WHEN** un usuario con todos los datos obligatorios de sus roles inicia sesión
- **THEN** entra directamente, sin aviso

### Requirement: Formato y validación de los datos
El sistema SHALL validar cada dato que se ingresa, aunque no sea obligatorio en ese momento: el DNI tiene 8 dígitos; el RUC tiene 11 dígitos y dígito verificador válido (módulo 11); la fecha de nacimiento no es futura; el número de documento no pertenece a otra cuenta de la misma red; y el número de colegiatura no está vacío y va acompañado de su colegio profesional.

#### Scenario: Dato opcional mal formado
- **WHEN** un usuario guarda su perfil con un DNI de 7 dígitos
- **THEN** el sistema rechaza el guardado e indica que el DNI debe tener 8 dígitos, aunque el DNI no sea obligatorio en ese momento

#### Scenario: RUC con dígito verificador inválido
- **WHEN** un dueño ingresa un RUC de 11 dígitos con dígito verificador incorrecto
- **THEN** el sistema rechaza el dato e indica que el RUC no es válido

### Requirement: Datos verificados no editables por el usuario
Una vez que el documento de identidad o la colegiatura quedan verificados (verificación presencial o por la clínica), el usuario SHALL NOT poder modificarlos desde su perfil; el formulario SHALL mostrarlos como de solo lectura e indicar que el cambio se solicita en la clínica. Los datos no verificados SHALL poder editarse libremente.

#### Scenario: DNI verificado
- **WHEN** un usuario con DNI verificado abre su perfil
- **THEN** el DNI aparece como solo lectura con el texto «Para cambiarlo, acércate a recepción»

#### Scenario: DNI no verificado
- **WHEN** un usuario con DNI aún no verificado lo corrige en su perfil
- **THEN** el sistema guarda el nuevo DNI
