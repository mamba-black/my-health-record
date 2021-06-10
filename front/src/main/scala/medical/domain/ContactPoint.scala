package medical.domain

import medical.domain.SystemContactPoint.SystemContactPoint
import medical.domain.UseContactPoint.UseContactPoint

/**
  *
  * @param system C? phone | fax | email | pager | url | sms | other
  * @param value The actual contact point details
  * @param use // home | work | temp | old | mobile - purpose of this contact point
  *
  * @see <a href="https://www.hl7.org/fhir/datatypes.html#ContactPoint">fhir/datatypes#ContactPoint</a>
  */
class ContactPoint(
                    val system: SystemContactPoint,
                    val value: String,
                    val use:  UseContactPoint = UseContactPoint.HOME,
                  )

object SystemContactPoint extends Enumeration {
  type SystemContactPoint = Value
  val PHONE, FAX, EMAIL, PAGER, URL, SMS = Value
}

object UseContactPoint extends Enumeration {
  type UseContactPoint = Value
  val HOME, WORK, TEMP, OLD, MOBILE = Value
}
