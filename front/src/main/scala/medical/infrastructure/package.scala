package medical

import medical.domain.repository.PatientRepository
import medical.infrastructure.repository.PatientRepositoryImpl

package object infrastructure {
  val patientRepository: PatientRepository = new PatientRepositoryImpl
}
