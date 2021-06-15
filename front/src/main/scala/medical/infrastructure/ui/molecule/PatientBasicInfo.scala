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
    val readOnlyFlag = Var[Boolean](true)

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
      h1("Informacion del paciente", cls := ("text-2xl", "py-8")),
      form(
        cls := "grid grid-cols-3 gap-4",
        inContext(thisForm => onSubmit --> Observer[dom.Event](_onSubmit(thisForm, readOnlyFlag))),
        InputLabel("name", "Nombre", name, readOnlyFlag.signal),
        InputLabel("name", "Apellido paterno", fathersFamily, readOnlyFlag.signal),
        InputLabel("name", "Apellido materno", mothersFamily, readOnlyFlag.signal),
        InputLabel("age", "Edad", age, readOnlyFlag.signal, Some("number")),
        InputLabel("email", "Correo", email, readOnlyFlag.signal, Some("email")),
        InputLabel("phone", "Telefono", phone, readOnlyFlag.signal, Some("tel")),
        InputLabel("allergies", "Alergias", Signal.fromValue(Some("")), readOnlyFlag.signal),
        div(cls := "col-start-3 flex justify-end",
          Button("Editar", readOnlyFlag, "Guardar"),
        ),
      ),
    )
  }


  private def _onSubmit(_form: FormElement,
                        readOnlyFlag: Var[Boolean]
                        )(e: dom.Event): Unit = {
    e.preventDefault()
    val elements = _form.ref.elements
    for (i <- 0 until elements.length) {
      val input = elements(i).asInstanceOf[HTMLInputElement]
      debug(s"$i input.name: ${input.name}")
      debug(s"$i input.value: ${input.value}")
      debug(s"$i input.validity.valid: ${input.validity.valid}")
    }
    if (readOnlyFlag.now() ) {
      val dd = div()
      _form.ref.parentElement.appendChild(dd.ref)
      render(dd.ref, Modal("Esto es una prueba", () => _form.ref.reset(), () => _form.ref.submit()))
    }
    ()
  }
}
