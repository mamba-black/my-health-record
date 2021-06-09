package medical.infrastructure.ui.organism

import com.raquo.laminar.api.L._
import medical.domain.Patient
import medical.infrastructure.ui.molecule.{ PatientBasicInfo, TableBasic }
import scribe._

import scala.scalajs.js.timers.setTimeout

object PatientSection {

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    info(s"Begin patientId: $patientId")

    val patientBus = new EventBus[Option[Patient]]

    if (patient.isEmpty) {
      // FIXME: Aqui colocar el servicio para traer la data si patient es None
      // FIXME: Quitar, esto solo es paro demo
      setTimeout(2000) {
        patientBus.writer.onNext(Some(new Patient("Test", "Test", "Test", "Test")))
        info("TIMEOUT")
      }
    }


    section(
      className := "container m-4 p-10 border-2 rounded-lg",
      div(
        PatientBasicInfo(patientBus.events.toSignal(patient.orElse(None))),
      ),
      div(
        cls := "mt-5",
        TableBasic(List("Fecha", "Nota"), None),
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
