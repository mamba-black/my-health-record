package medical.infrastructure.ui.molecule

import com.raquo.laminar.api.L._
import medical.domain.Patient
import medical.infrastructure.ui.atom.{ Button, InputLabel }

object PatientBasicInfo {

  def apply(patient: Patient): HtmlElement = {

    val readOnly = Var[Boolean](true)
    div(
      h1("Informacion del paciente",
        cls := ("text-2xl", "py-8"),
      ),
      div(
        cls := "grid grid-cols-3 gap-4",
        InputLabel("name", "Nombre", patient.name, readOnly.signal),
        InputLabel("name", "Apellido paterno", patient.paternalSurname, readOnly.signal),
        InputLabel("name", "Apellido materno", patient.maternalSurname, readOnly.signal),
        InputLabel("age", "Edad", "80", readOnly.signal),
        InputLabel("email", "Correo", "email@gmail.com", readOnly.signal),
        InputLabel("phone", "Telefono", "+51 555 555 555", readOnly.signal),
        InputLabel("allergies", "Alergias", "", readOnly.signal),
        div(cls := "col-start-3 flex justify-end",
          Button(readOnly),
        ),
      )
    )
  }

}
