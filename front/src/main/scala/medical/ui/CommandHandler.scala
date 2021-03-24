package medical.ui

import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveElement.Base
import io.frontroute.BrowserNavigation
import medical.command.{ Command, CommandBus, ShowPatient }
import wvlet.log.LogSupport

object CommandHandler extends LogSupport {
  def apply(commandBus: CommandBus): Binder[Base] = {

    commandBus.events --> Observer[Command](onNext = {
      case ShowPatient(patient) =>
        info(s"patient: $patient")
        BrowserNavigation.pushState(data = patient, url = s"/patient/${patient.id}")
      case _ => info(s"main: no hago nada")
    })
  }
}
