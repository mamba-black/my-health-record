package medical.domain

import scala.scalajs.js

/**
  *
  * @param text    Text representation of the full name
  * @param family  Family name (often called 'Surname')
  * @param fathersFamily
  * @param mothersFamily
  * @param given Given names (not always 'first'). Includes middle names
  *
  * @see <a href="http://hl7.org/fhir/datatypes#HumanName">fhir/datatypes.html#HumanName</a>
  */
class HumanName(
                 val fathersFamily: String,
                 val mothersFamily: String,
                 val given: Seq[String],
               ) extends js.Object {
  val family: String = s"$fathersFamily $mothersFamily"
  val text: String = s"$fathersFamily $mothersFamily, ${`given`.foldLeft(" ")( _ + _).trim}"
}
