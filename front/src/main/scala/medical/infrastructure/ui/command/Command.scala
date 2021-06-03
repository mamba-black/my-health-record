package medical.infrastructure.ui.command

import medical.domain.Patient

sealed trait Command

case class FindPatient(name: String) extends Command

case class ShowPatient(patient: Patient) extends Command
