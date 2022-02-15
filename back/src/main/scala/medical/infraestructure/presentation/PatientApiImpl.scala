package medical.infraestructure.presentation

import akka.NotUsed
import akka.actor.typed.ActorSystem
import akka.stream.scaladsl.Source
import medical.api.*
import medical.application.PatientService
import scribe.*

import java.util.UUID
import scala.concurrent.{ ExecutionContextExecutor, Future }

class PatientApiImpl(system: ActorSystem[?], patientService: PatientService) extends PatientApi {
  //  private implicit val sys: ActorSystem[_] = system
  private implicit val ec: ExecutionContextExecutor = system.executionContext

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
    Future.successful(PatientReply(UUID.randomUUID().toString, "Hector Miuler", "Malpica", "Gallegos"))
  }

  override def getFullPatient(in: PatientIdRequest): Future[FullPatientReply] = {

    Future.successful(FullPatientReply(UUID.randomUUID().toString, "Hector Miuler", "Malpica", "Gallegos"))
  }
}
