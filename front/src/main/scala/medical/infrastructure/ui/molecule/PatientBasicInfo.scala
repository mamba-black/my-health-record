package medical.infrastructure.ui.molecule

import com.raquo.laminar.api.L._
import medical.domain.{ ContactPoint, HumanName, Patient, SystemContactPoint }
import medical.infrastructure.ui.atom.{ Button, InputLabel }
import org.scalajs.dom
import org.scalajs.dom.raw.HTMLInputElement
import scribe._

import java.time.LocalDate
import scala.scalajs.js.timers.setTimeout

object PatientBasicInfo {
  val NAME = "name"
  val FATHERS_FAMILY = "fathersFamily"
  val MOTHERS_FAMILY = "mothersFamily"
  val AGE = "age"
  val EMAIL = "email"
  val PHONE = "phone"

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    val now = java.time.LocalDate.now()
    val readOnlyFlag = Var[Boolean](true)
    val patientVar = Var(patient)
    val patientSignal: Signal[Option[Patient]] = patientVar.signal

    if (patient.isEmpty) {
      // FIXME: Aqui colocar el servicio para traer la data si patient es None
      // FIXME: Quitar, esto solo es paro demo
      setTimeout(2000) {
        val patient = new Patient(
          patientId,
          new HumanName("Malpica", "Gallegos", Seq("Hector", "Miuler")),
          true,
          LocalDate.of(1979,10, 13),
          Seq(new ContactPoint(SystemContactPoint.PHONE, "993990103")),
        )
        patientVar.set(Some(patient))
        debug("TIMEOUT")
      }
    }

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
        inContext(thisForm => onSubmit --> Observer[dom.Event](_onSubmit(thisForm, readOnlyFlag, patientVar))),
        InputLabel(NAME, "Nombre", name, readOnlyFlag.signal),
        InputLabel(FATHERS_FAMILY, "Apellido paterno", fathersFamily, readOnlyFlag.signal),
        InputLabel(MOTHERS_FAMILY, "Apellido materno", mothersFamily, readOnlyFlag.signal),
        InputLabel(AGE, "Edad", age, readOnlyFlag.signal, Some("number")),
        InputLabel(EMAIL, "Correo", email, readOnlyFlag.signal, Some("email")),
        InputLabel(PHONE, "Telefono", phone, readOnlyFlag.signal, Some("tel")),
        InputLabel("allergies", "Alergias", Signal.fromValue(Some("")), readOnlyFlag.signal),
        div(cls := "col-start-3 flex justify-end",
          Button("Editar", readOnlyFlag, "Guardar"),
        ),
      ),
    )
  }


  private def _onSubmit(_form: FormElement,
                        readOnlyFlag: Var[Boolean],
                        patientVar: Var[Option[Patient]])(e: dom.Event): Unit = {
    e.preventDefault()
    val elements = _form.ref.elements
    var name: String = null
    var fathersFamily: String = null
    var mothersFamily: String = null
    var email: String = null
    var phone: String = null
    for (i <- 0 until elements.length) {
      val input = elements(i).asInstanceOf[HTMLInputElement]
      debug(s"$i input.name: ${input.name}")
      debug(s"$i input.value: ${input.value}")
      debug(s"$i input.validity.valid: ${input.validity.valid}")
      input.name match {
        case NAME => name = input.value
        case FATHERS_FAMILY => fathersFamily = input.value
        case MOTHERS_FAMILY => mothersFamily = input.value
        case EMAIL => email = input.value
        case PHONE => phone = input.value
        case _ => info("")
      }
    }
    if (readOnlyFlag.now()) {
      val dd = div()
      _form.ref.parentElement.appendChild(dd.ref)
      render(dd.ref, Modal("Esto es una prueba", () => _form.ref.reset(), () => {
        //_form.ref.submit()
        // FIXME: crear servicia para guardar
        val patient = new Patient(
          "Test",
          new HumanName(fathersFamily, mothersFamily, name.split(" ").toSeq),
          true,
          LocalDate.of(1979,10, 13),
          Seq(new ContactPoint(SystemContactPoint.PHONE, "993990103")),
        )
        patientVar.set(Some(patient))
      }))
    }
    ()
  }
}
