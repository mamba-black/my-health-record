package medical.domain

/**
  * @see <a href="https://www.hl7.org/fhir/patient.html">fhir/patient</a>
  * @param identifier
  * @param name
  * @param active
  * @param telecom
  */
class Patient(
    val identifier: String,
    val name: HumanName,
    val active: Boolean,
    val birthDate: java.time.LocalDate,
    val telecom: Seq[ContactPoint] = Seq(),
)
