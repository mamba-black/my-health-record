package medical

import akka.NotUsed
import akka.actor.typed.ActorSystem
import akka.stream.scaladsl.Source
import medical.backend.{ PatientIdRequest, PatientReply, PatientRequest, PatientService }
import wvlet.log.LogSupport

import java.util.UUID
import scala.concurrent.Future

class PatientServiceImpl(system: ActorSystem[_]) extends PatientService with LogSupport {
  private implicit val sys: ActorSystem[_] = system

  /**
   * Sends a greeting
   */
  override def find(in: PatientRequest): Source[PatientReply, NotUsed] = {
    info(s"[${Thread.currentThread().getName}/${system.name}] 1.name: ${in.name}")
    system.log.info(s"2.name: ${in.name}")
    Source.single(PatientReply(UUID.randomUUID().toString, "Find: Hector Miuler Malpica Gallegos"))
  }

  override def get(in: PatientIdRequest): Future[PatientReply] = {
    info(s"[${Thread.currentThread().getName}/${system.name}] 1.id: ${in.id}")
    system.log.info(s"2.id: ${in.id}")
    Future.successful(PatientReply(UUID.randomUUID().toString, "GET: Hector Miuler Malpica Gallegos"))
  }
}
