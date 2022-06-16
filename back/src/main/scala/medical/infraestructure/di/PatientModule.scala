package medical.infraestructure.di

import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.Behaviors
import algolia.AlgoliaClient
import com.softwaremill.macwire.wire
import com.typesafe.config.ConfigFactory
import grpc.health.v1.Health
import medical.api.PatientApi
import medical.application.{ PatientService, PatientServiceImpl }
import medical.domain.repository.PatientRepository
import medical.infraestructure.api.{ HealthImpl, PatientApiImpl }
import medical.infraestructure.repository.PatientRepositoryImpl
import medical.infraestructure.PatientServer


trait PatientModule {

  lazy val conf = ConfigFactory
    .parseString("akka.http.server.preview.enable-http2 = on")
    .withFallback(ConfigFactory.defaultApplication())

  lazy val system = ActorSystem[Nothing](Behaviors.empty, "PatientServer", conf)
  lazy val client = new AlgoliaClient("9DTTCP8MNP", "35185c2fd14f34ffade3b9dc9c363049")

  lazy val patientRepository: PatientRepository = wire[PatientRepositoryImpl]
  lazy val patientService: PatientService = wire[PatientServiceImpl]
  lazy val patientApi: PatientApi = wire[PatientApiImpl]
  lazy val health: Health = wire[HealthImpl]
  lazy val patientServer = wire[PatientServer]

}
