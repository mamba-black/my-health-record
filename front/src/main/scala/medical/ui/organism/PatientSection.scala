package medical.ui.organism

import com.raquo.laminar.api.L.*
import medical.domain.Patient
import medical.ui.molecule.{ PatientBasicInfo, TableBasic }
import scribe.*

import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter

object PatientSection {
  private val fmt = DateTimeFormatter.ISO_LOCAL_DATE
  // private val fmt = DateTimeFormatter.ofPattern("MMMM MM d EE EEEE yyyy G", new Locale("es", "ES"))

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    debug(s"Begin patientId: $patientId")
    val css = "@apply "

    section(
      className := "container m-4 p-10 border-2 rounded-lg",
      div(PatientBasicInfo(patientId, patient)),
      div(cls := "mt-5", TableBasic(List("Fecha", "Anamnesis", "Diagnóstico", "Tratamiento"), Some(test))),
    )
  }

  private def nameInput(_name: String): HtmlElement = {
    div(label(forId := "name", "Nombre"), input(idAttr := "name", name := "name", readOnly := true, value := _name))
  }

  private val test = {
    val now: String = ZonedDateTime.now().asInstanceOf[ZonedDateTime].format(fmt).asInstanceOf[String]

    List(
      tbody(
        tr(
          cls := "border-none ",
          td(cls := "px-6 pt-4", now),
          td(cls := "px-6 pt-4", "Dolor de cabeza"),
          td(cls := "px-6 pt-4", "Gripe comun"),
          td(cls := "px-6 pt-4", "Tomar paracetamol"),
        ),
        tr(cls := "border-none ", td(cls := "px-6 pb-4", span("uno"), span("dos"), colSpan(4))),
      ),
      tbody(
        tr(
          cls := "border-none",
          td(cls := "px-6 pt-4", now),
          td(cls := "px-6 pt-4", "Dolor de cabeza"),
          td(cls := "px-6 pt-4", "Gripe comun"),
          td(cls := "px-6 pt-4", "Tomar paracetamol"),
        ),
        tr(cls := "border-none", td(cls := "px-6 pb-4", span("dos"), span("tres"))),
      ),
      tbody(
        tr(
          cls := "border-none",
          td(cls := "px-6 pt-4", now),
          td(cls := "px-6 pt-4", "Dolor de cabeza"),
          td(cls := "px-6 pt-4", "Gripe comun"),
          td(cls := "px-6 pt-4", "Tomar paracetamol"),
        ),
        tr(cls := "border-none", td(cls := "px-6 pb-4", span("tres"), span("cuatro"), colSpan(4))),
      ),
      tbody(
        tr(
          cls := "border-none",
          td(cls := "px-6 pt-4", now),
          td(cls := "px-6 pt-4", "Dolor de cabeza"),
          td(cls := "px-6 pt-4", "Gripe comun"),
          td(cls := "px-6 pt-4", "Tomar paracetamol"),
        ),
        tr(cls := "border-none", td(cls := "px-6 pb-4", span("cuatro"), span("cinco"))),
      ),
    )
  }
}
