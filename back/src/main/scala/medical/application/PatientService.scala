package medical.application

import medical.domain.PatientBasic
import medical.domain.repository.PatientRepository

trait PatientService {
  def find(name: String): PatientBasic
}

class PatientServiceImpl(patientRepository: PatientRepository) extends PatientService {
  override def find(name: String): PatientBasic = {
    patientRepository.find(name)
  }
}
