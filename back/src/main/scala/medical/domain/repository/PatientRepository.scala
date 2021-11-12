package medical.domain.repository

import akka.stream.scaladsl.Source
import akka.NotUsed
import medical.domain.PatientBasic

trait PatientRepository {
  def find(name: String): Source[PatientBasic, NotUsed]

}
