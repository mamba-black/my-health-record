package medical.domain.repository

import com.raquo.airstream.core.Observer
import medical.api.patientapi.{ FullPatientReply, PatientReply }
import medical.domain.Patient

import scala.concurrent.Future

trait PatientRepository {
  def findByName(name: String, callBack: PatientReply => Unit): Unit

  def getById(patientId: String): Future[FullPatientReply]

  def save(patient: Patient, patientWriter: Observer[Option[Patient]]): Unit
}
