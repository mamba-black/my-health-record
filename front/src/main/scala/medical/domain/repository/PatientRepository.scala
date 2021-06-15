package medical.domain.repository

import com.raquo.airstream.core.Observer
import medical.domain.Patient

trait PatientRepository {
  def findById(patientId: String, patientWriter: Observer[Option[Patient]]): Unit

  def save(patient: Patient, patientWriter: Observer[Option[Patient]]): Unit
}
