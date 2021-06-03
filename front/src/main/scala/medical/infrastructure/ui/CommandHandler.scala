package medical.infrastructure.ui

import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveElement.Base
import io.frontroute.BrowserNavigation
import medical.infrastructure.ui.command.{ Command, CommandBus, ShowPatient }
import scribe._

object CommandHandler {
  def apply(commandBus: CommandBus): Binder[Base] = {

    commandBus.events --> Observer[Command](onNext = {
      case ShowPatient(patient) =>
        info(s"patient: $patient")
        BrowserNavigation.pushState(data = patient, url = s"/patient/${patient.id}")
      case _ => info(s"main: no hago nada")
    })
  }
}
