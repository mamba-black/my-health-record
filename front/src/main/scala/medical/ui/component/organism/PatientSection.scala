package medical.ui.component.organism

import com.raquo.laminar.api.L._
import medical.domain.Patient
import medical.ui.component.molecule.PatientBasicInfo
import wvlet.log.LogSupport

import scala.scalajs.js.timers.setTimeout

object PatientSection extends LogSupport {

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    info(s"Begin patientId: $patientId")

    // FIXME: Aqui colocar el servicio para traer la data si patient es None
    val _patient = patient.getOrElse(new Patient("Test", "Test", "Test", "Test"))
    val _child = Var(loading(_patient.name))
    setTimeout(1000) {
      val _ = _child.set(PatientBasicInfo(_patient))
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

}
