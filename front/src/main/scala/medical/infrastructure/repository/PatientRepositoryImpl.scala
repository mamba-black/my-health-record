package medical.infrastructure.repository

import com.raquo.airstream.core.Observer
import medical.domain.repository.PatientRepository
import medical.domain.{ ContactPoint, HumanName, Patient, SystemContactPoint }
import scribe.debug

import java.time.LocalDate
import scala.scalajs.js.timers.setTimeout

private [infrastructure]
class PatientRepositoryImpl extends PatientRepository {
  override def findById(patientId: String, patientWriter: Observer[Option[Patient]]): Unit = {

    setTimeout(2000) {
      val patient = new Patient(
        patientId,
        new HumanName("Malpica", "Gallegos", Seq("Hector", "Miuler")),
        true,
        LocalDate.of(1979,10, 13),
        Seq(new ContactPoint(SystemContactPoint.PHONE, "993990103")),
      )
      patientWriter.onNext(Some(patient))
      debug("TIMEOUT")
    }
    ()
  }

  override def save(patient: Patient, patientWriter: Observer[Option[Patient]]): Unit = {
    setTimeout(2000) {
      patientWriter.onNext(Some(patient))
      debug("TIMEOUT")
    }
    ()
  }
}
