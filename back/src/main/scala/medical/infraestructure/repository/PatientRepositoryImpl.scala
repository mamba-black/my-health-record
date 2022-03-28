package medical.infraestructure.repository

import akka.stream.scaladsl.Source
import akka.NotUsed
import algolia.AlgoliaClient
import algolia.AlgoliaDsl.*
import algolia.objects.Query
import medical.domain.PatientBasic
import medical.domain.repository.PatientRepository
import scribe.info

import scala.concurrent.ExecutionContext.Implicits.global

class PatientRepositoryImpl extends PatientRepository {

  private val client = new AlgoliaClient("9DTTCP8MNP", "35185c2fd14f34ffade3b9dc9c363049")

  override def find(name: String): Source[PatientBasic, NotUsed] = {
    info(s"name: $name")
    val future = client
      .execute {
        search into "patients" query Query(query = Some(name))
      }
      .map {
        _.as[PatientAlgolia]
      }
    Source
      .future(future)
      .mapConcat(identity)
      .map(pb => PatientBasic(pb.identifier, pb.text))
  }
}

case class PatientAlgolia(identifier: String, text: String)