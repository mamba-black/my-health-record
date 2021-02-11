package medical.domain.repository

import medical.domain.PatientBasic

trait PatientRepository {
  def find(name: String): PatientBasic

}
