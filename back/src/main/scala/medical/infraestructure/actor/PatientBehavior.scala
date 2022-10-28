package medical.infraestructure.actor

import medical.domain.Patient
import akka.actor.typed.{ ActorRef, Behavior }
import akka.cluster.sharding.typed.scaladsl.EntityTypeKey
import akka.persistence.typed.PersistenceId
import akka.persistence.typed.scaladsl.{ Effect, EventSourcedBehavior }
import medical.infraestructure.actor.PatientBehavior.Identifier

object PatientBehavior {
  type Identifier = String
  val PatientTypeKey = EntityTypeKey[PatientBehavior.Command]("Patient")

  sealed trait Event
  final case class SavePatient(patient: Patient, replyTo: ActorRef[Patient]) extends Command
  final case class GetPatient(identifier: Identifier, replyTo: ActorRef[Patient]) extends Command

  sealed trait Command
  final case class PatientSaved(patient: Patient) extends Event {}

  sealed case class State(patient: Option[Patient])

  def apply(identifier: Identifier): Behavior[Command] = {
    EventSourcedBehavior[Command, Event, State](
      PersistenceId.ofUniqueId(identifier),
      State(None),
      commandHandler,
      eventHandler
    )
  }

  def commandHandler(state: State, command: Command): Effect[Event, State] = command match {
    case SavePatient(patient, replyTo) =>
      if (patient != state.patient.orNull)
        Effect.persist(PatientSaved(patient)).thenRun(newState => replyTo.tell(newState.patient.get))
      else Effect.none
    case _ => Effect.none
  }

  def eventHandler(state: State, event: Event): State = event match {
    case PatientSaved(patient) => state.copy(Some(patient))
    case _                     => state
  }
}
