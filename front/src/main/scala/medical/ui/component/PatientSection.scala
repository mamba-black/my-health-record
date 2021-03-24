package medical.ui.component

import com.raquo.laminar.api.L._
import medical.domain.Patient
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
      label(forId := "name", "Nombre"),
      input(idAttr := "name", name := "name", readOnly := true, value := patient.name),
      label(forId := "age", "Edad"),
      input(idAttr := "age", name := "age", readOnly := true, value := "80"),
      label(forId := "email", "Correo"),
      input(idAttr := "email", name := "email", readOnly := true, value := "email@gmail.com"),
      label(forId := "phone", "Telefono"),
      input(idAttr := "phone", name := "phone", readOnly := true, value := "+51999999999"),
    )
  }
}
