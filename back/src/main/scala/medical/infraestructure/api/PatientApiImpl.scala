package medical.infraestructure.api

import akka.NotUsed
import akka.actor.typed.ActorSystem
import akka.cluster.sharding.typed.scaladsl.ClusterSharding
import akka.grpc.GrpcServiceException
import akka.grpc.scaladsl.MetadataBuilder
import akka.stream.scaladsl.Source
import akka.util.{ ByteString, Timeout }
import io.grpc.Status
import medical.api.*
import medical.application.PatientService
import medical.domain.{ HumanName, Patient }
import medical.infraestructure.actor.PatientBehavior.*
import scribe.*

import java.util.UUID
import scala.concurrent.duration.DurationInt
import scala.concurrent.{ ExecutionContextExecutor, Future }

class PatientApiImpl(
    patientService: PatientService,
    clusterSharding: ClusterSharding,
    implicit val system: ActorSystem[?]
) extends PatientApi {
  //  private implicit val sys: ActorSystem[_] = system
  private implicit val ec: ExecutionContextExecutor = system.executionContext
  private implicit val timeout: Timeout = 3.seconds

  /** Sends a greeting
    */
  override def find(patientRequest: PatientRequest): Source[PatientReply, NotUsed] = {
    info(s"[${Thread.currentThread().getName}/${system.name}] 1.name: ${patientRequest.name}")
    system.log.info(s"2.name: ${patientRequest.name}")
    val patientBasic = patientService.find(patientRequest.name)
    patientBasic.map(pb => PatientReply(pb.idenditier, pb.text))
  }

  override def get(in: PatientIdRequest): Future[PatientReply] = {
    info(s"[${Thread.currentThread().getName}/${system.name}] 1.id: ${in.id}")
    system.log.info(s"2.id: ${in.id}")

    val patientBehavior = clusterSharding.entityRefFor(PatientTypeKey, in.id)

    patientBehavior
      .ask(ref => GetPatient(in.id, ref))
      .map(patient =>
        PatientReply(
          patient.identifier,
          patient.name.`given`.mkString(" "),
          patient.name.fathersFamily,
          patient.name.mothersFamily
        )
      )
  }

  override def getFullPatient(in: PatientIdRequest): Future[FullPatientReply] = {

    Future.successful(FullPatientReply(UUID.randomUUID().toString, "Hector Miuler", "Malpica", "Gallegos"))
  }

  override def savePatient(in: FullPatientReply): Future[FullPatientReply] = {
    validatePatient(in) match {
      case Some(toReturn) => return toReturn
      case None           =>
    }

    val patientBehavior = clusterSharding.entityRefFor(PatientTypeKey, in.id)

    patientBehavior
      .ask(replyTo =>
        SavePatient(
          Patient(in.id, HumanName(in.paternalSurname, in.maternalSurname, in.name.split(" ")), false, null, null),
          replyTo
        )
      )
      .map(patient =>
        new FullPatientReply(
          patient.identifier,
          patient.name.`given`.mkString(" "),
          patient.name.fathersFamily,
          patient.name.mothersFamily
        )
      )
  }

  private def validatePatient(in: FullPatientReply): Option[Future[Nothing]] = {
    val exceptionMetadata = new MetadataBuilder()
      .addText("test-text", "test-text-data")
      .addBinary("test-binary-bin", ByteString("test-binary-data"))
      .build()

    if (in.id == null || in.id.isBlank)
      return Some(
        Future
          .failed(new GrpcServiceException(Status.INVALID_ARGUMENT.withDescription("Id invalid"), exceptionMetadata))
      )

    if (in.name == null || in.name.isBlank)
      return Some(
        Future
          .failed(new GrpcServiceException(Status.INVALID_ARGUMENT.withDescription("Name invalid"), exceptionMetadata))
      )

    if (in.paternalSurname == null || in.paternalSurname.isBlank)
      return Some(
        Future.failed(
          new GrpcServiceException(
            Status.INVALID_ARGUMENT.withDescription("Paternal surname invalid"),
            exceptionMetadata
          )
        )
      )
    None
  }
}
