package medical.infrastructure.ui.organism

import com.raquo.laminar.api.L._
import medical.domain.Patient
import medical.infrastructure.ui.atom.Loading
import medical.infrastructure.ui.molecule.{ PatientBasicInfo, TableBasic }
import scribe._

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
      val _ = patientHistories.set(TableBasic(List("Fecha", "Nota"), None))
    }


    section(
      className := "container m-4 p-10 border-2 rounded-lg",
      div(
        child <-- patientBasicInfo,
      ),
      div(
        cls := "mt-5",
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
