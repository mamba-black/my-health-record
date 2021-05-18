package medical.ui.component.organism

import com.raquo.laminar.api.L.*
import medical.domain.Patient
import medical.ui.component.atom.Loading
import medical.ui.component.molecule.{ PatientBasicInfo, TableBasic }
import scribe.*

import scala.scalajs.js.timers.setTimeout

object PatientSection {

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    info(s"Begin patientId: $patientId")

    // FIXME: Aqui colocar el servicio para traer la data si patient es None
    val _patient = patient.getOrElse(new Patient("Test", "Test", "Test", "Test"))
    val patientBasicInfo = Var(nameInput(_patient.name))
    val patientHistories = Var(Loading())

    setTimeout(1000) {
      val _ = patientBasicInfo.set(PatientBasicInfo(_patient))
    }
    setTimeout(1000) {
      val _ = patientHistories.set(TableBasic(None))
    }


    section(
      className := "container m-4 p-10 border-2 rounded-lg",
      div(
        child <-- patientBasicInfo,
      ),
      div(
        child <-- patientHistories,
      ),
    )
  }

  def nameInput(_name: String): HtmlElement = {
    div(
      label(forId := "name", "Nombre"),
      input(idAttr := "name", name := "name", readOnly := true, value := _name),
    )
  }

}
