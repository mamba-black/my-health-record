package medical.infrastructure.ui.molecule

import com.raquo.laminar.api.L._
import medical.domain.Patient
import medical.infrastructure.ui.atom.{ Button, InputLabel }
import org.scalajs.dom
import org.scalajs.dom.raw.HTMLInputElement
import scribe._

object PatientBasicInfo {

  def apply(patient: Patient): HtmlElement = {

    val readOnly = Var[Boolean](true)
    div(
      h1("Informacion del paciente",
        cls := ("text-2xl", "py-8"),
      ),
      form(
        inContext(thisNode => onSubmit --> Observer[dom.Event](e => {
          e.preventDefault()
          info(s"e: $e")
          info(s"e.target: ${e.target}")
          info(s"readOnly.tryNow(): ${readOnly.tryNow()}")
          info(s"thisNode.ref.elements.length: ${thisNode.ref.elements.length}")
          info(s"thisNode.ref.elements.item(0): ${thisNode.ref.elements.item(0)}")
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
        InputLabel("name", "Nombre", patient.name, readOnly.signal),
        InputLabel("name", "Apellido paterno", patient.paternalSurname, readOnly.signal),
        InputLabel("name", "Apellido materno", patient.maternalSurname, readOnly.signal),
        InputLabel("age", "Edad", "80", readOnly.signal, Some("number")),
        InputLabel("email", "Correo", "email@gmail.com", readOnly.signal, Some("email")),
        InputLabel("phone", "Telefono", "+51 555 555 555", readOnly.signal, Some("tel")),
        InputLabel("allergies", "Alergias", "", readOnly.signal),
        div(cls := "col-start-3 flex justify-end",
          Button(readOnly),
        ),
      ),
    )
  }

}
