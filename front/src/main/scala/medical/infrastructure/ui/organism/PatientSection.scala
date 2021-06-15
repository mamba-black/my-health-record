package medical.infrastructure.ui.organism

import com.raquo.laminar.api.L._
import medical.domain.{ ContactPoint, HumanName, Patient, SystemContactPoint }
import medical.infrastructure.ui.molecule.{ PatientBasicInfo, TableBasic }
import scribe._

import java.time.LocalDate
import scala.scalajs.js.timers.setTimeout

object PatientSection {

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    debug(s"Begin patientId: $patientId")

    val patientBus = new EventBus[Option[Patient]]

    if (patient.isEmpty) {
      // FIXME: Aqui colocar el servicio para traer la data si patient es None
      // FIXME: Quitar, esto solo es paro demo
      setTimeout(2000) {
        val patient = new Patient(
          "Test",
          new HumanName("Malpica", "Gallegos", Seq("Hector", "Miuler")),
          true,
          LocalDate.of(1979,10, 13),
          Seq(new ContactPoint(SystemContactPoint.PHONE, "993990103")),
        )
        patientBus.writer.onNext(Some(patient))
        debug("TIMEOUT")
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
