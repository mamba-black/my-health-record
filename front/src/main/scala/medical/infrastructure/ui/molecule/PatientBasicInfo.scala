package medical.infrastructure.ui.molecule

import com.raquo.laminar.api.L._
import medical.domain.{ Patient, SystemContactPoint }
import medical.infrastructure.ui.atom.{ Button, InputLabel }
import org.scalajs.dom
import org.scalajs.dom.raw.HTMLInputElement
import scribe._

object PatientBasicInfo {

  def apply(patientSignal: Signal[Option[Patient]]): HtmlElement = {
    val now = java.time.LocalDate.now()

    val readOnly = Var[Boolean](true)
    val name = patientSignal.map(p => p.map(_.name.`given`.foldLeft(" ")(_ + _)))
    val fathersFamily = patientSignal.map(p => p.map(_.name.fathersFamily))
    val mothersFamily = patientSignal.map(p => p.map(_.name.mothersFamily))
    val age = patientSignal.map(p => p.map(now.getYear - _.birthDate.getYear).map(_.toString))
    val email = patientSignal.map(p => {
      val _email = p.flatMap(_
        .telecom
        .to(LazyList)
        .filter(_.system == SystemContactPoint.EMAIL)
        .map(_.value)
        .headOption)
      if (p.isDefined && _email.isEmpty) Option("") else _email
    })
    val phone = patientSignal.map(p => {
      val _phone = p.flatMap(_
        .telecom
        .to(LazyList)
        .filter(_.system == SystemContactPoint.PHONE)
        .map(_.value)
        .headOption)
      if (p.isDefined && _phone.isEmpty) Option("") else _phone
    })

    div(
      h1("Informacion del paciente",
        cls := ("text-2xl", "py-8"),
      ),
      form(
        inContext(thisNode => onSubmit --> Observer[dom.Event](e => {
          e.preventDefault()
          val elements = thisNode.ref.elements
          for (i <- 0 until elements.length) {
            val input = elements(i).asInstanceOf[HTMLInputElement]
            info(s"$i input.name: ${input.name}")
            info(s"$i input.value: ${input.value}")
            info(s"$i input.validity.valid: ${input.validity.valid}")
          }
          ()
        })),
        cls := "grid grid-cols-3 gap-4",
        InputLabel("name", "Nombre", name, readOnly.signal),
        InputLabel("name", "Apellido paterno", fathersFamily, readOnly.signal),
        InputLabel("name", "Apellido materno", mothersFamily, readOnly.signal),
        InputLabel("age", "Edad", age, readOnly.signal, Some("number")),
        InputLabel("email", "Correo", email, readOnly.signal, Some("email")),
        InputLabel("phone", "Telefono", phone, readOnly.signal, Some("tel")),
        InputLabel("allergies", "Alergias", Signal.fromValue(Some("")), readOnly.signal),
        div(cls := "col-start-3 flex justify-end",
          Button(readOnly),
        ),
      ),
      child <-- readOnly.signal.changes.map(a => div(s"a: $a")),
    )
  }

}
