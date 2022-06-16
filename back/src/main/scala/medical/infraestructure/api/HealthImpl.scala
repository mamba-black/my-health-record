package medical.infraestructure.api

import akka.NotUsed
import akka.stream.scaladsl.Source
import grpc.health.v1.{ Health, HealthCheckRequest, HealthCheckResponse }
import grpc.health.v1.HealthCheckResponse.ServingStatus.SERVING

import scala.concurrent.Future

class HealthImpl extends Health {
  override def check(in: HealthCheckRequest): Future[HealthCheckResponse] =
    Future.successful(HealthCheckResponse(SERVING))

  override def watch(in: HealthCheckRequest): Source[HealthCheckResponse, NotUsed] =
    Source.single(HealthCheckResponse(SERVING))
}
