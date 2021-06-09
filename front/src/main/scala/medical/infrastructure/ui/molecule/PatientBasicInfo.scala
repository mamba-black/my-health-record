package medical.infrastructure.ui.molecule

import com.raquo.laminar.api.L._
import medical.domain.Patient
import medical.infrastructure.ui.atom.{ Button, InputLabel }
import org.scalajs.dom
import org.scalajs.dom.raw.HTMLInputElement
import scribe._

object PatientBasicInfo {

  def apply(patient: Signal[Option[Patient]]): HtmlElement = {

    val readOnly = Var[Boolean](true)
    div(
      h1("Informacion del paciente",
        cls := ("text-2xl", "py-8"),
      ),
      form(
        inContext(thisNode => onSubmit --> Observer[dom.Event](e => {
          e.preventDefault()
          val elements = thisNode.ref.elements
          for (i <- 0 until elements.length ) {
            val input = elements(i).asInstanceOf[HTMLInputElement]
            info(s"$i input.name: ${input.name}")
            info(s"$i input.value: ${input.value}")
            info(s"$i input.validity.valid: ${input.validity.valid}")
          }
          ()
        })),
        cls := "grid grid-cols-3 gap-4",
        InputLabel("name", "Nombre", patient.map(p => p.map(_.name)), readOnly.signal),
        InputLabel("name", "Apellido paterno", patient.map(p => p.map(_.paternalSurname)), readOnly.signal),
        InputLabel("name", "Apellido materno", patient.map(p => p.map(_.maternalSurname)), readOnly.signal),
        InputLabel("age", "Edad", Signal.fromValue(Some("80")), readOnly.signal, Some("number")),
        InputLabel("email", "Correo", Signal.fromValue(Some("email@gmail.com")), readOnly.signal, Some("email")),
        InputLabel("phone", "Telefono", Signal.fromValue(Some("+51 555 555 555")), readOnly.signal, Some("tel")),
        InputLabel("allergies", "Alergias", Signal.fromValue(Some("")), readOnly.signal),
        div(cls := "col-start-3 flex justify-end",
          Button(readOnly),
        ),
      ),
    )
  }

}
