package medical.infraestructure.di

import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.Behaviors
import akka.cluster.sharding.typed.scaladsl.{ ClusterSharding, Entity, EntityTypeKey }
import algolia.AlgoliaClient
import com.softwaremill.macwire.*
import com.typesafe.config.ConfigFactory
import grpc.health.v1.Health
import medical.api.PatientApi
import medical.application.{ PatientService, PatientServiceImpl }
import medical.domain.repository.PatientRepository
import medical.infraestructure.api.{ HealthImpl, PatientApiImpl }
import medical.infraestructure.repository.PatientRepositoryImpl
import medical.infraestructure.PatientServer
import medical.infraestructure.actor.PatientBehavior
import medical.infraestructure.actor.PatientBehavior.PatientTypeKey

trait PatientModule {
  lazy val conf = ConfigFactory
    .parseString("akka.http.server.preview.enable-http2 = on")
    .withFallback(ConfigFactory.defaultApplication())

  lazy implicit val system = ActorSystem[Nothing](Behaviors.empty, "PatientServer", conf)

  lazy val clusterSharding = {
    lazy val clusterSharding = ClusterSharding(system)
    clusterSharding.init(
      Entity(PatientTypeKey)(createBehavior = entityContext => PatientBehavior(entityContext.entityId))
    )
    clusterSharding
  }

  lazy val client = new AlgoliaClient("9DTTCP8MNP", "35185c2fd14f34ffade3b9dc9c363049")

  lazy val patientRepository: PatientRepository = wire[PatientRepositoryImpl]
  lazy val patientService: PatientService = wire[PatientServiceImpl]
  lazy val patientApi: PatientApi = wire[PatientApiImpl]
  lazy val health: Health = wire[HealthImpl]
  lazy val patientServer = wire[PatientServer]

}
