package medical.presentation

import akka.NotUsed
import akka.actor.typed.ActorSystem
import akka.stream.scaladsl.Source
import medical.api.{ PatientApi, PatientIdRequest, PatientReply, PatientRequest }
import medical.application.PatientService
import medical.domain.PatientBasic
import scribe._

import java.util.UUID
import scala.concurrent.Future

class PatientApiImpl(system: ActorSystem[_],
                     patientService: PatientService) extends PatientApi {
//  private implicit val sys: ActorSystem[_] = system

  /**
   * Sends a greeting
   */
  override def find(patientRequest: PatientRequest): Source[PatientReply, NotUsed] = {
    info(s"[${Thread.currentThread().getName}/${system.name}] 1.name: ${patientRequest.name}")
    val patientBasic: PatientBasic = patientService.find(patientRequest.name)
    system.log.info(s"2.name: ${patientRequest.name}")
    Source.single(PatientReply(patientBasic.id, patientBasic.name))
  }

  override def get(in: PatientIdRequest): Future[PatientReply] = {
    info(s"[${Thread.currentThread().getName}/${system.name}] 1.id: ${in.id}")
    system.log.info(s"2.id: ${in.id}")
    Future.successful(PatientReply(UUID.randomUUID().toString, "Hector Miuler", "Malpica", "Gallegos"))
  }
}
