package medical.domain

import scala.scalajs.js

/**
  * @see <a href="https://www.hl7.org/fhir/patient.html">fhir/patient</a>
  *
  * @param id
  * @param name
  * @param active
  * @param age
  * @param telecom
  */
class Patient(
    val id: String,
    val name: HumanName,
    val active: Boolean,
    val birthDate: java.time.LocalDate,
    val telecom: Seq[ContactPoint] = Seq(),
) extends js.Object
