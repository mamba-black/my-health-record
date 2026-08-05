# language: es
Característica: Caso de Uso 1 - Registro progresivo de usuario y datos obligatorios para cita médica

  Esquema del escenario: Registro exitoso de usuario con diferentes grados de completitud de perfil
    Dado un entorno activo con servicio gRPC de usuario y base de datos
    Cuando se envía una solicitud de registro con ID "<id>", email "<email>", nombre "<nombre>", primer apellido "<primer_apellido>", segundo apellido "<segundo_apellido>", DNI "<dni>", teléfono "<telefono>", fecha nacimiento "<fecha_nacimiento>" y crear clínica "<crear_clinica>"
    Entonces la respuesta de registro es exitosa y el usuario con nombre "<nombre>" se persiste en la base de datos

    Ejemplos:
      | id   | email                     | nombre | primer_apellido | segundo_apellido | dni      | telefono  | fecha_nacimiento | crear_clinica | caso_de_uso                                   |
      | auto | juan.minimo@example.com   | Juan   | -               | -                | -        | -         | -                | false         | 1. Perfil Mínimo OIDC (Solo nombre y correo)  |
      | auto | maria.hispana@example.com | María  | Pérez           | Gómez            | -        | -         | -                | false         | 2. Perfil Hispanos (2 apellidos sin DNI)      |
      | auto | carlos.dni@example.com    | Carlos | López           | Torres           | 77778888 | 999888777 | 1990-05-15       | false         | 3. Perfil Completo Ley 30024 (DNI + Teléfono) |
      | auto | ana.doctora@example.com   | Ana    | Ramírez         | Salazar          | 10000002 | 911222333 | 1985-11-20       | true          | 4. Registro con Inicialización de Clínica     |
      | auto | carlos.dni@example.com    | Carlos |                 | Torres           | 77778888 | 999888777 | 1990-05-15       | false         | 5. Perfil con segundo apellido                |

  Escenario: Registro fallido cuando el ID de usuario no es UUID v7
    Dado un entorno activo con servicio gRPC de usuario y base de datos
    Cuando se envía una solicitud de registro con un UUID v4 inválido "52a92673-207b-4fcb-a2d5-c36fae4edf6a" y email "invalido@example.com"
    Entonces la respuesta de registro devuelve un error indicando "no es un UUID V7"

  Escenario: Exigencia obligatoria de DNI y teléfono al agendar cita médica
    Dado un usuario registrado con perfil mínimo email "maria.gomez@example.com"
    Cuando el usuario intenta agendar una cita médica ingresando DNI "77778888" y teléfono "999888777"
    Entonces el sistema exige la captura obligatoria de DNI y teléfono según la Ley N° 30024
    Y al ingresar el DNI y teléfono la cita médica queda guardada en estado "CONFIRMADA"
