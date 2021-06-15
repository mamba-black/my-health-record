package medical.ui.organism

import com.raquo.laminar.api.L._
import medical.domain.Patient
import medical.ui.molecule.{ PatientBasicInfo, TableBasic }
import scribe._

object PatientSection {

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    debug(s"Begin patientId: $patientId")

    section(
      className := "container m-4 p-10 border-2 rounded-lg",
      div(
        PatientBasicInfo(patientId, patient),
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
