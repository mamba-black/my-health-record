package medical.infraestructure.repository

import medical.domain.PatientBasic
import medical.domain.repository.PatientRepository

class PatientRepositoryImpl extends PatientRepository {
  override def find(name: String): PatientBasic = {
    ???
  }

}
