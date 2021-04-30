package medical.ui.component

import com.raquo.laminar.api.L._
import medical.domain.Patient
import medical.ui.component.atoms.InputLabel
import wvlet.log.LogSupport

import scala.scalajs.js.timers.setTimeout

object PatientSection extends LogSupport {

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    info(s"Begin patientId: $patientId")

    val _child = Var(loading(patient.get.name))
    setTimeout(1000) {
      val _ = _child.set(basicInfo(patient.get))
    }

    section(
      className := "container m-4 p-10 border-2 rounded-lg",
      div(
        child <-- _child,
      ),
    )
  }

  def loading(_name: String): HtmlElement = {
    div(
      label(forId := "name", "Nombre"),
      input(idAttr := "name", name := "name", readOnly := true, value := _name),
    )
  }

  def basicInfo(patient: Patient): HtmlElement = {
    div(
      cls := "grid grid-cols-3 gap-4",
      InputLabel("name", "Nombre", patient.name),
      InputLabel("name", "Apellido paterno", patient.name),
      InputLabel("name", "Apellido materno", patient.name),
      InputLabel("age", "Edad", "80"),
      InputLabel("email", "Correo", "email@gmail.com"),
      InputLabel("phone", "Telefono", "+51 555 555 555"),
    )
  }

}
