package medical.application

import akka.NotUsed
import akka.stream.scaladsl.Source
import medical.domain.PatientBasic
import medical.domain.repository.PatientRepository

trait PatientService {
  def find(name: String): Source[PatientBasic, NotUsed]
}

class PatientServiceImpl(patientRepository: PatientRepository) extends PatientService {
  override def find(name: String): Source[PatientBasic, NotUsed] = {
    patientRepository.find(name)
  }
}
