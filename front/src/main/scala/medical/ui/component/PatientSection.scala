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
      inputLabel("name", "Nombre", patient.name),
      inputLabel("age", "Edad", "80"),
      inputLabel("email", "Correo", "email@gmail.com"),
      inputLabel("phone", "Telefono", "+51 555 555 555"),
    )
  }

  def inputLabel(_name: String, description: String, _value: String = null): HtmlElement = {
    span(
      label(forId := _name, description, cls := "mx-2"),
      input(idAttr := "name", name := _name, readOnly := true, value := Option(_value).orNull,
        cls := "mx-2 px-3 py-2 placeholder-gray-300 border border-gray-300 rounded-md focus:outline-none focus:ring focus:ring-indigo-100 focus:border-indigo-300")
    )
  }
}
