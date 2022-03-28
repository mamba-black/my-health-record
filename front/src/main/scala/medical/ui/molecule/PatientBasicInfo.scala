package medical.ui.molecule

import com.raquo.laminar.api.L.*
import medical.domain.{ ContactPoint, HumanName, Patient, SystemContactPoint }
import medical.infrastructure.patientRepository
import medical.ui.atom.{ Button, InputLabel }
import medical.ui.molecule.EditStatus.{ ReadOnly, Edit }
import org.scalajs.dom.{ HTMLCollection, HTMLInputElement, MouseEvent }
import org.scalajs.macrotaskexecutor.MacrotaskExecutor.Implicits.*
import scribe.*

import java.time.LocalDate
import scala.util.{ Failure, Success, Try }

object PatientBasicInfo {
  val NAME = "name"
  val FATHERS_FAMILY = "fathersFamily"
  val MOTHERS_FAMILY = "mothersFamily"
  val AGE = "age"
  val EMAIL = "email"
  val PHONE = "phone"
  val ADDRESS = "direction"
  val ALLERGIES = "alergias"

  def apply(patientId: String, patient: Option[Patient]): HtmlElement = {
    val basicInfo = generateInputs(patientId, patient)

    // val bt = button("validate")
    val editarButton = Var("Editar")
    val status = Var[EditStatus](ReadOnly)
    div(
      h1("Informacion del paciente", cls := ("text-2xl", "py-8")),
      form(
        cls := "grid grid-cols-3 gap-4",
        // inContext(thisForm => onSubmit --> Observer[Event](_onSubmit(thisForm, readOnlyFlagVar.signal, patientVar))),
        InputLabel(NAME, "Nombre", basicInfo.name, basicInfo.readOnlyFlag.signal),
        InputLabel(FATHERS_FAMILY, "Apellido paterno", basicInfo.fathersFamily, basicInfo.readOnlyFlag.signal),
        InputLabel(MOTHERS_FAMILY, "Apellido materno", basicInfo.mothersFamily, basicInfo.readOnlyFlag.signal),
        InputLabel(AGE, "Edad", basicInfo.age, basicInfo.readOnlyFlag.signal, Some("number")),
        InputLabel(EMAIL, "Correo", basicInfo.email, basicInfo.readOnlyFlag.signal, Some("email")),
        InputLabel(PHONE, "Telefono", basicInfo.phone, basicInfo.readOnlyFlag.signal, Some("tel")),
        InputLabel(ADDRESS, "Direccion", basicInfo.phone, basicInfo.readOnlyFlag.signal, Some("address")),
        InputLabel(ALLERGIES, "Alergias", basicInfo.allergies, basicInfo.readOnlyFlag.signal, important = true),
        inContext(thisForm =>
          div(
            cls := "relative",
            div(
              cls := "col-start-3 flex justify-end w-24 h-12 absolute bottom-0 right-0",
              Button(
                editarButton.signal,
                event => {
                  debug(s"status in Guardar button: $event")
                  status.now() match {
                    case ReadOnly => // Editar
                      basicInfo.readOnlyFlag.set(false)
                      editarButton.set("Guardar")
                      status.set(Edit)
                      true
                    case Edit => // Guardar o Descartar
                      savePatientBasic(
                        thisForm,
                        basicInfo.readOnlyFlag,
                        basicInfo.patientVar,
                        editarButton,
                        status,
                        event,
                      )
                      status.set(ReadOnly)
                      true
                  }
                },
              ),
            ),
          )
        ),
      ),
    )
  }

  private def savePatientBasic(
      _form: FormElement,
      readOnlyFlag: Var[Boolean],
      patientVar: Var[Option[Patient]],
      labelEditButton: Var[String],
      status: Var[EditStatus],
      e: MouseEvent,
  ): Boolean = {
    val elements = _form.ref.elements

    val (name, fathersFamily, mothersFamily, _, _) = getBasicElements(elements) match {
      case Success(value) => value
      case _              => return false
    }
    e.preventDefault()

    info(s"$readOnlyFlag")
    // if (readOnlyFlag.now()) {
    val dd = div()
    _form.ref.parentElement.appendChild(dd.ref)
    render(
      dd.ref,
      Modal(
        "Esto es una prueba",
        () => {
          readOnlyFlag.set(true)
          _form.ref.reset()
          labelEditButton.set("Editar")
        },
        () => {
          readOnlyFlag.set(true)
          for (i <- 0 until elements.length) {
            val input = elements(i).asInstanceOf[HTMLInputElement]
            input.defaultValue = input.value
          }

          // FIXME: crear servicia para guardar
          val patient = new Patient(
            "Test",
            new HumanName(fathersFamily, mothersFamily, name.split(" ").asInstanceOf[Array[String]].toSeq),
            true,
            LocalDate.of(1979, 10, 13).asInstanceOf[LocalDate],
            Seq(new ContactPoint(SystemContactPoint.PHONE, "993990103")),
          )
          patientRepository.save(patient, patientVar.writer)
          labelEditButton.set("Editar")
        },
        () => {
          status.set(Edit)
        },
      ),
    )
    // }
    true
  }

  private def getBasicElements(
      elements: HTMLCollection[org.scalajs.dom.Element]
  ): Try[(String, String, String, String, String)] = {
    var name: String = ""
    var fathersFamily: String = ""
    var mothersFamily: String = ""
    var email: String = ""
    var phone: String = ""
    for (i <- 0 until elements.length) {
      val input = elements(i).asInstanceOf[HTMLInputElement]
      debug(s"$i ${input.name}: ${input.value} (valid: ${input.validity.valid})")
      if (!input.validity.valid) {
        error("input invalid")
        // _form.ref.submit()
        return Failure(new Exception("Invalid input"))
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
    Success((name, fathersFamily, mothersFamily, email, phone))
  }

  private def generateInputs(patientId: String, patient: Option[Patient]): PatientBasicInfo = {
    val now: LocalDate = java.time.LocalDate.now().asInstanceOf[LocalDate]
    val readOnlyFlag = Var[Boolean](true)
    val patientVar = Var(patient)
    val patientSignal: Signal[Option[Patient]] = patientVar.signal

    if (patient.isEmpty) {
      debug("patient is empty, call the repository")
      patientRepository
        .getById(patientId)
        .map(fullPatientReply => {
          debug("Prueba de mapeo")
          fullPatientReply.copy(name = s"--${fullPatientReply.name}--")
        })
        .onComplete(fullPatientReply => {
          debug(s"reply: $fullPatientReply")
          val patient = new Patient(
            patientId,
            new HumanName("Malpica2", "Gallegos2", Seq("Hector2", "Miuler2")),
            true,
            LocalDate.of(1979, 10, 13).asInstanceOf[LocalDate],
            Seq(new ContactPoint(SystemContactPoint.PHONE, "+51993990103")),
          )
          patientVar.writer.onNext(Some(patient))
        })
    }

    val name = patientSignal.map(p => p.map(_.name.`given`.foldLeft(" ")(_ + _)))
    val fathersFamily = patientSignal.map(p => p.map(_.name.fathersFamily))
    val mothersFamily = patientSignal.map(p => p.map(_.name.mothersFamily))
    val age =
      patientSignal.map(p => p.map(now.getYear - _.birthDate.getYear).map(_.toString))
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
    PatientBasicInfo(
      readOnlyFlag,
      patientVar.signal,
      patientVar,
      name,
      fathersFamily,
      mothersFamily,
      age,
      email,
      phone,
      email,
    )
  }

}

private[molecule] case class PatientBasicInfo(
    readOnlyFlag: Var[Boolean],
    patient: Signal[Option[Patient]],
    patientVar: Var[Option[Patient]],
    name: Signal[Option[String]],
    fathersFamily: Signal[Option[String]],
    mothersFamily: Signal[Option[String]],
    age: Signal[Option[String]],
    email: Signal[Option[String]],
    phone: Signal[Option[String]],
    allergies: Signal[Option[String]],
)

private enum EditStatus {
  case ReadOnly, Edit
}
