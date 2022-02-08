package medical.domain.repository

import com.raquo.airstream.core.{ EventStream, Observer }
import medical.api.patientapi.PatientReply
import medical.domain.Patient

trait PatientRepository {
  def findByName(name: String): EventStream[PatientReply]

  def getById(patientId: String, patientWriter: Observer[Option[Patient]]): Unit

  def save(patient: Patient, patientWriter: Observer[Option[Patient]]): Unit
}
