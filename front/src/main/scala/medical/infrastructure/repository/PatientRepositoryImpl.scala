package medical.infrastructure.repository

import com.raquo.airstream.core.{ EventStream, Observer }
import com.raquo.airstream.eventbus.EventBus
import io.grpc.stub.StreamObserver
import medical.api.patientapi.*
import medical.domain.Patient
import medical.domain.repository.PatientRepository
import scalapb.grpc.Channels
import scribe.*

import scala.concurrent.Future
//import scala.scalajs.concurrent.JSExecutionContext.Implicits.queue

import scala.scalajs.js.timers.setTimeout

private[infrastructure] class PatientRepositoryImpl extends PatientRepository {
  val grpcUrl = "https://0.0.0.0:9443" // "http://0.0.0.0:9090"
  val patientApi = PatientApiGrpcWeb.stub(Channels.grpcwebChannel(grpcUrl))

  override def findByName(name: String): EventStream[PatientReply] = {
    val eventBus = new EventBus[PatientReply]

    patientApi.find(
      PatientRequest(name),
      new StreamObserver[PatientReply] {
        override def onNext(patientReply: PatientReply): Unit = {
          debug(s"patient: $patientReply")
          eventBus.emit(patientReply)
        } // patients.update(_ :+ value)

        override def onError(throwable: Throwable): Unit = warn("onError")

        override def onCompleted(): Unit = debug("onCompleted")
      },
    )
    eventBus.events
  }

  override def getById(patientId: String): Future[FullPatientReply] = {
    debug("1")
    patientApi
      .getFullPatient(PatientIdRequest.of(patientId))

    // setTimeout(2000) {
    //  val patient = new Patient(
    //    patientId,
    //    new HumanName("Malpica", "Gallegos", Seq("Hector", "Miuler")),
    //    true,
    //    LocalDate.of(1979, 10, 13),
    //    Seq(new ContactPoint(SystemContactPoint.PHONE, "993990103")),
    //  )
    //  patientWriter.onNext(Some(patient))
    //  debug("TIMEOUT")
    // }
  }

  override def save(patient: Patient, patientWriter: Observer[Option[Patient]]): Unit = {
    setTimeout(2000) {
      patientWriter.onNext(Some(patient))
      debug("TIMEOUT")
    }
    ()
  }
}
