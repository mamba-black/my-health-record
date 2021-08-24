package medical.ui.molecule

import com.raquo.laminar.api.L._
import medical.domain.{ ContactPoint, HumanName, Patient, SystemContactPoint }
import medical.infrastructure.patientRepository
import medical.ui.atom.{ Button, InputLabel, One, Two }
import org.scalajs.dom.MouseEvent
import org.scalajs.dom.raw.HTMLInputElement
import scribe._

import java.time.LocalDate

object PatientBasicInfo {
  val NAME = "name"
  val FATHERS_FAMILY = "fathersFamily"
  val MOTHERS_FAMILY = "mothersFamily"
  val AGE = "age"
  val EMAIL = "email"
  val PHONE = "phone"

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    val (readOnlyFlag, patientVar, name, fathersFamily, mothersFamily, age, email, phone) =
      generateInputs(patientId, patient)

    //val bt = button("validate")
    val editarButton = Var("Editar")

    div(
      h1("Informacion del paciente", cls := ("text-2xl", "py-8")),
      form(
        cls := "grid grid-cols-3 gap-4",
        //inContext(thisForm => onSubmit --> Observer[Event](_onSubmit(thisForm, readOnlyFlag, patientVar))),
        InputLabel(NAME, "Nombre", name, readOnlyFlag.signal),
        InputLabel(FATHERS_FAMILY, "Apellido paterno", fathersFamily, readOnlyFlag.signal),
        InputLabel(MOTHERS_FAMILY, "Apellido materno", mothersFamily, readOnlyFlag.signal),
        InputLabel(AGE, "Edad", age, readOnlyFlag.signal, Some("number")),
        InputLabel(EMAIL, "Correo", email, readOnlyFlag.signal, Some("email")),
        InputLabel(PHONE, "Telefono", phone, readOnlyFlag.signal, Some("tel")),
        InputLabel("allergies", "Alergias", Signal.fromValue(Some("")), readOnlyFlag.signal),
        inContext(thisForm =>
          div(
            cls := "col-start-3 flex justify-end",
            //Button("Editar", Observer[Boolean](toggle => if(toggle) readOnlyFlag.set(!readOnlyFlag.now())), "Guardar"),
            Button(editarButton.signal, {
              case (One, _) =>
                readOnlyFlag.set(false)
                editarButton.set("Guardar")
                true
              case (Two, e) =>
                _onSubmit(thisForm, readOnlyFlag, patientVar, e)
              case _ =>
                info("test")
                true
            }),
          )
        )
      )
    )
  }

  private def generateInputs(patientId: String, patient: Option[Patient]): (
      Var[Boolean],
      Var[Option[Patient]],
      Signal[Option[String]],
      Signal[Option[String]],
      Signal[Option[String]],
      Signal[Option[String]],
      Signal[Option[String]],
      Signal[Option[String]]
  ) = {
    val now = java.time.LocalDate.now()
    val readOnlyFlag = Var[Boolean](true)
    val patientVar = Var(patient)
    val patientSignal: Signal[Option[Patient]] = patientVar.signal

    if (patient.isEmpty) {
      patientRepository.findById(patientId, patientVar.writer)
    }

    val name = patientSignal.map(p => p.map(_.name.`given`.foldLeft(" ")(_ + _)))
    val fathersFamily = patientSignal.map(p => p.map(_.name.fathersFamily))
    val mothersFamily = patientSignal.map(p => p.map(_.name.mothersFamily))
    val age = patientSignal.map(p => p.map(now.getYear - _.birthDate.getYear).map(_.toString))
    val email = patientSignal.map(p => {
      val _email = p.flatMap(
        _.telecom
          .to(LazyList)
          .filter(_.system == SystemContactPoint.EMAIL)
          .map(_.value)
          .headOption
      )
      if (p.isDefined && _email.isEmpty) Option("") else _email
    })
    val phone = patientSignal.map(p => {
      val _phone = p.flatMap(
        _.telecom
          .to(LazyList)
          .filter(_.system == SystemContactPoint.PHONE)
          .map(_.value)
          .headOption
      )
      if (p.isDefined && _phone.isEmpty) Option("") else _phone
    })
    (readOnlyFlag, patientVar, name, fathersFamily, mothersFamily, age, email, phone)
  }

  private def _onSubmit(_form: FormElement, readOnlyFlag: Var[Boolean], patientVar: Var[Option[Patient]], e: MouseEvent): Boolean = {
    val elements = _form.ref.elements
    var name: String = null
    var fathersFamily: String = null
    var mothersFamily: String = null
    var email: String = null
    var phone: String = null
    for (i <- 0 until elements.length) {
      val input = elements(i).asInstanceOf[HTMLInputElement]
      debug(s"$i ${input.name}: ${input.value} (valid: ${input.validity.valid})")
      if (!input.validity.valid) {
        error("input invalid")
        //_form.ref.submit()
        return false
      }
      input.name match {
        case NAME           => name = input.value
        case FATHERS_FAMILY => fathersFamily = input.value
        case MOTHERS_FAMILY => mothersFamily = input.value
        case EMAIL          => email = input.value
        case PHONE          => phone = input.value
        case others @ _     => info(others)
      }
    }
    e.preventDefault()
    info(s"$readOnlyFlag")
    //if (readOnlyFlag.now()) {
      val dd = div()
      _form.ref.parentElement.appendChild(dd.ref)
      render(
        dd.ref,
        Modal(
          "Esto es una prueba",
          () => _form.ref.reset(),
          () => {
            //_form.ref.submit()
            // FIXME: crear servicia para guardar
            val patient = new Patient(
              "Test",
              new HumanName(fathersFamily, mothersFamily, name.split(" ").toSeq),
              true,
              LocalDate.of(1979, 10, 13),
              Seq(new ContactPoint(SystemContactPoint.PHONE, "993990103"))
            )
            patientRepository.save(patient, patientVar.writer)
          }
        )
      )
    //}
    true
  }
}
