package medical.domain

/**
  * @param fathersFamily <a href="http://hl7.org/fhir/StructureDefinition/humanname-fathers-family">fathers family</a>
  * @param mothersFamily <a href="http://hl7.org/fhir/StructureDefinition/humanname-mothers-family">mothers family</a>
  * @param given         Given names (not always 'first'). Includes middle names
  * @see <a href="http://hl7.org/fhir/datatypes.html#humanname">fhir/datatypes.html#HumanName</a>
  */
case class HumanName(val fathersFamily: String, val mothersFamily: String, val given: Seq[String]) {

  /** Family name (often called 'Surname') */
  val family: String = s"$fathersFamily $mothersFamily"

  /** Text representation of the full name */
  val text: String = s"$fathersFamily $mothersFamily, ${`given`.foldLeft(" ")(_ + _).trim}"
}
