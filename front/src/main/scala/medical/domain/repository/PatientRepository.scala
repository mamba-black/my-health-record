package medical.domain.repository

import com.raquo.airstream.core.{ EventStream, Observer }
import medical.api.patientapi.{ FullPatientReply, PatientReply }
import medical.domain.Patient

import scala.concurrent.Future

trait PatientRepository {
  def findByName(name: String): EventStream[PatientReply]

  def getById(patientId: String): Future[FullPatientReply]

  def save(patient: Patient, patientWriter: Observer[Option[Patient]]): Unit
}
