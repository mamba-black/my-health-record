package medical.infraestructure.api

import akka.actor.typed.scaladsl.AskPattern.*
import akka.NotUsed
import akka.actor.typed.ActorSystem
import akka.stream.scaladsl.Source
import akka.util.Timeout
import medical.api.*
import medical.application.PatientService
import medical.domain.Patient
import medical.infraestructure.actor.PatientBehavior
import medical.infraestructure.actor.{ GetPatient, SavePatient }
import scribe.*

import java.util.UUID
import scala.concurrent.duration.DurationInt
import scala.concurrent.{ ExecutionContextExecutor, Future }

class PatientApiImpl(patientService: PatientService, implicit val system: ActorSystem[?]) extends PatientApi {
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
    val patientBehavior = system.systemActorOf(PatientBehavior(in.id), "patient")

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
    val patientBehavior = system.systemActorOf(PatientBehavior(in.id), "patient")
    patientBehavior
      .ask(replyTo => SavePatient(Patient(in.id, null, false, null, null), replyTo))
      .map(patient =>
        new FullPatientReply(
          patient.identifier,
          patient.name.`given`.mkString(" "),
          patient.name.fathersFamily,
          patient.name.mothersFamily
        )
      )
  }
}
